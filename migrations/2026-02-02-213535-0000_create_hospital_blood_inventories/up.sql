CREATE TABLE hospital_blood_inventories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hospital_id UUID NOT NULL REFERENCES hospitals(id) ON DELETE CASCADE,
    blood_type blood_type NOT NULL,
    units_available INT NOT NULL DEFAULT 0,
    bank_capacity INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(hospital_id, blood_type)
);

-- Index for faster lookups
CREATE INDEX idx_hospital_blood_inventories_hospital_id ON hospital_blood_inventories(hospital_id);
