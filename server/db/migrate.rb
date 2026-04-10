require 'sequel'

DB = Sequel.connect("sqlite://#{File.join(__dir__, 'site.db')}")
Sequel::Migrator.run(DB, File.join(__dir__, 'migrations'))
puts "Migrations complete."
