.PHONY: db seed convert restore backup server clean

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

clean:
	rm -f db/site.db
	rm -rf db/backups
