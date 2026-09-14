-- Preserve the sale breakdown at order creation; old orders retain their amounts.
ALTER TABLE pay_purchase_orders ADD COLUMN IF NOT EXISTS pricing_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb;
