import type { PackagePermission, PermissionTemplate } from '../types/admin/iam-enhanced';
import { PackageTier } from '../types/admin/iam-enhanced';

export const DEFAULT_PACKAGE_PERMISSIONS: Record<PackageTier, PackagePermission[]> = {
  [PackageTier.FREE]: [
    {
      id: 'free_dashboard_basic',
      packageTier: PackageTier.FREE,
      featureId: 'dashboard_basic',
      permission: { action: 'view', resource: 'dashboard:basic' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'free_profile_basic',
      packageTier: PackageTier.FREE,
      featureId: 'profile_basic',
      permission: { action: 'view', resource: 'profile' },
      isDefault: true,
      autoGranted: true,
    },
  ],

  [PackageTier.BRONZE]: [
    // Inherit FREE permissions
    ...[] as PackagePermission[], // Will be populated by inheritance logic
    {
      id: 'bronze_api_personal',
      packageTier: PackageTier.BRONZE,
      featureId: 'api_personal',
      permission: { 
        action: 'execute', 
        resource: 'api:personal',
        conditions: [{ type: 'usage_limit', value: 1000, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'bronze_support_basic',
      packageTier: PackageTier.BRONZE,
      featureId: 'support_basic',
      permission: { action: 'create', resource: 'support:ticket' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'bronze_rankings_limited',
      packageTier: PackageTier.BRONZE,
      featureId: 'rankings_limited',
      permission: { 
        action: 'view', 
        resource: 'rankings:stock',
        conditions: [{ type: 'usage_limit', value: 5, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
  ],

  [PackageTier.SILVER]: [
    // Inherit BRONZE permissions
    ...[] as PackagePermission[], // Will be populated by inheritance logic
    {
      id: 'silver_api_company',
      packageTier: PackageTier.SILVER,
      featureId: 'api_company',
      permission: { 
        action: 'execute', 
        resource: 'api:company',
        conditions: [{ type: 'usage_limit', value: 10000, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'silver_analytics_advanced',
      packageTier: PackageTier.SILVER,
      featureId: 'analytics_advanced',
      permission: { action: 'view', resource: 'analytics:advanced' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'silver_rankings_enhanced',
      packageTier: PackageTier.SILVER,
      featureId: 'rankings_enhanced',
      permission: { 
        action: 'view', 
        resource: 'rankings:stock',
        conditions: [{ type: 'usage_limit', value: 25, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
  ],

  [PackageTier.GOLD]: [
    // Inherit SILVER permissions
    ...[] as PackagePermission[], // Will be populated by inheritance logic
    {
      id: 'gold_api_partner',
      packageTier: PackageTier.GOLD,
      featureId: 'api_partner',
      permission: { 
        action: 'execute', 
        resource: 'api:partner',
        conditions: [{ type: 'usage_limit', value: 50000, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'gold_data_export',
      packageTier: PackageTier.GOLD,
      featureId: 'data_export',
      permission: { action: 'execute', resource: 'data:export' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'gold_priority_support',
      packageTier: PackageTier.GOLD,
      featureId: 'support_priority',
      permission: { action: 'create', resource: 'support:priority_ticket' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'gold_rankings_premium',
      packageTier: PackageTier.GOLD,
      featureId: 'rankings_premium',
      permission: { 
        action: 'view', 
        resource: 'rankings:stock',
        conditions: [{ type: 'usage_limit', value: 50, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
  ],

  [PackageTier.PLATINUM]: [
    // Inherit GOLD permissions
    ...[] as PackagePermission[], // Will be populated by inheritance logic
    {
      id: 'platinum_api_unlimited',
      packageTier: PackageTier.PLATINUM,
      featureId: 'api_unlimited',
      permission: { action: 'execute', resource: 'api:*' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'platinum_white_label',
      packageTier: PackageTier.PLATINUM,
      featureId: 'white_label',
      permission: { action: 'manage', resource: 'branding' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'platinum_rankings_unlimited',
      packageTier: PackageTier.PLATINUM,
      featureId: 'rankings_unlimited',
      permission: { action: 'view', resource: 'rankings:*' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'platinum_dedicated_support',
      packageTier: PackageTier.PLATINUM,
      featureId: 'dedicated_support',
      permission: { action: '*', resource: 'support:dedicated' },
      isDefault: true,
      autoGranted: true,
    },
  ],

  [PackageTier.ENTERPRISE]: [
    // Inherit PLATINUM permissions
    ...[] as PackagePermission[], // Will be populated by inheritance logic
    {
      id: 'enterprise_custom_integration',
      packageTier: PackageTier.ENTERPRISE,
      featureId: 'custom_integration',
      permission: { action: '*', resource: 'integration:*' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'enterprise_full_analytics',
      packageTier: PackageTier.ENTERPRISE,
      featureId: 'full_analytics',
      permission: { action: '*', resource: 'analytics:*' },
      isDefault: true,
      autoGranted: true,
    },
    {
      id: 'enterprise_custom_branding',
      packageTier: PackageTier.ENTERPRISE,
      featureId: 'custom_branding',
      permission: { action: '*', resource: 'branding:*' },
      isDefault: true,
      autoGranted: true,
    },
  ],
};

// Custom permission templates for admins to quickly apply
export const PERMISSION_TEMPLATES: PermissionTemplate[] = [
  {
    id: 'api_access_boost',
    name: 'API Access Boost',
    description: 'Increase API limits by 50%',
    category: 'API',
    isSystem: false,
    permissions: [
      { action: 'execute', resource: 'api:boost', conditions: [{ type: 'usage_limit', value: 1.5, operator: 'eq' }] }
    ],
  },
  {
    id: 'beta_features',
    name: 'Beta Features Access',
    description: 'Access to experimental features',
    category: 'Features',
    isSystem: false,
    permissions: [
      { action: '*', resource: 'features:beta' }
    ],
  },
  {
    id: 'data_retention_extended',
    name: 'Extended Data Retention',
    description: 'Keep data for 2 years instead of 1',
    category: 'Data',
    isSystem: false,
    permissions: [
      { action: 'view', resource: 'data:extended_retention' }
    ],
  },
  {
    id: 'developer_tools',
    name: 'Developer Tools Access',
    description: 'Access to debugging and development tools',
    category: 'Development',
    isSystem: false,
    permissions: [
      { action: 'view', resource: 'tools:debug' },
      { action: 'execute', resource: 'tools:test' },
      { action: 'view', resource: 'logs:system' }
    ],
  },
  {
    id: 'premium_support',
    name: 'Premium Support Package',
    description: 'Priority support with SLA guarantees',
    category: 'Support',
    isSystem: false,
    permissions: [
      { action: 'create', resource: 'support:premium_ticket' },
      { action: 'view', resource: 'support:sla_metrics' },
      { action: 'schedule', resource: 'support:call' }
    ],
  },
  {
    id: 'analytics_enterprise',
    name: 'Enterprise Analytics',
    description: 'Advanced analytics and reporting features',
    category: 'Analytics',
    isSystem: false,
    permissions: [
      { action: 'view', resource: 'analytics:custom_reports' },
      { action: 'create', resource: 'analytics:dashboard' },
      { action: 'export', resource: 'analytics:raw_data' }
    ],
  },
];

// Function to build complete package permissions with inheritance
export function buildPackagePermissions(): Record<PackageTier, PackagePermission[]> {
  const result: Record<PackageTier, PackagePermission[]> = {
    [PackageTier.FREE]: DEFAULT_PACKAGE_PERMISSIONS[PackageTier.FREE],
    [PackageTier.BRONZE]: [],
    [PackageTier.SILVER]: [],
    [PackageTier.GOLD]: [],
    [PackageTier.PLATINUM]: [],
    [PackageTier.ENTERPRISE]: [],
  };

  // Build Bronze (FREE + Bronze specific)
  result[PackageTier.BRONZE] = [
    ...result[PackageTier.FREE],
    ...DEFAULT_PACKAGE_PERMISSIONS[PackageTier.BRONZE].filter(p => p.packageTier === PackageTier.BRONZE)
  ];

  // Build Silver (Bronze + Silver specific)
  result[PackageTier.SILVER] = [
    ...result[PackageTier.BRONZE],
    ...DEFAULT_PACKAGE_PERMISSIONS[PackageTier.SILVER].filter(p => p.packageTier === PackageTier.SILVER)
  ];

  // Build Gold (Silver + Gold specific)
  result[PackageTier.GOLD] = [
    ...result[PackageTier.SILVER],
    ...DEFAULT_PACKAGE_PERMISSIONS[PackageTier.GOLD].filter(p => p.packageTier === PackageTier.GOLD)
  ];

  // Build Platinum (Gold + Platinum specific)
  result[PackageTier.PLATINUM] = [
    ...result[PackageTier.GOLD],
    ...DEFAULT_PACKAGE_PERMISSIONS[PackageTier.PLATINUM].filter(p => p.packageTier === PackageTier.PLATINUM)
  ];

  // Build Enterprise (Platinum + Enterprise specific)
  result[PackageTier.ENTERPRISE] = [
    ...result[PackageTier.PLATINUM],
    ...DEFAULT_PACKAGE_PERMISSIONS[PackageTier.ENTERPRISE].filter(p => p.packageTier === PackageTier.ENTERPRISE)
  ];

  return result;
}

// Feature definitions for EPSX
export const EPSX_FEATURES = {
  // Customer-facing features
  DASHBOARD_BASIC: {
    id: 'dashboard_basic',
    name: 'Basic Dashboard',
    description: 'View basic dashboard metrics',
    requiredTier: PackageTier.FREE,
  },
  RANKINGS_LIMITED: {
    id: 'rankings_limited',
    name: 'Limited Stock Rankings',
    description: 'View up to 5 stock rankings',
    requiredTier: PackageTier.BRONZE,
  },
  RANKINGS_ENHANCED: {
    id: 'rankings_enhanced',
    name: 'Enhanced Stock Rankings',
    description: 'View up to 25 stock rankings',
    requiredTier: PackageTier.SILVER,
  },
  RANKINGS_PREMIUM: {
    id: 'rankings_premium',
    name: 'Premium Stock Rankings',
    description: 'View up to 50 stock rankings',
    requiredTier: PackageTier.GOLD,
  },
  RANKINGS_UNLIMITED: {
    id: 'rankings_unlimited',
    name: 'Unlimited Stock Rankings',
    description: 'Unlimited stock rankings access',
    requiredTier: PackageTier.PLATINUM,
  },
  API_PERSONAL: {
    id: 'api_personal',
    name: 'Personal API Access',
    description: 'API access for personal use (1k calls/month)',
    requiredTier: PackageTier.BRONZE,
  },
  API_COMPANY: {
    id: 'api_company',
    name: 'Company API Access',
    description: 'API access for company use (10k calls/month)',
    requiredTier: PackageTier.SILVER,
  },
  API_PARTNER: {
    id: 'api_partner',
    name: 'Partner API Access',
    description: 'API access for partners (50k calls/month)',
    requiredTier: PackageTier.GOLD,
  },
  API_UNLIMITED: {
    id: 'api_unlimited',
    name: 'Unlimited API Access',
    description: 'Unlimited API access',
    requiredTier: PackageTier.PLATINUM,
  },
};
