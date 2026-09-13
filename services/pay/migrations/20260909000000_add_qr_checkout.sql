ALTER TABLE pay_merchant_intents ADD COLUMN IF NOT EXISTS deposit_address text;
CREATE UNIQUE INDEX IF NOT EXISTS pay_merchant_deposit_address_unique
    ON pay_merchant_intents(chain_id, deposit_address) WHERE deposit_address IS NOT NULL;
ALTER TABLE pay_merchant_operations DROP CONSTRAINT IF EXISTS pay_merchant_operations_kind_check;
ALTER TABLE pay_merchant_operations ADD CONSTRAINT pay_merchant_operations_kind_check
    CHECK(kind IN ('pay','deposit','release','refund','dispute','resolve-release','resolve-refund','collect'));
