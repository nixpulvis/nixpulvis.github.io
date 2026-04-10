#!/usr/bin/env ruby
# Generates a complete backup:
#   db/backups/content/  - raw markdown with YAML front-matter (for restore)
#   db/backups/site/     - rendered static site (browsable offline)
#   db/backups/fortunes/ - fortune files
#   db/backups/backup.tar.gz - tarball of everything
#
# Usage: ruby db/backup.rb [--push]

require 'sequel'
require 'kramdown'
require 'kramdown-parser-gfm'
require 'rouge'
require 'json'
require 'yaml'
require 'erb'
require 'fileutils'
require 'date'

DB = Sequel.connect("sqlite://#{File.join(__dir__, 'site.db')}")

BACKUP_DIR = File.join(__dir__, 'backups')
SITE_DIR   = File.join(BACKUP_DIR, 'site')
CONTENT_DIR = File.join(BACKUP_DIR, 'content')

def render_markdown(text)
  Kramdown::Document.new(text, input: 'GFM', syntax_highlighter: 'rouge').to_html
end

def word_count(text)
  text.split(/\s+/).size
end

def format_date(date)
  return nil unless date
  date.strftime("%B %d, %Y")
end

def parse_metadata(page)
  page[:metadata] ? JSON.parse(page[:metadata]) : {}
rescue JSON::ParserError
  {}
end

def page_path(page)
  case page[:collection]
  when 'mathematics' then "/math/#{page[:slug]}"
  when 'standalone'  then page[:slug] == 'index' ? '/' : "/#{page[:slug]}"
  else "/#{page[:collection]}/#{page[:slug]}"
  end
end

def static_path(page)
  case page[:collection]
  when 'mathematics' then "math/#{page[:slug]}.html"
  when 'standalone'
    page[:slug] == 'index' ? 'index.html' : "#{page[:slug]}.html"
  else "#{page[:collection]}/#{page[:slug]}.html"
  end
end

def render_layout(title:, content:, meta: {})
  layout_path = File.join(__dir__, '..', 'views', 'layout.erb')
  layout_template = ERB.new(File.read(layout_path))

  @title = title
  @meta = meta

  # ERB yield trick: render content inside layout
  layout_template.result(binding { content })
end

def render_page_in_layout(page)
  meta = parse_metadata(page)
  body_html = render_markdown(page[:body])

  # Render the inner template
  template_name = page[:layout] || 'page'
  template_path = File.join(__dir__, '..', 'views', "#{template_name}.erb")
  template_path = File.join(__dir__, '..', 'views', 'page.erb') unless File.exist?(template_path)

  @page = page
  @content = body_html
  @meta = meta
  @title = page[:title]

  inner = ERB.new(File.read(template_path)).result(binding)

  # Wrap in layout
  layout_path = File.join(__dir__, '..', 'views', 'layout.erb')
  layout_erb = File.read(layout_path)

  # Use a simple yield-based approach
  ERB.new(layout_erb).result(binding { inner })
end

# --- Export raw markdown ---

def export_content
  puts "Exporting raw markdown..."
  DB[:pages].all.each do |page|
    collection = page[:collection]
    slug = page[:slug]
    dir = File.join(CONTENT_DIR, collection)
    FileUtils.mkdir_p(dir)

    meta = parse_metadata(page)
    front = {
      'collection' => collection,
      'slug' => slug,
      'layout' => page[:layout],
      'title' => page[:title],
      'draft' => page[:draft],
      'hidden' => page[:hidden],
      'published_at' => page[:published_at]&.to_s,
      'metadata' => meta.empty? ? nil : meta,
    }.compact

    File.open(File.join(dir, "#{slug}.md"), 'w') do |f|
      f.puts front.to_yaml
      f.puts '---'
      f.write page[:body]
    end
    puts "  content/#{collection}/#{slug}.md"
  end
end

# --- Render static site ---

def export_site
  puts "Rendering static site..."
  DB[:pages].all.each do |page|
    out_path = File.join(SITE_DIR, static_path(page))
    FileUtils.mkdir_p(File.dirname(out_path))

    html = render_page_in_layout(page)
    File.write(out_path, html)
    puts "  site/#{static_path(page)}"
  end
end

# --- Copy assets, uploads, fortunes ---

def export_assets
  puts "Copying assets..."

  # Copy from server/public/ if it exists (uploads, etc.)
  public_dir = File.join(__dir__, '..', 'public')
  if File.directory?(public_dir)
    Dir.glob(File.join(public_dir, '**', '*')).each do |src|
      next if File.directory?(src)
      rel = src.sub(public_dir + '/', '')
      dst = File.join(SITE_DIR, rel)
      FileUtils.mkdir_p(File.dirname(dst))
      FileUtils.cp(src, dst)
    end
    puts "  Copied public/ -> site/"
  end

  # Copy assets from parent repo (img, css, js) if not already in public
  %w[img css js].each do |asset_dir|
    src = File.join(__dir__, '..', '..', asset_dir)
    next unless File.directory?(src)
    dst = File.join(SITE_DIR, asset_dir)
    FileUtils.mkdir_p(dst)
    FileUtils.cp_r(Dir.glob(File.join(src, '*')), dst)
    puts "  Copied #{asset_dir}/ -> site/#{asset_dir}/"
  end
end

def export_fortunes
  puts "Copying fortunes..."
  src = File.join(__dir__, '..', '..', 'fortunes')
  return unless File.directory?(src)

  dst = File.join(BACKUP_DIR, 'fortunes')
  FileUtils.mkdir_p(dst)
  FileUtils.cp_r(Dir.glob(File.join(src, '*')), dst)
  puts "  Copied fortunes/"
end

# --- Create tarball ---

def create_tarball
  puts "Creating tarball..."
  tarball = File.join(BACKUP_DIR, 'backup.tar.gz')
  system("tar", "-czf", tarball, "-C", BACKUP_DIR, "content", "site", "fortunes")
  puts "  #{tarball}"
end

# --- Main ---

# Clean previous backup
FileUtils.rm_rf(SITE_DIR)
FileUtils.rm_rf(CONTENT_DIR)
FileUtils.rm_rf(File.join(BACKUP_DIR, 'fortunes'))

puts "Generating backup..."
puts

export_content
puts
export_site
puts
export_assets
puts
export_fortunes
puts
create_tarball

puts
puts "Backup complete: #{BACKUP_DIR}"

if ARGV.include?('--push')
  puts
  puts "Pushing backup to git..."
  Dir.chdir(BACKUP_DIR) do
    system("git", "init") unless File.directory?('.git')
    system("git", "add", "-A")
    system("git", "commit", "-m", "Backup #{Time.now.strftime('%Y-%m-%d %H:%M:%S')}")
    system("git", "push", "origin", "main") if system("git", "remote", "get-url", "origin", out: File::NULL, err: File::NULL)
  end
end
