-- This file should undo anything in `up.sql`
-- Revert to VARCHAR
ALTER TABLE users 
  ALTER COLUMN gender TYPE VARCHAR USING gender::VARCHAR,
  ALTER COLUMN role TYPE VARCHAR USING role::VARCHAR;

-- Drop the ENUM types
DROP TYPE IF EXISTS role_type;
DROP TYPE IF EXISTS gender_type;