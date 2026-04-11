-- Drop triggers
DROP TRIGGER IF EXISTS update_consultation_benefits_updated_at ON consultation_benefits;
DROP TRIGGER IF EXISTS update_consultation_types_updated_at ON consultation_types;

-- Drop tables
DROP TABLE IF EXISTS consultation_type_benefits;
DROP TABLE IF EXISTS consultation_benefits;
DROP TABLE IF EXISTS consultation_types;
