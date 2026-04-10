Sequel.migration do
  up do
    create_table :users do
      primary_key :id
      String :username, null: false, unique: true
      String :password_hash, null: false
      DateTime :created_at, default: Sequel::CURRENT_TIMESTAMP
    end
  end

  down do
    drop_table :users
  end
end
