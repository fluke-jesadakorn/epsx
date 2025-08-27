// ============================================================================
// SIMPLE ROLE CONFIGURATION - MATCHES BACKEND EXACTLY
// ============================================================================
// This file replaces complex IAM system with simple role-based configuration
// Roles: admin, user, guest
// Features: view_eps, export_data, realtime, profile, notifications, billing, advanced_filters

import { Role, checkFeatureAccess } from '../../types/permissions';

// ============================================================================
// SIMPLE ROLE DEFINITIONS
// ============================================================================

export interface SimpleRole {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
  features: string[];
  isSystem: boolean;
  isActive: boolean;
}

// The 7 core features that align with backend
export const AVAILABLE_FEATURES = [
  'view_eps',
  'export_data', 
  'realtime',
  'profile',
  'notifications',
  'billing',
  'advanced_filters'
] as const;

export const SIMPLE_ROLES: SimpleRole[] = [
  {
    id: 'admin',
    name: 'Admin',
    description: 'Full system access with all features',
    color: '#dc2626',
    icon: 'Shield',
    features: [...AVAILABLE_FEATURES], // All features
    isSystem: true,
    isActive: true
  },
  {
    id: 'user', 
    name: 'User',
    description: 'Premium user with access to all features',
    color: '#16a34a',
    icon: 'User',
    features: [...AVAILABLE_FEATURES], // All features (user = premium)
    isSystem: true,
    isActive: true
  },
  {
    id: 'guest',
    name: 'Guest',
    description: 'Basic access to view EPS data only',
    color: '#6b7280',
    icon: 'Eye',
    features: ['view_eps'], // Only basic viewing
    isSystem: true,
    isActive: true
  }
];

// ============================================================================
// ROLE UTILITIES
// ============================================================================

export const getRoleById = (roleId: string): SimpleRole | undefined => {
  return SIMPLE_ROLES.find(role => role.id === roleId);
};

export const getRoleByName = (roleName: string): SimpleRole | undefined => {
  return SIMPLE_ROLES.find(role => role.name.toLowerCase() === roleName.toLowerCase());
};

export const getUserFeatures = (role: Role): string[] => {
  return AVAILABLE_FEATURES.filter(feature => checkFeatureAccess(role, feature));
};

// Import helper from types/permissions to avoid circular dependency
const { checkFeatureAccess: checkFeature } = require('../../types/permissions');

export const getUserFeaturesById = (roleId: string): string[] => {
  const { roleFromString } = require('../../types/permissions');
  try {
    const role = roleFromString(roleId);
    return AVAILABLE_FEATURES.filter(feature => checkFeature(role, feature));
  } catch (error) {
    console.warn(`Invalid role ID: ${roleId}, defaulting to guest features`);
    return ['view_eps'];
  }
};

export const hasFeatureAccess = (role: Role, feature: string): boolean => {
  return checkFeatureAccess(role, feature);
};

// ============================================================================
// ROLE VALIDATION
// ============================================================================

export const isValidRole = (roleId: string): boolean => {
  return SIMPLE_ROLES.some(role => role.id === roleId);
};

export const getActiveRoles = (): SimpleRole[] => {
  return SIMPLE_ROLES.filter(role => role.isActive);
};

export const getSystemRoles = (): SimpleRole[] => {
  return SIMPLE_ROLES.filter(role => role.isSystem);
};

// ============================================================================
// BACKWARD COMPATIBILITY
// ============================================================================

// Legacy exports for components that might still reference them
export const DEFAULT_ROLES = SIMPLE_ROLES;
export const DEFAULT_PERMISSIONS = AVAILABLE_FEATURES.map(feature => ({
  id: feature,
  name: feature.charAt(0).toUpperCase() + feature.slice(1),
  description: `Access to ${feature} functionality`,
  category: 'feature',
  isSystem: true
}));

// Legacy role mapping for old code
export const ROLE_MAPPINGS = {
  'admin': Role.Admin,
  'user': Role.User,
  'premium': Role.User, // Map premium to user (they're the same now)
  'basic': Role.Guest,  // Map basic to guest
  'guest': Role.Guest,
  'viewer': Role.Guest, // Map viewer to guest
} as const;

export const mapLegacyRole = (legacyRole: string): Role => {
  const mapped = ROLE_MAPPINGS[legacyRole.toLowerCase() as keyof typeof ROLE_MAPPINGS];
  return mapped || Role.Guest; // Default to guest if unknown
};

// ============================================================================
// FEATURE DESCRIPTIONS
// ============================================================================

export const FEATURE_DESCRIPTIONS = {
  view_eps: 'View EPS growth analytics and stock rankings',
  export_data: 'Export analytics data to various formats (CSV, Excel, PDF)',
  realtime: 'Access real-time market data and live updates',
  profile: 'Manage user profile and account settings',
  notifications: 'Receive and manage system notifications',
  billing: 'Access billing information and payment management',
  advanced_filters: 'Use advanced filtering and search capabilities'
} as const;

export const getFeatureDescription = (feature: string): string => {
  return FEATURE_DESCRIPTIONS[feature as keyof typeof FEATURE_DESCRIPTIONS] || 'Unknown feature';
};

// ============================================================================
// ROLE MATRIX (FOR DOCUMENTATION AND UI)
// ============================================================================

export const ROLE_FEATURE_MATRIX = SIMPLE_ROLES.map(role => ({
  role: role.id,
  name: role.name,
  color: role.color,
  icon: role.icon,
  features: role.features.reduce((acc, feature) => ({
    ...acc,
    [feature]: true
  }), {} as Record<string, boolean>)
}));

// ============================================================================
// ROLE COMPARISON UTILITIES
// ============================================================================

export const compareRoles = (role1: Role, role2: Role): number => {
  const hierarchy = { [Role.Admin]: 3, [Role.User]: 2, [Role.Guest]: 1 };
  return hierarchy[role1] - hierarchy[role2];
};

export const isRoleHigherThan = (role1: Role, role2: Role): boolean => {
  return compareRoles(role1, role2) > 0;
};

export const getHighestRole = (roles: Role[]): Role => {
  return roles.reduce((highest, current) => 
    isRoleHigherThan(current, highest) ? current : highest, Role.Guest);
};