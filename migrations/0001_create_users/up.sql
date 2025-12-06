CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    phone VARCHAR,
    gender VARCHAR,
    dob DATE,
    password_hash TEXT NOT NULL,
    role VARCHAR NOT NULL, -- e.g. patient, specialist, admin
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
