CREATE TABLE specialties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE consultation_type AS ENUM (
    'video_call',
    'voice_call',
    'in_person'
);

CREATE TYPE days_of_week_type AS ENUM (
    'monday',
    'tuesday',
    'wednesday',
    'thursday',
    'friday',
    'saturday',
    'sunday'
);

CREATE TABLE specialists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    hospital_id UUID REFERENCES hospitals(id),
    specialty_id UUID NOT NULL REFERENCES specialties(id),
    bio TEXT,
    years_of_experience INT,
    consultation_type consultation_type NOT NULL,
    session_duration_minutes INT,
    primary_phone VARCHAR,
    secondary_phone VARCHAR,
    languages_spoken VARCHAR,
    country VARCHAR,
    time_zone VARCHAR,
    license_url TEXT,
    verified BOOLEAN NOT NULL DEFAULT false,
    suspended BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE specialist_availabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    specialist_id UUID NOT NULL REFERENCES specialists(id) ON DELETE CASCADE,
    day_of_week days_of_week_type NOT NULL,
    opens_at TIME NOT NULL,
    closes_at TIME NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_specialist_availabilities_specialist_id ON specialist_availabilities(specialist_id);