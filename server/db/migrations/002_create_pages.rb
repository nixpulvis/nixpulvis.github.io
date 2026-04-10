Sequel.migration do
  up do
    create_table :pages do
      primary_key :id
      String :collection, null: false
      String :slug, null: false
      String :title
      String :layout, null: false
      Text :body, null: false
      Text :metadata  # JSON: references, scripts, style, etc.
      Date :published_at
      TrueClass :draft, default: false
      TrueClass :hidden, default: false
      DateTime :created_at, default: Sequel::CURRENT_TIMESTAMP
      DateTime :updated_at, default: Sequel::CURRENT_TIMESTAMP

      unique [:collection, :slug]
    end
  end

  down do
    drop_table :pages
  end
end
