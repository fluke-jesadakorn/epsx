-- Initial seed migration
-- Creates default admin user, theme, and configuration

BEGIN;

-- Default admin user (password hash placeholder - replace in production)
-- bcrypt('epsx-admin-2024', 10) = placeholder
INSERT INTO users (id, address, email, role, kyc_status, created_at, updated_at)
VALUES (
  '0x0000000000000000000000000000000000000001',
  '0x0000000000000000000000000000000000000001',
  'admin@epsx.io',
  'admin',
  'approved',
  NOW(),
  NOW()
)
ON CONFLICT (address) DO UPDATE SET role = 'admin', updated_at = NOW();

-- Editor user
INSERT INTO users (id, address, email, role, kyc_status, created_at, updated_at)
VALUES (
  '0x0000000000000000000000000000000000000002',
  '0x0000000000000000000000000000000000000002',
  'editor@epsx.io',
  'editor',
  'approved',
  NOW(),
  NOW()
)
ON CONFLICT (address) DO UPDATE SET role = 'editor', updated_at = NOW();

-- Default subscription plans
INSERT INTO plans (id, name, description, merchant, period, amount, token, active, created_at, updated_at)
VALUES
  ('plan_free', 'Free', 'Basic free tier', 'epsx', 'month', '0', 'USDC', true, NOW(), NOW()),
  ('plan_pro', 'Pro', 'Professional tier with advanced features', 'epsx', 'month', '29000000', 'USDC', true, NOW(), NOW()),
  ('plan_enterprise', 'Enterprise', 'Custom enterprise pricing', 'epsx', 'month', '0', 'USDC', true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Default welcome page (only if content service is empty)
INSERT INTO pages (slug, title, blocks_json, status, created_at, updated_at)
VALUES (
  'welcome',
  'Welcome to EPSX',
  '[]',
  'published',
  NOW(),
  NOW()
)
ON CONFLICT (slug) DO NOTHING;

COMMIT;
