# Rack entry point for the Sinatra CMS.
# Start with: bundle exec puma -C config.ru
require_relative 'app'

run App
