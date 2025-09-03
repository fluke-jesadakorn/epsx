-- Add test notifications for the user info@epsx.io (f2bXM52g18ZuexhX2T9ldGmtdS72)
INSERT INTO notifications (id, title, body, notification_type, priority, created_at) VALUES
    ('550e8400-e29b-41d4-a716-446655440001', 'Welcome to EPSX!', 'Thank you for joining our analytics platform. Explore powerful trading insights and real-time data.', 'system', 'normal', NOW() - INTERVAL '2 hours'),
    ('550e8400-e29b-41d4-a716-446655440002', 'New Feature: Advanced Charts', 'We''ve added new charting capabilities to help you analyze market trends better.', 'feature', 'high', NOW() - INTERVAL '1 day'),
    ('550e8400-e29b-41d4-a716-446655440003', 'Market Alert: High Volatility', 'Increased market volatility detected. Review your positions and risk management.', 'data', 'urgent', NOW() - INTERVAL '30 minutes'),
    ('550e8400-e29b-41d4-a716-446655440004', 'Security Update', 'Your account security has been enhanced with new authentication features.', 'security', 'high', NOW() - INTERVAL '3 days'),
    ('550e8400-e29b-41d4-a716-446655440005', 'Weekly Portfolio Summary', 'Your portfolio performance summary for this week is now available.', 'data', 'normal', NOW() - INTERVAL '1 week');

-- Link these notifications to the user
INSERT INTO user_notifications (id, user_id, notification_id, delivery_status, delivered_at) VALUES
    ('660e8400-e29b-41d4-a716-446655440001', 'f2bXM52g18ZuexhX2T9ldGmtdS72', '550e8400-e29b-41d4-a716-446655440001', 'delivered', NOW() - INTERVAL '2 hours'),
    ('660e8400-e29b-41d4-a716-446655440002', 'f2bXM52g18ZuexhX2T9ldGmtdS72', '550e8400-e29b-41d4-a716-446655440002', 'delivered', NOW() - INTERVAL '1 day'),
    ('660e8400-e29b-41d4-a716-446655440003', 'f2bXM52g18ZuexhX2T9ldGmtdS72', '550e8400-e29b-41d4-a716-446655440003', 'delivered', NOW() - INTERVAL '30 minutes'),
    ('660e8400-e29b-41d4-a716-446655440004', 'f2bXM52g18ZuexhX2T9ldGmtdS72', '550e8400-e29b-41d4-a716-446655440004', 'delivered', NOW() - INTERVAL '3 days'),
    ('660e8400-e29b-41d4-a716-446655440005', 'f2bXM52g18ZuexhX2T9ldGmtdS72', '550e8400-e29b-41d4-a716-446655440005', 'delivered', NOW() - INTERVAL '1 week');
