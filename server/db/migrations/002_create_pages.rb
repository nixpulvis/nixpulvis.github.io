# Central content table. Each row corresponds to one markdown page, replacing
# the Jekyll convention of _collection/YYYY-MM-DD-slug.md files with YAML
# front-matter.
#
# Collections: ramblings, projects, mathematics, research, references,
#              musings, standalone (for index.md, about.md, etc.)
#
# The `metadata` column is a JSON blob for variable front-matter fields
# that differ between layouts (e.g. references for projects, scripts/style
# for pages with custom JS/CSS, subcollection for research articles vs notes).
Sequel.migration do
  up do
    create_table :pages do
      primary_key :id
      String :collection, null: false   # e.g. "ramblings", "projects", "standalone"
      String :slug, null: false         # URL-friendly identifier
      String :title
      String :layout, null: false       # ERB template name: rambling, article, project, etc.
      Text :body, null: false           # raw markdown content
      Text :metadata                    # JSON: references, scripts, style, subcollection
      Date :published_at
      TrueClass :draft, default: false
      TrueClass :hidden, default: false # hidden pages are excluded from indexes
      DateTime :created_at, default: Sequel::CURRENT_TIMESTAMP
      DateTime :updated_at, default: Sequel::CURRENT_TIMESTAMP

      unique [:collection, :slug]
    end
  end

  down do
    drop_table :pages
  end
end
