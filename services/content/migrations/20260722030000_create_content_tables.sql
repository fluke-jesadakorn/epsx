CREATE TABLE IF NOT EXISTS public.themes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    colors_json JSONB NOT NULL DEFAULT '{}',
    fonts_json JSONB NOT NULL DEFAULT '{}',
    spacing_json JSONB NOT NULL DEFAULT '{}',
    breakpoints_json JSONB NOT NULL DEFAULT '{}',
    radius_json JSONB DEFAULT '{}',
    is_default BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS public.pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    locale VARCHAR(10) NOT NULL DEFAULT 'en',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    blocks_json JSONB NOT NULL DEFAULT '[]',
    seo_json JSONB DEFAULT '{}',
    theme_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS public.block_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    block_type VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50) NOT NULL,
    description TEXT,
    schema_json JSONB NOT NULL DEFAULT '{}',
    default_props_json JSONB NOT NULL DEFAULT '{}',
    admin_only BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.edit_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id UUID NOT NULL REFERENCES public.pages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ
);
