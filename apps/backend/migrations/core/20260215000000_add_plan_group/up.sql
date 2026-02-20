-- Add plan_group column for display grouping on pricing page
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'plans' AND column_name = 'plan_group') THEN
    ALTER TABLE plans ADD COLUMN plan_group VARCHAR(20) NOT NULL DEFAULT 'personal';
  END IF;
END $$;

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'valid_plan_group') THEN
    ALTER TABLE plans ADD CONSTRAINT valid_plan_group CHECK (plan_group IN ('personal', 'enterprise', 'api', 'custom'));
  END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_plans_group ON plans(plan_group);
