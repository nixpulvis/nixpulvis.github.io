# Run all pending Sequel migrations against the SQLite database.
# Migrations are versioned files in db/migrations/ and are safe to
# re-run (Sequel tracks which have already been applied).
#
# Usage: ruby db/migrate.rb

require 'sequel'

DB = Sequel.connect("sqlite://#{File.join(__dir__, 'site.db')}")
Sequel::Migrator.run(DB, File.join(__dir__, 'migrations'))
puts "Migrations complete."
