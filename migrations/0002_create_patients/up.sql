CREATE TABLE patients (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    blood_type VARCHAR,
    medical_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
