-- Revert: remove notes field from appointments
ALTER TABLE appointments
    DROP COLUMN notes;
