-- Drop tables
DROP TABLE IF EXISTS referral_rewards;
DROP TABLE IF EXISTS referrals;
DROP TABLE IF EXISTS app_configs;

-- Remove columns from users
ALTER TABLE users DROP COLUMN IF EXISTS referral_link;
ALTER TABLE users DROP COLUMN IF EXISTS referral_code;

-- Drop enums
DROP TYPE IF EXISTS reward_type;
DROP TYPE IF EXISTS referral_status;
