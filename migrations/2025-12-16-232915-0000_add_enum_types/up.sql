-- Your SQL goes here
-- Create ENUM types
CREATE TYPE gender_type AS ENUM ('male', 'female');
CREATE TYPE role_type AS ENUM ('patient', 'donor', 'specialist', 'hospital');

-- Alter the users table to use the new types
ALTER TABLE users 
  ALTER COLUMN gender TYPE gender_type USING gender::gender_type,
  ALTER COLUMN role TYPE role_type USING role::role_type;