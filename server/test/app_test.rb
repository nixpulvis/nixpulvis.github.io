require 'minitest/autorun'
require 'rack/test'
require 'sequel'
require 'sequel/extensions/migration'
require 'bcrypt'
require 'json'
require 'tmpdir'
require 'fileutils'
require 'tempfile'

# Set up test DB before loading app
TEST_DB_PATH = File.join(Dir.tmpdir, "nixpulvis_test_#{$$}.db")
ENV['DATABASE_URL'] = "sqlite://#{TEST_DB_PATH}"

ENV['RACK_ENV'] = 'test'
require_relative '../app'

# Run migrations once at load time
Sequel::Migrator.run(DB, File.join(__dir__, '..', 'db', 'migrations'))

class AppTest < Minitest::Test
  include Rack::Test::Methods

  def app
    App
  end

  def setup
    DB[:pages].delete
    DB[:users].delete

    # Seed a test user
    DB[:users].insert(
      username: 'admin',
      password_hash: BCrypt::Password.create('testpass')
    )

    # Seed some test pages
    DB[:pages].insert(
      collection: 'standalone', slug: 'index', title: 'Home',
      layout: 'page', body: '# Welcome', published_at: Date.today,
      draft: false, hidden: false
    )
    DB[:pages].insert(
      collection: 'standalone', slug: 'about', title: 'About',
      layout: 'page', body: 'About me.', published_at: Date.today,
      draft: false, hidden: false
    )
    DB[:pages].insert(
      collection: 'ramblings', slug: 'hello-world', title: 'Hello World',
      layout: 'rambling', body: 'This is my first post.',
      published_at: Date.new(2024, 1, 15), draft: false, hidden: false
    )
    DB[:pages].insert(
      collection: 'ramblings', slug: 'draft-post', title: 'Draft Post',
      layout: 'rambling', body: 'Not ready yet.',
      published_at: Date.new(2024, 2, 1), draft: true, hidden: false
    )
    DB[:pages].insert(
      collection: 'projects', slug: 'cool-project', title: 'Cool Project',
      layout: 'project', body: 'A project I built.',
      metadata: '{"references":["https://github.com/example"]}',
      draft: false, hidden: false
    )
    DB[:pages].insert(
      collection: 'mathematics', slug: 'fibonacci', title: 'Fibonacci',
      layout: 'article', body: 'The sequence 0, 1, 1, 2, 3...',
      published_at: Date.new(2023, 5, 10), draft: false, hidden: false
    )
    DB[:pages].insert(
      collection: 'references', slug: 'terminal', title: 'The Terminal',
      layout: 'reference', body: '- [Alacritty](https://github.com/alacritty)',
      draft: false, hidden: false
    )
    DB[:pages].insert(
      collection: 'research', slug: 'secure-mpc', title: 'Secure MPC',
      layout: 'article', body: 'Multi-party computation research.',
      metadata: '{"subcollection":"articles"}',
      published_at: Date.new(2023, 7, 23), draft: false, hidden: false
    )

    # Set up a temp fortunes dir
    @fortune_dir = File.join(Dir.tmpdir, "nixpulvis_fortunes_#{$$}")
    FileUtils.mkdir_p(@fortune_dir)
    File.write(File.join(@fortune_dir, '2024'), "A test fortune.\n%\n")

    # Stub the fortune path by pointing __dir__/.. to our temp
    # We can't easily do this, so we'll skip fortune file tests that need FS
  end

  def teardown
    DB[:pages].delete
    DB[:users].delete
    FileUtils.rm_rf(@fortune_dir) if @fortune_dir
  end

  def self.shutdown
    File.delete(TEST_DB_PATH) if File.exist?(TEST_DB_PATH)
  end

  Minitest.after_run { shutdown }

  # --- Homepage ---

  def test_get_homepage
    get '/'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Welcome'
    assert_includes last_response.body, 'nixpulvis'
  end

  def test_homepage_uses_layout
    get '/'
    assert_includes last_response.body, '<header>'
    assert_includes last_response.body, '<footer>'
    assert_includes last_response.body, 'Nathan Lilienthal'
  end

  # --- About ---

  def test_get_about
    get '/about'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'About me.'
  end

  # --- Collection indexes ---

  def test_get_ramblings_index
    get '/ramblings'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Hello World'
    assert_includes last_response.body, 'Draft Post'
    assert_includes last_response.body, 'Fibonacci'
  end

  def test_ramblings_index_shows_draft_class
    get '/ramblings'
    assert_includes last_response.body, 'class="draft"'
  end

  def test_get_projects_index
    get '/projects'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Cool Project'
  end

  def test_get_math_index
    get '/math'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Fibonacci'
  end

  def test_get_research_index
    get '/research'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Secure MPC'
  end

  def test_get_references_index
    get '/references'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'The Terminal'
  end

  def test_get_musings_index
    get '/musings'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Music'
  end

  # --- Individual content pages ---

  def test_get_rambling
    get '/ramblings/hello-world'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Hello World'
    assert_includes last_response.body, 'first post'
    assert_includes last_response.body, 'published'
  end

  def test_get_draft_rambling
    get '/ramblings/draft-post'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'drafted'
  end

  def test_get_project
    get '/projects/cool-project'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Cool Project'
    assert_includes last_response.body, 'github.com/example'
  end

  def test_get_math_page
    get '/math/fibonacci'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Fibonacci'
  end

  def test_get_reference
    get '/references/terminal'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'The Terminal'
  end

  def test_get_research_page
    get '/research/secure-mpc'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Secure MPC'
  end

  # --- 404 ---

  def test_missing_page_returns_404
    get '/ramblings/nonexistent'
    assert_equal 404, last_response.status
  end

  def test_missing_standalone_returns_404
    get '/nope'
    assert_equal 404, last_response.status
  end

  def test_missing_math_returns_404
    get '/math/nope'
    assert_equal 404, last_response.status
  end

  # --- Auth ---

  def test_login_with_valid_credentials
    post '/login', username: 'admin', password: 'testpass'
    assert_equal 302, last_response.status
  end

  def test_login_with_invalid_credentials
    post '/login', username: 'admin', password: 'wrong'
    # Warden failure_app redirects to /
    assert_equal 302, last_response.status
    follow_redirect!
    refute_includes last_response.body, 'Logout'
  end

  def test_logout
    # Login first
    post '/login', username: 'admin', password: 'testpass'
    post '/logout'
    assert_equal 302, last_response.status
  end

  # --- Uploads (auth required) ---

  def test_upload_requires_auth
    post '/uploads'
    # Should redirect via warden failure
    assert_equal 302, last_response.status
  end

  def test_upload_without_file_returns_400
    post '/login', username: 'admin', password: 'testpass'
    post '/uploads'
    assert_equal 400, last_response.status
  end

  def test_upload_with_file
    post '/login', username: 'admin', password: 'testpass'

    tempfile = Tempfile.new(['test', '.txt'])
    tempfile.write('hello')
    tempfile.rewind

    post '/uploads', file: Rack::Test::UploadedFile.new(tempfile.path, 'text/plain', false, original_filename: 'test.txt')

    assert_equal 200, last_response.status
    assert_match %r{/uploads/\d{4}/\d{2}/test\.txt}, last_response.body
  ensure
    tempfile&.close
    tempfile&.unlink
  end

  # --- Fortunes ---

  def test_get_fortunes_index
    get '/fortunes'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'Fortunes'
  end

  def test_fortune_post_requires_auth
    post '/fortunes/2024', fortune: 'A new one'
    assert_equal 302, last_response.status
  end

  # --- Markdown rendering ---

  def test_markdown_rendered_to_html
    DB[:pages].insert(
      collection: 'standalone', slug: 'md-test', title: 'MD Test',
      layout: 'page', body: "**bold** and `code`",
      draft: false, hidden: false
    )

    get '/md-test'
    assert_equal 200, last_response.status
    assert_includes last_response.body, '<strong>bold</strong>'
    assert_includes last_response.body, '<code>code</code>'
  end

  def test_code_blocks_get_syntax_highlighting
    DB[:pages].insert(
      collection: 'standalone', slug: 'code-test', title: 'Code Test',
      layout: 'page', body: "```ruby\nputs 'hi'\n```",
      draft: false, hidden: false
    )

    get '/code-test'
    assert_equal 200, last_response.status
    assert_includes last_response.body, 'highlight'
  end

  # --- Layout elements ---

  def test_layout_includes_nav
    get '/'
    assert_includes last_response.body, 'Projects'
    assert_includes last_response.body, 'Ramblings'
    assert_includes last_response.body, '/references'
    assert_includes last_response.body, '/musings'
  end

  def test_layout_includes_css
    get '/'
    assert_includes last_response.body, 'style.css'
    assert_includes last_response.body, 'syntax.css'
  end

  def test_page_title_in_head
    get '/about'
    assert_includes last_response.body, '<title>nixpulvis - About</title>'
  end

  # --- Metadata (scripts, style, references) ---

  def test_page_with_scripts_metadata
    DB[:pages].insert(
      collection: 'standalone', slug: 'scripted', title: 'Scripted',
      layout: 'page', body: 'Has scripts.',
      metadata: '{"scripts":["/js/extra.js"]}',
      draft: false, hidden: false
    )

    get '/scripted'
    assert_equal 200, last_response.status
    assert_includes last_response.body, '/js/extra.js'
  end

  def test_page_with_style_metadata
    DB[:pages].insert(
      collection: 'standalone', slug: 'styled', title: 'Styled',
      layout: 'page', body: 'Has style.',
      metadata: '{"style":"/css/custom.css"}',
      draft: false, hidden: false
    )

    get '/styled'
    assert_equal 200, last_response.status
    assert_includes last_response.body, '/css/custom.css'
  end

  # --- Word count display ---

  def test_rambling_shows_word_count
    get '/ramblings/hello-world'
    assert_includes last_response.body, '5 words'
  end

  def test_ramblings_index_shows_word_count
    get '/ramblings'
    assert_includes last_response.body, 'words'
  end

  # --- Hidden pages filtered ---

  def test_hidden_pages_excluded_from_ramblings_index
    DB[:pages].insert(
      collection: 'ramblings', slug: 'hidden-post', title: 'Hidden',
      layout: 'rambling', body: 'Secret.', hidden: true, draft: false,
      published_at: Date.today
    )

    get '/ramblings'
    refute_includes last_response.body, '>Hidden<'
  end

  def test_hidden_pages_excluded_from_projects_index
    DB[:pages].insert(
      collection: 'projects', slug: 'hidden-proj', title: 'Hidden Project',
      layout: 'project', body: 'Secret.', hidden: true, draft: false
    )

    get '/projects'
    refute_includes last_response.body, 'Hidden Project'
  end
end
