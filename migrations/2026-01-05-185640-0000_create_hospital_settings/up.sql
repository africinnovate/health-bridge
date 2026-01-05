-- Your SQL goes here
CREATE TABLE hospital_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hospital_id UUID NOT NULL UNIQUE REFERENCES hospitals(id) ON DELETE CASCADE,

    -- Blood request notifications
    donation_requests BOOLEAN NOT NULL DEFAULT TRUE,

    -- Donor appointments
    new_donor_appointments BOOLEAN NOT NULL DEFAULT TRUE,
    donor_appointment_reminders BOOLEAN NOT NULL DEFAULT TRUE,

    -- System activity
    login_alerts BOOLEAN NOT NULL DEFAULT TRUE,
    account_notifications BOOLEAN NOT NULL DEFAULT TRUE,

    -- Delivery options
    email_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    sms_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    push_notifications BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
