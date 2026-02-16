-- Add plan_group column for display grouping on pricing page
ALTER TABLE plans ADD COLUMN plan_group VARCHAR(20) NOT NULL DEFAULT 'personal';
ALTER TABLE plans ADD CONSTRAINT valid_plan_group CHECK (plan_group IN ('personal', 'enterprise', 'api', 'custom'));
CREATE INDEX idx_plans_group ON plans(plan_group);
