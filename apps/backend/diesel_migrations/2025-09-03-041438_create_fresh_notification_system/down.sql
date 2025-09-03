-- Drop fresh notification system
DROP TABLE IF EXISTS user_notification_preferences CASCADE;
DROP TABLE IF EXISTS notification_deliveries CASCADE;
DROP TABLE IF EXISTS notifications CASCADE;
DROP TABLE IF EXISTS fcm_tokens CASCADE;
DROP TABLE IF EXISTS fcm_topics CASCADE;

-- Drop fresh notification types
DROP TYPE IF EXISTS delivery_status CASCADE;
DROP TYPE IF EXISTS delivery_channel CASCADE;
DROP TYPE IF EXISTS notification_type CASCADE;
DROP TYPE IF EXISTS notification_priority CASCADE;