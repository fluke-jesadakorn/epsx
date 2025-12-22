-- Create system_settings table for global admin console settings
CREATE TABLE IF NOT EXISTS system_settings (
    id SERIAL PRIMARY KEY,
    category VARCHAR(50) NOT NULL,
    key VARCHAR(100) NOT NULL,
    value JSONB NOT NULL DEFAULT '{}',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(category, key)
);

-- Create index for fast category lookups
CREATE INDEX IF NOT EXISTS idx_system_settings_category ON system_settings(category);

-- Insert default settings
INSERT INTO system_settings (category, key, value, description) VALUES
    -- General Settings
    ('general', 'systemName', '"EPSX Admin Console"', 'Display name for the admin console'),
    ('general', 'adminEmail', '"admin@epsx.com"', 'Primary admin contact email'),
    ('general', 'maintenanceMode', 'false', 'Enable/disable maintenance mode'),
    
    -- Notification Settings
    ('notifications', 'emailNotifications', 'true', 'Enable email notifications'),
    ('notifications', 'pushNotifications', 'false', 'Enable push notifications'),
    ('notifications', 'smsNotifications', 'true', 'Enable SMS notifications'),
    ('notifications', 'securityAlerts', 'true', 'Enable security alert notifications'),
    
    -- Security Settings
    ('security', 'sessionTimeout', '30', 'Session timeout in minutes'),
    
    -- Appearance Settings
    ('appearance', 'theme', '"light"', 'Theme mode: light, dark, or auto'),
    ('appearance', 'primaryColor', '"#FF8C00"', 'Primary accent color')
ON CONFLICT (category, key) DO NOTHING;
