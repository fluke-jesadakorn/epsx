-- Seed default subscription plans
-- This migration ensures every EPSX database starts with essential subscription plans

-- Free Plan (Entry-level plan with basic features)
INSERT INTO permission_groups (
    id, name, slug, description, group_type, group_metadata,
    price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
) VALUES (
    gen_random_uuid(),
    'Free Plan',
    'free',
    'Perfect for getting started with basic analytics',
    'subscription',
    '{
        "permissions": ["epsx:analytics:view:5", "epsx:rankings:view:5"],
        "features": [
            "Basic analytics view",
            "5 stock rankings limit",
            "Community support",
            "Daily market updates"
        ],
        "limits": {
            "analytics_queries_per_day": 10,
            "stocks_tracked": 5,
            "historical_data_months": 1
        }
    }'::jsonb,
    0.00,
    'USD',
    'monthly',
    true,
    false,
    1,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- Starter Plan (Individual investors)
INSERT INTO permission_groups (
    id, name, slug, description, group_type, group_metadata,
    price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
) VALUES (
    gen_random_uuid(),
    'Starter Plan',
    'starter',
    'Ideal for individual investors and traders',
    'subscription',
    '{
        "permissions": ["epsx:analytics:view:25", "epsx:rankings:view:25", "epsx:analytics:export", "epsx:alerts:create"],
        "features": [
            "Advanced analytics",
            "25 stock rankings",
            "Export functionality",
            "Price alerts",
            "Email support"
        ],
        "limits": {
            "analytics_queries_per_day": 50,
            "stocks_tracked": 25,
            "historical_data_months": 6,
            "alerts": 10
        }
    }'::jsonb,
    14.99,
    'USD',
    'monthly',
    true,
    false,
    2,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- Pro Plan (Serious traders)
INSERT INTO permission_groups (
    id, name, slug, description, group_type, group_metadata,
    price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
) VALUES (
    gen_random_uuid(),
    'Pro Plan',
    'pro',
    'For serious traders who need advanced tools',
    'subscription',
    '{
        "permissions": ["epsx:analytics:view:100", "epsx:rankings:view:100", "epsx:analytics:export", "epsx:analytics:advanced", "epsx:alerts:create", "epsx:alerts:manage", "epsx:portfolio:view", "epsx:portfolio:manage"],
        "features": [
            "Advanced analytics",
            "100 stock rankings",
            "Export functionality",
            "Advanced charting tools",
            "Portfolio management",
            "Unlimited price alerts",
            "Priority support",
            "Real-time data"
        ],
        "limits": {
            "analytics_queries_per_day": 200,
            "stocks_tracked": 100,
            "historical_data_months": 24,
            "alerts": -1,
            "portfolios": 5
        },
        "highlighted": true
    }'::jsonb,
    29.99,
    'USD',
    'monthly',
    true,
    true,
    3,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- Enterprise Plan (Professional teams)
INSERT INTO permission_groups (
    id, name, slug, description, group_type, group_metadata,
    price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
) VALUES (
    gen_random_uuid(),
    'Enterprise Plan',
    'enterprise',
    'Complete solution for professional teams and institutions',
    'subscription',
    '{
        "permissions": ["epsx:*:*", "epsx:api:access", "epsx:enterprise:*"],
        "features": [
            "Unlimited stock analysis",
            "Unlimited rankings access",
            "Full API access",
            "Premium analytics suite",
            "Advanced portfolio tools",
            "Custom integrations",
            "Dedicated account manager",
            "24/7 priority support",
            "White-label options"
        ],
        "limits": {
            "analytics_queries_per_day": -1,
            "stocks_tracked": -1,
            "historical_data_months": -1,
            "alerts": -1,
            "portfolios": -1,
            "api_calls_per_month": 1000000
        },
        "highlighted": false,
        "contact_sales": true
    }'::jsonb,
    99.99,
    'USD',
    'monthly',
    true,
    false,
    4,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- API Developer Plan (Developers)
INSERT INTO permission_groups (
    id, name, slug, description, group_type, group_metadata,
    price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
) VALUES (
    gen_random_uuid(),
    'API Developer',
    'api-developer',
    'For developers building on EPSX platform',
    'subscription',
    '{
        "permissions": ["epsx:api:access", "epsx:analytics:view:unlimited", "epsx:rankings:view:unlimited"],
        "features": [
            "Full REST API access",
            "Unlimited analytics queries",
            "Unlimited rankings access",
            "WebSocket support",
            "API documentation",
            "Developer support",
            "100k API calls/month"
        ],
        "limits": {
            "api_calls_per_month": 100000,
            "websocket_connections": 5,
            "rate_limit_per_second": 10
        },
        "plan_type": "api"
    }'::jsonb,
    49.99,
    'USD',
    'monthly',
    true,
    false,
    5,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;