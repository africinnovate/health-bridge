-- Your SQL goes here
CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(32) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT email_verification_tokens_unique_user UNIQUE (user_id),
    CONSTRAINT email_verification_tokens_unique_token UNIQUE (token)
);

CREATE INDEX idx_email_verification_tokens_user_id
    ON email_verification_tokens (user_id);

CREATE INDEX idx_email_verification_tokens_expires_at
    ON email_verification_tokens (expires_at);
