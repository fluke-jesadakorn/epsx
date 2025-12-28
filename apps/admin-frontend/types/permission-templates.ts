/**
 * Permission Template Types
 * 
 * NOTE: This file was recreated as a stub after the original was deleted.
 * It provides minimal definitions needed for the admin plans page.
 */

export type PermissionTemplateName =
    | 'Free Template'
    | 'Starter Template'
    | 'Professional Template'
    | 'Enterprise Template'
    | 'API Developer Template'

interface PermissionTemplate {
    name: string
    description: string
    permissions: string[]
    features: string[]
}

export const PERMISSION_TEMPLATE_CONFIGS: Record<PermissionTemplateName, PermissionTemplate> = {
    'Free Template': {
        name: 'Free',
        description: 'Basic access for all users',
        permissions: [
            'epsx:rankings:view:10',
            'epsx:public:read'
        ],
        features: [
            'View top 10 rankings',
            'Public data access'
        ]
    },
    'Starter Template': {
        name: 'Starter',
        description: 'Entry-level paid tier',
        permissions: [
            'epsx:rankings:view:50',
            'epsx:analytics:basic',
            'epsx:export:csv'
        ],
        features: [
            'View top 50 rankings',
            'Basic analytics',
            'CSV export'
        ]
    },
    'Professional Template': {
        name: 'Professional',
        description: 'Full access for professionals',
        permissions: [
            'epsx:rankings:view:unlimited',
            'epsx:analytics:advanced',
            'epsx:export:all',
            'epsx:alerts:create'
        ],
        features: [
            'Unlimited rankings',
            'Advanced analytics',
            'All export formats',
            'Price alerts'
        ]
    },
    'Enterprise Template': {
        name: 'Enterprise',
        description: 'Custom enterprise solutions',
        permissions: [
            'epsx:rankings:view:unlimited',
            'epsx:analytics:advanced',
            'epsx:export:all',
            'epsx:alerts:create',
            'epsx:api:advanced',
            'epsx:support:priority'
        ],
        features: [
            'Everything in Professional',
            'Advanced API access',
            'Priority support',
            'Custom integrations'
        ]
    },
    'API Developer Template': {
        name: 'API Developer',
        description: 'Full API access for developers',
        permissions: [
            'epsx:api:read',
            'epsx:api:write',
            'epsx:api:advanced',
            'epsx:export:all'
        ],
        features: [
            'Full API access',
            'Read/Write operations',
            'All export formats',
            'Webhook support'
        ]
    }
}
