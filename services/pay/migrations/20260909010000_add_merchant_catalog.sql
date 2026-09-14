-- Additive catalog and immutable checkout presentation; legacy contracts stay intact.
CREATE TABLE IF NOT EXISTS pay_merchant_products (
 id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL REFERENCES pay_merchants(id),
 environment TEXT NOT NULL CHECK(environment IN ('test','live')),
 name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '', prices JSONB NOT NULL,
 duration_days INTEGER CHECK(duration_days > 0), enabled BOOLEAN NOT NULL DEFAULT true,
 revision BIGINT NOT NULL DEFAULT 1, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS pay_merchant_products_catalog ON pay_merchant_products(merchant_id,environment,enabled);
ALTER TABLE pay_merchant_intents ADD COLUMN IF NOT EXISTS checkout_snapshot JSONB NOT NULL DEFAULT '{}';
ALTER TABLE pay_merchant_links ADD COLUMN IF NOT EXISTS payment_method TEXT NOT NULL DEFAULT 'contract' CHECK(payment_method IN ('contract','transfer'));
ALTER TABLE pay_merchant_endpoints ADD COLUMN IF NOT EXISTS replaces_id TEXT REFERENCES pay_merchant_endpoints(id);
UPDATE pay_merchant_intents p SET checkout_snapshot=jsonb_build_object(
 'merchant_name',m.name,'item_name',coalesce(p.description,'Payment'),'kind','merchant',
 'payee',p.payee,'token',p.token,'amount',p.amount,'duration_days',NULL,'product_id',NULL
) FROM pay_merchants m WHERE m.id=p.merchant_id AND p.checkout_snapshot='{}'::jsonb;
