-- Create consultation_types table
CREATE TABLE consultation_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    duration_minutes INTEGER NOT NULL DEFAULT 30,
    base_price NUMERIC(20, 2) NOT NULL DEFAULT 0.00,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed default records
INSERT INTO consultation_types (name, description, duration_minutes, base_price) VALUES
    ('Quick Consultation', 'A brief consultation for straightforward queries and minor health concerns.', 15, 0.00),
    ('Standard Session', 'A standard in-depth consultation session for comprehensive assessment and treatment plans.', 30, 0.00);

-- Create consultation_benefits table
CREATE TABLE consultation_benefits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(150) NOT NULL UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create consultation_type_benefits join table (many-to-many)
CREATE TABLE consultation_type_benefits (
    consultation_type_id UUID NOT NULL REFERENCES consultation_types(id) ON DELETE CASCADE,
    consultation_benefit_id UUID NOT NULL REFERENCES consultation_benefits(id) ON DELETE CASCADE,
    PRIMARY KEY (consultation_type_id, consultation_benefit_id)
);

-- Triggers for updated_at
CREATE TRIGGER update_consultation_types_updated_at
    BEFORE UPDATE ON consultation_types
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_consultation_benefits_updated_at
    BEFORE UPDATE ON consultation_benefits
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Indexes
CREATE INDEX idx_consultation_type_benefits_type_id ON consultation_type_benefits(consultation_type_id);
CREATE INDEX idx_consultation_type_benefits_benefit_id ON consultation_type_benefits(consultation_benefit_id);
