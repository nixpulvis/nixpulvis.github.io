# Server Makefile
#
# Typical workflow for initial setup from Jekyll content:
#   make convert   # parse Jekyll files into db/backups/content/
#   make db        # run Sequel migrations to create tables
#   make restore   # load backup markdown into the database
#   make seed      # create admin user (requires ADMIN_USER/ADMIN_PASS)
#   make server    # start Puma
#
# For ongoing use:
#   make backup    # export DB to static site + markdown + tarball
#   make test      # run the test suite

.PHONY: db seed convert restore backup server test clean

db:
	ruby db/migrate.rb

seed:
	ruby db/seed.rb

convert:
	ruby db/convert.rb

restore: db
	ruby db/restore.rb

backup:
	ruby db/backup.rb

server: db
	bundle exec puma -C config.ru

test:
	bundle exec ruby test/app_test.rb

clean:
	rm -f db/site.db
	rm -rf db/backups
