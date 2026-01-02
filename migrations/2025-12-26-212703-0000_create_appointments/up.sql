-- Your SQL goes here
CREATE TYPE cancelled_by AS ENUM ('hospital', 'donor', 'patient');
CREATE TYPE appointment_type AS ENUM ('donor', 'patient');
CREATE TYPE appointment_status AS ENUM ('created', 'confirmed', 'rescheduled', 'cancelled', 'completed');

CREATE TABLE appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blood_request_id UUID NOT NULL REFERENCES blood_requests(id) ON DELETE CASCADE,

    hospital_id UUID NOT NULL REFERENCES hospitals(id),
    user_id UUID NOT NULL REFERENCES users(id), -- donor or patient

    appointment_type appointment_type NOT NULL,
    status appointment_status NOT NULL DEFAULT 'created',

    scheduled_time TIMESTAMPTZ NOT NULL,
    previous_time TIMESTAMPTZ,

    cancelled_by cancelled_by,
    cancelled_by_id UUID REFERENCES users(id),
    cancelled_reason TEXT,
    cancelled_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_appointments_blood_request_id
    ON appointments (blood_request_id);

CREATE INDEX idx_appointments_hospital_id
    ON appointments (hospital_id);

CREATE INDEX idx_appointments_user_id
    ON appointments (user_id);

CREATE INDEX idx_appointments_scheduled_time
    ON appointments (scheduled_time);

CREATE INDEX idx_appointments_status
    ON appointments (status);

