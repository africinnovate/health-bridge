-- Revert: restore NOT NULL constraint on blood_request_id
-- Note: this will fail if any rows have NULL blood_request_id
ALTER TABLE appointments
    ALTER COLUMN blood_request_id SET NOT NULL;
