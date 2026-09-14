CREATE TABLE IF NOT EXISTS public.templates (
    id VARCHAR(66) PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    channel VARCHAR(20) NOT NULL,
    subject TEXT,
    body TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.notifications (
    id VARCHAR(66) PRIMARY KEY,
    user_id VARCHAR(66),
    channel VARCHAR(20) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    template_id VARCHAR(66),
    subject TEXT,
    body TEXT NOT NULL,
    data JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error TEXT,
    sent_at TIMESTAMPTZ(6),
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ(6),
    title TEXT,
    notification_type VARCHAR(50),
    priority VARCHAR(20),
    action_url TEXT
);

CREATE INDEX IF NOT EXISTS idx_notif_user
    ON public.notifications (user_id ASC, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notif_status
    ON public.notifications (status ASC);
