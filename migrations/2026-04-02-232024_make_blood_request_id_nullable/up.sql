-- Make blood_request_id nullable on appointments
ALTER TABLE appointments
    ALTER COLUMN blood_request_id DROP NOT NULL;
