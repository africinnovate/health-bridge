-- Your SQL goes here
CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,

    -- Notification categories
    appointment_reminders BOOLEAN NOT NULL DEFAULT TRUE,
    specialist_recommendations BOOLEAN NOT NULL DEFAULT TRUE,
    donation_alerts BOOLEAN NOT NULL DEFAULT TRUE,
    account_notifications BOOLEAN NOT NULL DEFAULT TRUE,

    -- Notification methods
    email_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    sms_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    push_notifications BOOLEAN NOT NULL DEFAULT TRUE,

    -- Privacy
    medical_profile_visibility TEXT NOT NULL DEFAULT 'only_assigned',
    allow_specialists_view_history BOOLEAN NOT NULL DEFAULT TRUE,
    allow_app_analytics BOOLEAN NOT NULL DEFAULT TRUE,
    allow_marketing_notifications BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
