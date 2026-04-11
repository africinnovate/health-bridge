CREATE TABLE consultation_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    specialist_id UUID NOT NULL REFERENCES specialists(id) ON DELETE CASCADE,
    consultation_type_id UUID NOT NULL REFERENCES consultation_types(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    description TEXT,
    custom_price NUMERIC(15, 2),
    custom_duration_minutes INT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE consultation_package_benefits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES consultation_packages(id) ON DELETE CASCADE,
    consultation_benefit_id UUID REFERENCES consultation_benefits(id) ON DELETE CASCADE,
    custom_title VARCHAR,
    custom_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT benefit_provided CHECK (
        (consultation_benefit_id IS NOT NULL) OR (custom_title IS NOT NULL)
    )
);

CREATE INDEX idx_consultation_packages_specialist ON consultation_packages(specialist_id);
CREATE INDEX idx_consultation_package_benefits_package ON consultation_package_benefits(package_id);

CREATE TRIGGER update_consultation_packages_updated_at
    BEFORE UPDATE ON consultation_packages
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
