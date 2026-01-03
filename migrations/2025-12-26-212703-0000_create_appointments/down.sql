-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS appointments;

DROP TYPE IF EXISTS cancelled_by;
DROP TYPE IF EXISTS appointment_type;
DROP TYPE IF EXISTS appointment_status;
