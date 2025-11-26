-- Remove seeded subscription plans
-- This rollback removes the default subscription plans added in the up migration

-- Remove plans in reverse order of creation to avoid foreign key constraints
DELETE FROM permission_groups WHERE slug = 'api-developer';
DELETE FROM permission_groups WHERE slug = 'enterprise';
DELETE FROM permission_groups WHERE slug = 'pro';
DELETE FROM permission_groups WHERE slug = 'starter';
DELETE FROM permission_groups WHERE slug = 'free';