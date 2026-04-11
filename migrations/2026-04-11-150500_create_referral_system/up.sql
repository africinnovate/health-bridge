-- Create ENUM types for referral system
CREATE TYPE referral_status AS ENUM ('pending', 'active', 'verified');
CREATE TYPE reward_type AS ENUM ('earned', 'applied');

-- Modify users table
ALTER TABLE users ADD COLUMN referral_code VARCHAR(50) UNIQUE;
ALTER TABLE users ADD COLUMN referral_link TEXT;

-- Generate random referral codes for existing users (8 chars)
UPDATE users SET referral_code = UPPER(SUBSTR(MD5(id::text), 1, 8)) WHERE referral_code IS NULL;

-- Make referral_code NOT NULL after backfilling
ALTER TABLE users ALTER COLUMN referral_code SET NOT NULL;

-- Create app_configs table
CREATE TABLE app_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(100) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed default configs
INSERT INTO app_configs (key, value, description) VALUES 
('reward_point', '100', 'The Naira equivalent of 1 reward point'),
('referral_points_patient', '1', 'Points earned for referring a patient'),
('referral_points_donor', '1', 'Points earned for referring a donor'),
('referral_points_specialist', '1', 'Points earned for referring a specialist'),
('referral_points_hospital', '1', 'Points earned for referring a hospital');

-- Create referrals table
CREATE TABLE referrals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    referrer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    referred_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    referral_code VARCHAR(50) NOT NULL, -- The code used during registration
    status referral_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(referrer_id, referred_user_id)
);

-- Create referral_rewards table
CREATE TABLE referral_rewards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    referral_id UUID REFERENCES referrals(id) ON DELETE SET NULL, -- Optional if not linked to a specific referral (e.g. bonus)
    points INT4 NOT NULL,
    reward_type reward_type NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX idx_users_referral_code ON users(referral_code);
CREATE INDEX idx_referrals_referrer_id ON referrals(referrer_id);
CREATE INDEX idx_referrals_referred_user_id ON referrals(referred_user_id);
CREATE INDEX idx_referral_rewards_user_id ON referral_rewards(user_id);
