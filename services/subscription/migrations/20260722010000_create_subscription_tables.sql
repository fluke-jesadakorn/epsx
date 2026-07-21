CREATE TABLE IF NOT EXISTS public.subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    amount VARCHAR(78) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    chain_id VARCHAR(10) NOT NULL,
    interval INTEGER NOT NULL,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    plan_id UUID REFERENCES public.subscription_plans(id),
    status VARCHAR(20) DEFAULT 'active',
    account_id VARCHAR(42),
    payment_token VARCHAR(42),
    vault_position_id VARCHAR(100),
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
