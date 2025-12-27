-- Your SQL goes here
CREATE TYPE blood_type AS ENUM ('apositive', 'anegative', 'bpositive', 'bnegative', 'abpositive', 'abnegative', 'opositive', 'onegative');
CREATE TYPE urgency_type AS ENUM ('standard', 'urgent');
CREATE TYPE timeline_type AS ENUM ('request_created', 'visible_to_donors', 'donation_appointment_scheduled', 'donation_completed', 'request_fulfilled');
CREATE TYPE blood_request_status_type AS ENUM ('confirmed', 'accepted', 'completed', 'cancelled');

CREATE TABLE blood_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hospital_id UUID NOT NULL REFERENCES hospitals(id) ON DELETE CASCADE,
    donor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_id UUID REFERENCES users(id) ON DELETE CASCADE,

    ref_id TEXT NOT NULL UNIQUE,

    units INTEGER,

    blood_type blood_type,
    urgency urgency_type,
    timeline_status timeline_type,
    request_status blood_request_status_type,

    request_reason TEXT,
    note TEXT,

    cancelled_by UUID REFERENCES users(id),
    cancelled_at TIMESTAMPTZ,
    cancelled_reason TEXT,

    license_status BOOLEAN NOT NULL DEFAULT false,
    accreditation_doc_url TEXT,

    preferred_time TIMESTAMPTZ,
    donated_at TIMESTAMPTZ,
    administered_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
