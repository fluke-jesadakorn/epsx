ALTER TABLE wallet_group_assignments
  DROP COLUMN assignment_source,
  DROP COLUMN payment_reference,
  DROP COLUMN subscription_id,
  DROP COLUMN auto_renew,
  DROP COLUMN next_billing_date,
  DROP COLUMN assignment_metadata;
