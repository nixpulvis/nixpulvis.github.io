#!/usr/bin/env ruby
# Populate the database from backup markdown files produced by either
# db/convert.rb (initial Jekyll import) or db/backup.rb (ongoing snapshots).
#
# For each .md file in db/backups/content/<collection>/:
#   - Parses YAML front-matter for page attributes
#   - Upserts into the pages table (updates existing, inserts new)
#
# Also restores:
#   - Uploaded media from db/backups/site/uploads/ -> public/uploads/
#   - Fortune files from db/backups/fortunes/ -> ../fortunes/
#
# Requires migrations to have been run first (make db).
#
# Usage: ruby db/restore.rb

require 'sequel'
require 'yaml'
require 'json'
require 'fileutils'
require 'date'

DB = Sequel.connect("sqlite://#{File.join(__dir__, 'site.db')}")

BACKUP_DIR  = File.join(__dir__, 'backups')
CONTENT_DIR = File.join(BACKUP_DIR, 'content')

def parse_backup_file(path)
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

def restore_content
  puts "Restoring content from backups..."
  count = 0

  Dir.glob(File.join(CONTENT_DIR, '**', '*.md')).each do |path|
    front_matter, body = parse_backup_file(path)

    collection = front_matter['collection']
    slug = front_matter['slug']
    next unless collection && slug

    metadata = front_matter['metadata']
    published_at = front_matter['published_at']
    published_at = Date.parse(published_at) if published_at.is_a?(String)

    record = {
      collection: collection,
      slug: slug,
      title: front_matter['title'],
      layout: front_matter['layout'] || 'page',
      body: body,
      metadata: metadata ? JSON.generate(metadata) : nil,
      published_at: published_at,
      draft: front_matter['draft'] || false,
      hidden: front_matter['hidden'] || false,
      updated_at: Time.now,
    }

    existing = DB[:pages].where(collection: collection, slug: slug)
    if existing.count > 0
      existing.update(record)
      puts "  Updated: #{collection}/#{slug}"
    else
      record[:created_at] = Time.now
      DB[:pages].insert(record)
      puts "  Inserted: #{collection}/#{slug}"
    end
    count += 1
  end

  puts "Restored #{count} pages."
end

def restore_uploads
  src = File.join(BACKUP_DIR, 'site', 'uploads')
  return unless File.directory?(src)

  dst = File.join(__dir__, '..', 'public', 'uploads')
  FileUtils.mkdir_p(dst)
  FileUtils.cp_r(Dir.glob(File.join(src, '*')), dst)
  puts "Restored uploads."
end

def restore_fortunes
  src = File.join(BACKUP_DIR, 'fortunes')
  return unless File.directory?(src)

  dst = File.join(__dir__, '..', '..', 'fortunes')
  FileUtils.mkdir_p(dst)
  FileUtils.cp_r(Dir.glob(File.join(src, '*')), dst)
  puts "Restored fortunes."
end

# --- Main ---

restore_content
puts
restore_uploads
puts
restore_fortunes

puts
puts "Restore complete."
