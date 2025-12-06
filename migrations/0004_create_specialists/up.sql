CREATE TABLE specialists (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    hospital_id UUID REFERENCES hospitals(id) ON DELETE SET NULL,
    speciality VARCHAR,
    bio TEXT,
    email VARCHAR,
    phone VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
