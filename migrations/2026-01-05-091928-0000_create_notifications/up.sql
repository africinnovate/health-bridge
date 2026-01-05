-- Your SQL goes here
-- Migration: Create notifications table

-- Create notification category enum
CREATE TYPE notification_category AS ENUM (
    'verification',
    'blood_request',
    'appointment',
    'system'
);

-- Create notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category notification_category NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    related_id UUID, -- ID of related entity (specialist_id, hospital_id, blood_request_id, etc.)
    related_type VARCHAR(50), -- Type of related entity ('specialist', 'hospital', 'blood_request', etc.)
    metadata Text, -- Additional data like specialist name, specialty, etc.
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID REFERENCES users(id), -- User who triggered the notification (optional)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ
);

-- Create indexes for better query performance
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_category ON notifications(category);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read) WHERE is_read = FALSE;

-- Example queries:
-- 
-- Get unread notifications for a user:
-- SELECT * FROM notifications 
-- WHERE user_id = 'user-uuid' AND is_read = FALSE 
-- ORDER BY created_at DESC;
--
-- Get all verification notifications:
-- SELECT * FROM notifications 
-- WHERE category = 'verification' 
-- ORDER BY created_at DESC;
--
-- Mark notification as read:
-- UPDATE notifications 
-- SET is_read = TRUE, read_at = NOW() 
-- WHERE id = 'notification-uuid';