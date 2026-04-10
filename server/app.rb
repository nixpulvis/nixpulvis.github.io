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

    def page_path(page)
      collection = page[:collection]
      slug = page[:slug]
      case collection
      when 'mathematics' then "/math/#{slug}"
      when 'standalone'  then "/#{slug}"
      else "/#{collection}/#{slug}"
      end
    end

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

  get '/ramblings' do
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
    # Fallback: if subcollection tagging isn't set, just show all
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
