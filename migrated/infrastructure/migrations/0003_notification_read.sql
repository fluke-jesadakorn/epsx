-- 0003_notification_read.sql
-- Add read tracking to notifications for the /notifications page.

ALTER TABLE notifications
  ADD COLUMN IF NOT EXISTS read_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS title VARCHAR(255),
  ADD COLUMN IF NOT EXISTS notification_type VARCHAR(40) DEFAULT 'system',
  ADD COLUMN IF NOT EXISTS priority VARCHAR(20) DEFAULT 'normal',
  ADD COLUMN IF NOT EXISTS action_url TEXT;

CREATE INDEX IF NOT EXISTS idx_notif_user_unread
  ON notifications (user_id, created_at DESC)
  WHERE read_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_notif_priority
  ON notifications (user_id, priority, created_at DESC);
