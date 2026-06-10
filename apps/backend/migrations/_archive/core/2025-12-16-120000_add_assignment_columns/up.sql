ALTER TABLE wallet_group_assignments
  ADD COLUMN assignment_source VARCHAR(50) DEFAULT 'manual' NOT NULL,
  ADD COLUMN payment_reference VARCHAR(255),
  ADD COLUMN subscription_id VARCHAR(255),
  ADD COLUMN auto_renew BOOLEAN DEFAULT FALSE NOT NULL,
  ADD COLUMN next_billing_date TIMESTAMPTZ,
  ADD COLUMN assignment_metadata JSONB DEFAULT '{}' NOT NULL;
