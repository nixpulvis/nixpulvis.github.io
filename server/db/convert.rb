#!/usr/bin/env ruby
# One-time conversion of the Jekyll site into the backup format that
# db/restore.rb can load into the database.
#
# Reads:
#   ../_ramblings/*.md, ../_projects/*.md, etc.  (Jekyll collections)
#   ../img/, ../css/, ../js/                     (static assets)
#   ../fortunes/                                 (fortune text files)
#
# Writes:
#   db/backups/content/<collection>/<slug>.md    (YAML front-matter + markdown)
#   db/backups/site/assets/{img,css,js}/         (static asset copies)
#   db/backups/fortunes/                         (fortune file copies)
#
# Usage: ruby db/convert.rb

require 'yaml'
require 'fileutils'
require 'date'

REPO_ROOT  = File.expand_path('../../..', __FILE__)
BACKUP_DIR = File.join(__dir__, 'backups')

# Maps Jekyll collection directories to their DB collection name and default
# layout. Research has two subdirectories that become subcollections.
COLLECTIONS = {
  '_ramblings'  => { collection: 'ramblings',   default_layout: 'rambling' },
  '_projects'   => { collection: 'projects',     default_layout: 'project' },
  '_mathematics' => { collection: 'mathematics', default_layout: 'article' },
  '_research/articles' => { collection: 'research', default_layout: 'article', subcollection: 'articles' },
  '_research/notes'    => { collection: 'research', default_layout: 'note',    subcollection: 'notes' },
  '_references' => { collection: 'references',   default_layout: 'reference' },
  '_musings'    => { collection: 'musings',       default_layout: 'page' },
}

# Standalone pages (not in collections)
STANDALONE_PAGES = %w[index.md about.md playground.md musings.md]

# Split a Jekyll file into its YAML front-matter hash and markdown body.
def parse_jekyll_file(path)
  content = File.read(path)
  if content =~ /\A---\s*\n(.*?\n)---\s*\n(.*)\z/m
    front_matter = YAML.safe_load($1, permitted_classes: [Date, Time]) || {}
    body = $2
  else
    front_matter = {}
    body = content
  end
  [front_matter, body]
end

# Jekyll filenames encode the date: 2018-02-05-hello-world.md -> hello-world
def slug_from_filename(filename)
  name = File.basename(filename, File.extname(filename))
  name.sub(/^\d{4}-\d{2}-\d{2}-/, '')
end

# Extract the date from a Jekyll-style dated filename, if present.
def date_from_filename(filename)
  name = File.basename(filename)
  if name =~ /^(\d{4}-\d{2}-\d{2})/
    Date.parse($1) rescue nil
  end
end

def convert_collection(source_dir, config)
  dir = File.join(REPO_ROOT, source_dir)
  return unless File.directory?(dir)

  collection = config[:collection]
  output_dir = File.join(BACKUP_DIR, 'content', collection)
  FileUtils.mkdir_p(output_dir)

  Dir.glob(File.join(dir, '**', '*.md')).each do |path|
    front_matter, body = parse_jekyll_file(path)

    slug = slug_from_filename(path)
    layout = front_matter.delete('layout') || config[:default_layout]
    title = front_matter.delete('title')
    draft = front_matter.delete('draft') || false
    hidden = front_matter.delete('hidden') || false
    published = front_matter.delete('published')
    date = front_matter.delete('date') || date_from_filename(path)

    # Jekyll's `published: false` is equivalent to our `draft: true`
    draft = true if published == false

    # Everything not consumed above goes into the metadata JSON blob.
    # This preserves layout-specific front-matter like `references` (projects),
    # `scripts` / `style` (custom JS/CSS pages), and `subcollection` (research).
    metadata = front_matter.reject { |k, _| k == 'permalink' }
    metadata['subcollection'] = config[:subcollection] if config[:subcollection]
    output = {
      'collection' => collection,
      'slug' => slug,
      'layout' => layout,
      'title' => title,
      'draft' => draft,
      'hidden' => hidden,
      'published_at' => date&.to_s,
      'metadata' => metadata.empty? ? nil : metadata,
    }.compact

    backup_path = File.join(output_dir, "#{slug}.md")
    File.open(backup_path, 'w') do |f|
      f.puts output.to_yaml
      f.puts '---'
      f.write body
    end

    puts "  #{collection}/#{slug}"
  end
end

def convert_standalone_pages
  output_dir = File.join(BACKUP_DIR, 'content', 'standalone')
  FileUtils.mkdir_p(output_dir)

  STANDALONE_PAGES.each do |page_file|
    path = File.join(REPO_ROOT, page_file)
    next unless File.exist?(path)

    front_matter, body = parse_jekyll_file(path)
    slug = File.basename(page_file, '.md')
    layout = front_matter.delete('layout') || 'page'
    title = front_matter.delete('title')
    metadata = front_matter.reject { |k, _| %w[permalink].include?(k) }

    output = {
      'collection' => 'standalone',
      'slug' => slug,
      'layout' => layout == 'default' ? 'page' : layout,
      'title' => title,
      'metadata' => metadata.empty? ? nil : metadata,
    }.compact

    backup_path = File.join(output_dir, "#{slug}.md")
    File.open(backup_path, 'w') do |f|
      f.puts output.to_yaml
      f.puts '---'
      f.write body
    end

    puts "  standalone/#{slug}"
  end
end

def copy_assets
  # Copy img/
  src_img = File.join(REPO_ROOT, 'img')
  if File.directory?(src_img)
    dst_img = File.join(BACKUP_DIR, 'site', 'assets', 'img')
    FileUtils.mkdir_p(dst_img)
    FileUtils.cp_r(Dir.glob(File.join(src_img, '*')), dst_img)
    puts "  Copied img/ -> backups/site/assets/img/"
  end

  # Copy css/
  src_css = File.join(REPO_ROOT, 'css')
  if File.directory?(src_css)
    dst_css = File.join(BACKUP_DIR, 'site', 'assets', 'css')
    FileUtils.mkdir_p(dst_css)
    FileUtils.cp_r(Dir.glob(File.join(src_css, '*')), dst_css)
    puts "  Copied css/ -> backups/site/assets/css/"
  end

  # Copy js/
  src_js = File.join(REPO_ROOT, 'js')
  if File.directory?(src_js)
    dst_js = File.join(BACKUP_DIR, 'site', 'assets', 'js')
    FileUtils.mkdir_p(dst_js)
    FileUtils.cp_r(Dir.glob(File.join(src_js, '*')), dst_js)
    puts "  Copied js/ -> backups/site/assets/js/"
  end
end

def copy_fortunes
  src = File.join(REPO_ROOT, 'fortunes')
  return unless File.directory?(src)

  dst = File.join(BACKUP_DIR, 'fortunes')
  FileUtils.mkdir_p(dst)
  FileUtils.cp_r(Dir.glob(File.join(src, '*')), dst)
  puts "  Copied fortunes/"
end

# --- Main ---

puts "Converting Jekyll site to backup format..."
puts

puts "Collections:"
COLLECTIONS.each do |source_dir, config|
  convert_collection(source_dir, config)
end

puts
puts "Standalone pages:"
convert_standalone_pages

puts
puts "Assets:"
copy_assets

puts
puts "Fortunes:"
copy_fortunes

puts
puts "Done. Backups written to: #{BACKUP_DIR}"
