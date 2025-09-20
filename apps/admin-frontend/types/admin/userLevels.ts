export type PermissionTemplateName = 
  | 'Free Template'
  | 'Bronze Template' 
  | 'Silver Template'
  | 'Gold Template'
  | 'Platinum Template'
  | 'Enterprise Template'
  | 'Admin Template';

export interface PermissionTemplateConfig {
  templateName: PermissionTemplateName;
  displayTier: string;
  name: string;
  description: string;
  color: string;
  permissions: string[];
  features: string[];
  tokenMultiplier: number;
  maxTokens: number;
  priority: number;
}

export const PERMISSION_TEMPLATE_CONFIGS: Record<PermissionTemplateName, PermissionTemplateConfig> = {
  'Free Template': {
    templateName: 'Free Template',
    displayTier: 'FREE',
    name: 'Free',
    description: 'Basic free tier with essential features',
    color: 'bg-gray-100 text-gray-800',
    permissions: ['epsx:rankings:view:3', 'epsx:trading:basic', 'epsx:portfolio:view'],
    features: ['View 3 rankings', 'Basic trading', 'Portfolio view'],
    tokenMultiplier: 1,
    maxTokens: 500,
    priority: 0
  },
  'Bronze Template': {
    templateName: 'Bronze Template',
    displayTier: 'BRONZE',
    name: 'Bronze',
    description: 'Enhanced access with basic features',
    color: 'bg-amber-100 text-amber-800',
    permissions: ['epsx:rankings:view:5', 'epsx:trading:basic', 'epsx:portfolio:view', 'epsx:portfolio:history'],
    features: ['View 5 rankings', 'Basic trading', 'Portfolio history'],
    tokenMultiplier: 1.2,
    maxTokens: 1000,
    priority: 1
  },
  'Silver Template': {
    templateName: 'Silver Template',
    displayTier: 'SILVER',
    name: 'Silver',
    description: 'Premium access with advanced analytics',
    color: 'bg-gray-100 text-gray-800',
    permissions: ['epsx:rankings:view:25', 'epsx:trading:basic', 'epsx:trading:advanced', 'epsx:portfolio:view', 'epsx:analytics:basic'],
    features: ['View 25 rankings', 'Advanced trading', 'Basic analytics'],
    tokenMultiplier: 1.5,
    maxTokens: 2500,
    priority: 2
  },
  'Gold Template': {
    templateName: 'Gold Template',
    displayTier: 'GOLD',
    name: 'Gold',
    description: 'Professional access with premium tools',
    color: 'bg-yellow-100 text-yellow-800',
    permissions: ['epsx:rankings:view:50', 'epsx:trading:premium', 'epsx:portfolio:tools', 'epsx:analytics:advanced'],
    features: ['View 50 rankings', 'Premium trading', 'Advanced analytics'],
    tokenMultiplier: 2,
    maxTokens: 5000,
    priority: 3
  },
  'Platinum Template': {
    templateName: 'Platinum Template',
    displayTier: 'PLATINUM',
    name: 'Platinum',
    description: 'VIP access with advanced features',
    color: 'bg-purple-100 text-purple-800',
    permissions: ['epsx:rankings:view:100', 'epsx:trading:premium', 'epsx:analytics:premium', 'epsx:research:reports', 'epsx:dashboards:custom'],
    features: ['View 100 rankings', 'Premium trading', 'Custom dashboards'],
    tokenMultiplier: 3,
    maxTokens: 10000,
    priority: 4
  },
  'Enterprise Template': {
    templateName: 'Enterprise Template',
    displayTier: 'ENTERPRISE',
    name: 'Enterprise',
    description: 'Unlimited access with all platform features',
    color: 'bg-blue-100 text-blue-800',
    permissions: ['epsx:rankings:view:unlimited', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*'],
    features: ['Unlimited rankings', 'All platform features', 'API access'],
    tokenMultiplier: 5,
    maxTokens: 25000,
    priority: 5
  },
  'Admin Template': {
    templateName: 'Admin Template',
    displayTier: 'ADMIN',
    name: 'Admin',
    description: 'Full administrative access',
    color: 'bg-red-100 text-red-800',
    permissions: ['admin:*:*', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*'],
    features: ['Full admin access', 'User management', 'System configuration'],
    tokenMultiplier: 10,
    maxTokens: -1, // Unlimited
    priority: 6
  }
};

export interface PermissionTemplateAssignment {
  uid: string;
  templateName: PermissionTemplateName;
  permissions: string[];
  assignedBy: string;
  assignedAt: Date;
  reason?: string;
  previousTemplate?: PermissionTemplateName;
}
