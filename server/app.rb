# Sinatra CMS application for nixpulvis.com
#
# Replaces Jekyll's static site generation with a dynamic server backed by
# SQLite. Content is stored as markdown in a `pages` table, rendered at
# request time with kramdown/Rouge. Authentication via Warden/bcrypt
# gates write operations (fortune posting, media uploads).
#
# Routes mirror the original Jekyll permalink structure:
#   /ramblings/:slug, /projects/:slug, /math/:slug,
#   /research/:slug, /references/:slug, /musings/:slug
#
# The DATABASE_URL env var overrides the default SQLite path (used by tests).

require 'sinatra/base'
require 'sequel'
require 'bcrypt'
require 'warden'
require 'rack/session/cookie'
require 'kramdown'
require 'kramdown-parser-gfm'
require 'rouge'
require 'json'
require 'fileutils'

DB = Sequel.connect(ENV.fetch('DATABASE_URL', "sqlite://#{File.join(__dir__, 'db', 'site.db')}"))

# Warden password strategy
Warden::Strategies.add(:password) do
  def valid?
    params['username'] && params['password']
  end

  def authenticate!
    user = DB[:users].where(username: params['username']).first
    if user && BCrypt::Password.new(user[:password_hash]) == params['password']
      success!(user)
    else
      fail!("Invalid username or password")
    end
  end
end

class App < Sinatra::Base
  set :views, File.join(__dir__, 'views')
  set :public_folder, File.join(__dir__, 'public')
  set :method_override, true

  use Rack::Session::Cookie,
    key: 'nixpulvis.session',
    secret: ENV.fetch('SESSION_SECRET') { SecureRandom.hex(64) }

  use Warden::Manager do |config|
    config.default_strategies :password
    config.failure_app = self
    config.serialize_into_session { |user| user[:id] }
    config.serialize_from_session { |id| DB[:users].where(id: id).first }
  end

  helpers do
    def warden
      env['warden']
    end

    def authenticated?
      warden.authenticated?
    end

    def authenticate!
      warden.authenticate!
    end

    def current_user
      warden.user
    end

    # Render GitHub-Flavored Markdown to HTML with Rouge syntax highlighting.
    # This replaces Jekyll's kramdown build step with on-the-fly rendering.
    def render_markdown(text)
      Kramdown::Document.new(
        text,
        input: 'GFM',
        syntax_highlighter: 'rouge'
      ).to_html
    end

    def word_count(text)
      text.split(/\s+/).size
    end

    def excerpt(text, words: 50)
      text.split(/\s+/).first(words).join(' ')
    end

    # Build URL path for a page, matching the Jekyll permalink scheme from _config.yml.
    # Mathematics uses /math/:name, standalone pages live at the root.
    def page_path(page)
      collection = page[:collection]
      slug = page[:slug]
      case collection
      when 'mathematics' then "/math/#{slug}"
      when 'standalone'  then "/#{slug}"
      else "/#{collection}/#{slug}"
      end
    end

    # Deserialize the JSON metadata column (references, scripts, style, etc.)
    def parse_metadata(page)
      page[:metadata] ? JSON.parse(page[:metadata]) : {}
    rescue JSON::ParserError
      {}
    end

    def format_date(date)
      return nil unless date
      date.strftime("%B %d, %Y")
    end
  end

  # --- Auth routes ---

  post '/login' do
    warden.authenticate!
    redirect back
  end

  post '/unauthenticated' do
    redirect '/'
  end

  post '/logout' do
    warden.logout
    redirect '/'
  end

  # --- Upload route ---
  # Saves files to public/uploads/YYYY/MM/filename, served statically by
  # Sinatra (or nginx in production). Returns the URL path to the uploaded file.

  post '/uploads' do
    authenticate!
    file = params[:file]
    halt 400, "No file provided" unless file

    now = Time.now
    dir = File.join(settings.public_folder, 'uploads', now.strftime('%Y'), now.strftime('%m'))
    FileUtils.mkdir_p(dir)

    filename = File.basename(file[:filename])
    path = File.join(dir, filename)
    File.write(path, file[:tempfile].read, mode: 'wb')

    "/uploads/#{now.strftime('%Y')}/#{now.strftime('%m')}/#{filename}"
  end

  # --- Fortune routes ---
  # Fortunes are plain text files in ../fortunes/ (one per year), delimited by
  # "%\n". Reading is public; adding new fortunes requires authentication.

  get '/fortunes' do
    @title = 'Fortunes'
    fortune_dir = File.join(__dir__, '..', 'fortunes')
    @years = Dir.children(fortune_dir).sort.reverse rescue []
    erb :fortunes_index
  end

  get '/fortunes/:year' do
    @title = "Fortunes #{params[:year]}"
    path = File.join(__dir__, '..', 'fortunes', params[:year])
    halt 404, "Not found" unless File.exist?(path)
    @year = params[:year]
    @fortunes = File.read(path).split("\n%\n")
    erb :fortunes_year
  end

  post '/fortunes/:year' do
    authenticate!
    path = File.join(__dir__, '..', 'fortunes', params[:year])
    fortune = params[:fortune]&.strip
    halt 400, "No fortune provided" if fortune.nil? || fortune.empty?

    File.open(path, 'a') do |f|
      f.puts fortune
      f.puts "%"
    end

    redirect "/fortunes/#{params[:year]}"
  end

  # --- Collection index routes ---
  # These mirror the Jekyll collection index pages (ramblings.md, projects.md, etc.)
  # with the same filtering (hidden, draft) and sorting behavior.

  get '/ramblings' do
    # Ramblings index includes both ramblings and mathematics, matching the
    # original Jekyll template which concatenates and sorts both collections.
    @title = 'Ramblings'
    @pages = DB[:pages]
      .where(collection: ['ramblings', 'mathematics'])
      .where(hidden: false)
      .order(Sequel.desc(:published_at))
      .all
    erb :ramblings_index
  end

  get '/projects' do
    @title = 'Projects'
    @finished = DB[:pages]
      .where(collection: 'projects', hidden: false, draft: false)
      .all
    @drafted = DB[:pages]
      .where(collection: 'projects', hidden: false, draft: true)
      .all
    erb :projects_index
  end

  get '/math' do
    @title = 'Mathematics'
    @pages = DB[:pages]
      .where(collection: 'mathematics')
      .order(Sequel.desc(:published_at))
      .all
    erb :mathematics_index
  end

  # Research is split into articles and notes (mirroring the Jekyll _research/articles/
  # and _research/notes/ subdirectory structure). The subcollection is stored in metadata
  # during convert, or inferred from slug patterns as a fallback.
  get '/research' do
    @title = 'Research'
    @articles = DB[:pages]
      .where(collection: 'research', hidden: false)
      .where(Sequel.like(:slug, '%article%') | Sequel.like(:metadata, '%"subcollection":"articles"%'))
      .order(Sequel.desc(:published_at))
      .all
    @notes = DB[:pages]
      .where(collection: 'research', hidden: false)
      .where(Sequel.like(:slug, '%note%') | Sequel.like(:metadata, '%"subcollection":"notes"%'))
      .order(Sequel.desc(:published_at))
      .all
    if @articles.empty? && @notes.empty?
      @articles = DB[:pages]
        .where(collection: 'research', hidden: false)
        .order(Sequel.desc(:published_at))
        .all
    end
    erb :research_index
  end

  get '/references' do
    @title = 'References'
    @pages = DB[:pages].where(collection: 'references').all
    erb :references_index
  end

  get '/musings' do
    @title = 'Music'
    @pages = DB[:pages].where(collection: 'musings').all
    erb :musings_index
  end

  # --- Content routes ---
  # Individual pages are fetched by collection + slug, rendered through kramdown,
  # and wrapped in the layout matching the page's `layout` column (rambling, article,
  # project, note, reference, or page). The ERB template name matches the layout.

  get '/math/:slug' do
    @page = DB[:pages].where(collection: 'mathematics', slug: params[:slug]).first
    halt 404, "Not found" unless @page
    @title = @page[:title]
    @meta = parse_metadata(@page)
    @content = render_markdown(@page[:body])
    erb @page[:layout].to_sym
  end

  get '/about' do
    @page = DB[:pages].where(collection: 'standalone', slug: 'about').first
    halt 404, "Not found" unless @page
    @title = @page[:title]
    @meta = parse_metadata(@page)
    @content = render_markdown(@page[:body])
    erb @page[:layout].to_sym
  end

  # Homepage falls back to an empty page if no index record exists in the DB,
  # allowing the server to boot before content is imported.
  get '/' do
    @page = DB[:pages].where(collection: 'standalone', slug: 'index').first
    if @page
      @title = @page[:title]
      @meta = parse_metadata(@page)
      @content = render_markdown(@page[:body])
      erb @page[:layout].to_sym
    else
      @title = 'Home'
      @content = ''
      erb :page
    end
  end

  get '/:collection/:slug' do
    collection = params[:collection]
    @page = DB[:pages].where(collection: collection, slug: params[:slug]).first
    halt 404, "Not found" unless @page
    @title = @page[:title]
    @meta = parse_metadata(@page)
    @content = render_markdown(@page[:body])
    erb @page[:layout].to_sym
  end

  get '/:slug' do
    @page = DB[:pages].where(collection: 'standalone', slug: params[:slug]).first
    halt 404, "Not found" unless @page
    @title = @page[:title]
    @meta = parse_metadata(@page)
    @content = render_markdown(@page[:body])
    erb @page[:layout].to_sym
  end
end
