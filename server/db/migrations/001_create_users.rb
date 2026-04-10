# Authentication table for Warden. Passwords are stored as bcrypt hashes.
# See db/seed.rb for creating the initial admin user.
Sequel.migration do
  up do
    create_table :users do
      primary_key :id
      String :username, null: false, unique: true
      String :password_hash, null: false  # bcrypt digest
      DateTime :created_at, default: Sequel::CURRENT_TIMESTAMP
    end
  end

  down do
    drop_table :users
  end
end
