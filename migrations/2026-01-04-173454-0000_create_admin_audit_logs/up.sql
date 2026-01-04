-- Your SQL goes here
-- Optional: Create audit log table to store reasons for actions
-- This is HIGHLY recommended for compliance and tracking purposes

CREATE TYPE action_type AS ENUM (
    'specialist_verified',
    'specialist_unverified', 
    'specialist_suspended',
    'specialist_unsuspended',
    'hospital_approved',
    'hospital_revoked'
);

CREATE TABLE admin_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_id UUID NOT NULL REFERENCES users(id),
    target_type VARCHAR(50) NOT NULL, -- 'specialist' or 'hospital'
    target_id UUID NOT NULL,
    action_type action_type NOT NULL,
    reason TEXT,
    metadata TEXT, -- Store additional context like previous values
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_admin_id ON admin_audit_logs(admin_id);
CREATE INDEX idx_audit_logs_target ON admin_audit_logs(target_type, target_id);
CREATE INDEX idx_audit_logs_created_at ON admin_audit_logs(created_at DESC);

-- Example query to view audit trail:
-- SELECT 
--     al.*,
--     u.first_name || ' ' || u.last_name as admin_name
-- FROM admin_audit_logs al
-- JOIN users u ON u.id = al.admin_id
-- WHERE target_type = 'specialist' AND target_id = 'some-uuid'
-- ORDER BY created_at DESC;