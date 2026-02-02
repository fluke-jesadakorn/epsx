export type PermissionTemplateName = 'Free Template' | 'Pro Template' | 'Enterprise Template';

export interface PermissionTemplateConfig {
    name: string;
    description: string;
    permissions: string[];
    features: string[];
}

export const PERMISSION_TEMPLATE_CONFIGS: Record<PermissionTemplateName, PermissionTemplateConfig> = {
    'Free Template': {
        name: 'Free',
        description: 'Basic access for all users',
        permissions: ['epsx:analytics:view:25'],
        features: ['Basic Analytics', 'Public Collections']
    },
    'Pro Template': {
        name: 'Pro',
        description: 'Advanced features for power users',
        permissions: ['epsx:analytics:view', 'epsx:analytics:export'],
        features: ['Advanced Analytics', 'Export Data', 'Priority Support']
    },
    'Enterprise Template': {
        name: 'Enterprise',
        description: 'Full access for organizations',
        permissions: ['epsx:analytics:view', 'epsx:analytics:export', 'admin:users:view'],
        features: ['Unlimited Data', 'API Access', 'Dedicated Account Manager']
    }
};
