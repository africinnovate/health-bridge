CREATE TABLE patients (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    blood_type VARCHAR,
    chronic_illnesses TEXT,
    allergies TEXT,
    medications TEXT,
    existing_conditions TEXT,
    primary_physician UUID REFERENCES users(id),
    hmo_number VARCHAR(255),
    emergency_contact_name VARCHAR(255),
    emergency_contact_phone VARCHAR(20),
    medical_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
