CREATE TABLE IF NOT EXISTS public.events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    event_name VARCHAR(100) NOT NULL,
    properties_json JSONB DEFAULT '{}',
    chain_id VARCHAR(10),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
