CREATE TYPE hospital_type AS ENUM ('clinic', 'general', 'teaching', 'specialist', 'diagnostic');

CREATE TABLE hospitals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    name VARCHAR NOT NULL,
    hospital_type hospital_type,
    address TEXT NOT NULL,
    city TEXT NOT NULL,
    country TEXT NOT NULL,

    primary_phone VARCHAR NOT NULL,
    emergency_phone VARCHAR,
    email VARCHAR,

    license_number VARCHAR NOT NULL,
    license_status BOOLEAN NOT NULL DEFAULT false,
    accreditation_doc_url TEXT NOT NULL,

    has_blood_bank BOOLEAN NOT NULL DEFAULT false,
    accepting_donors BOOLEAN NOT NULL DEFAULT false,
    donating_operating_hours TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    deleted_at TIMESTAMPTZ NULLABLE
);

CREATE INDEX idx_hospitals_user_id ON hospitals(user_id);
