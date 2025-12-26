-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS blood_requests;

DROP TYPE IF EXISTS blood_type;
DROP TYPE IF EXISTS urgency_type;
DROP TYPE IF EXISTS timeline_type;
DROP TYPE IF EXISTS blood_request_status_type;
