# Create the initial admin user for Warden authentication.
# Idempotent -- skips if the user already exists.
#
# Usage: ADMIN_USER=me ADMIN_PASS=secret ruby db/seed.rb

require 'sequel'
require 'bcrypt'

DB = Sequel.connect("sqlite://#{File.join(__dir__, 'site.db')}")

username = ENV.fetch('ADMIN_USER') { abort "Set ADMIN_USER env var" }
password = ENV.fetch('ADMIN_PASS') { abort "Set ADMIN_PASS env var" }

if DB[:users].where(username: username).count.zero?
  DB[:users].insert(
    username: username,
    password_hash: BCrypt::Password.create(password)
  )
  puts "Created admin user: #{username}"
else
  puts "Admin user already exists: #{username}"
end
