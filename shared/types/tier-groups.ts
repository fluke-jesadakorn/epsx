/**
 * UNIFIED TIER GROUP SYSTEM TYPES
 * 
 * This file defines the unified tier group system that replaces static package tiers
 * with dynamic, admin-controllable tier groups using the same access control
 * mechanism as permission groups.
 * 
 * Key Features:
 * - Same permission format as existing group system
 * - Admin-controllable pricing and features
 * - Unified access control via hasPermission()
 * - Auto-assignment on payment
 */

// ============================================================================
// CORE TIER GROUP TYPES
// ============================================================================

export interface TierGroup {
  id: string;
  name: string;                    // "Professional Analytics Plan"
  slug: string;                    // "professional-plan"
  tierDisplay: string;            // "PRO" (for UI compatibility)
  description: string;
  
  // Pricing (Admin Controllable)
  price: number;                  // 29.99
  currency: string;               // "USD"
  billingCycle: 'monthly' | 'yearly';
  
  // Permission Integration (Same as Group System)
  permissions: string[];          // ["epsx:analytics:view:50", "epsx:exports:unlimited"]
  features: string[];             // ["50 stock limit", "Real-time data", "Unlimited exports"]
  
  // Admin Control
  isActive: boolean;              // Admin can enable/disable
  isPromoted: boolean;            // Featured/highlighted tier
  displayOrder: number;           // Sort order in UI
  
  // Auto-Assignment Rules
  autoAssignOnPayment: boolean;   // Auto-assign users who pay for this tier
  maxMembers?: number;            // Optional member limit
  
  // Metadata
  metadata: {
    category: 'personal' | 'business' | 'enterprise';
    target: string[];             // ["traders", "analysts", "institutions"]
    promotions?: string[];        // Active promotional campaigns
    badges?: string[];           // ["Popular", "Best Value"]
  };
  
  // Audit Trail
  createdAt: string;
  updatedAt: string;
  createdBy: string;             // Admin who created this tier
  lastModifiedBy: string;        // Admin who last modified
  
  // Analytics
  subscriberCount: number;       // Current active subscribers
  revenue30Days: number;         // Revenue from this tier in last 30 days
  conversionRate: number;        // Conversion rate percentage
}

export interface TierGroupRequest {
  name: string;
  slug: string;
  tierDisplay: string;
  description: string;
  price: number;
  currency: string;
  billingCycle: 'monthly' | 'yearly';
  permissions: string[];
  features: string[];
  isActive: boolean;
  isPromoted?: boolean;
  displayOrder?: number;
  autoAssignOnPayment?: boolean;
  maxMembers?: number;
  metadata?: {
    category: 'personal' | 'business' | 'enterprise';
    target?: string[];
    promotions?: string[];
    badges?: string[];
  };
}

export interface UpdateTierGroupRequest {
  name?: string;
  description?: string;
  price?: number;
  currency?: string;
  permissions?: string[];
  features?: string[];
  isActive?: boolean;
  isPromoted?: boolean;
  displayOrder?: number;
  autoAssignOnPayment?: boolean;
  maxMembers?: number;
  metadata?: Partial<TierGroup['metadata']>;
}

// ============================================================================
// USER TIER ASSIGNMENT TYPES
// ============================================================================

export interface UserTierAssignment {
  id: string;
  userId: string;
  tierGroupId: string;
  
  // Assignment Details
  assignedAt: string;
  expiresAt?: string;             // For time-limited subscriptions
  isActive: boolean;
  
  // Assignment Source
  assignmentSource: 'payment' | 'admin' | 'promotion' | 'migration';
  assignmentReason: string;
  assignedBy?: string;            // Admin who assigned (if manual)
  
  // Payment Integration
  paymentReference?: string;      // Link to payment record
  subscriptionId?: string;        // Subscription management ID
  
  // Status Tracking
  status: 'active' | 'expired' | 'cancelled' | 'suspended';
  autoRenew: boolean;
  nextBillingDate?: string;
  
  // Metadata
  metadata?: {
    originalPrice?: number;       // Price when subscription started
    promotionCode?: string;       // If assigned via promotion
    upgradeHistory?: string[];    // Previous tier IDs
  };
}

export interface CreateTierAssignmentRequest {
  userId: string;
  tierGroupId: string;
  assignmentSource: 'payment' | 'admin' | 'promotion' | 'migration';
  assignmentReason: string;
  expiresAt?: string;
  autoRenew?: boolean;
  paymentReference?: string;
  subscriptionId?: string;
  metadata?: UserTierAssignment['metadata'];
}

export interface UpdateTierAssignmentRequest {
  expiresAt?: string;
  isActive?: boolean;
  status?: UserTierAssignment['status'];
  autoRenew?: boolean;
  assignmentReason?: string;
  metadata?: Partial<UserTierAssignment['metadata']>;
}

// ============================================================================
// TIER MIGRATION TYPES
// ============================================================================

export interface TierMigrationPlan {
  id: string;
  name: string;
  description: string;
  
  // Migration Configuration
  sourceTierGroups: string[];     // IDs of tiers to migrate from
  targetTierGroup: string;        // ID of tier to migrate to
  
  // Migration Rules
  preserveExpiry: boolean;        // Keep original expiry dates
  notifyUsers: boolean;          // Send notification to affected users
  requireConfirmation: boolean;   // Require user confirmation
  
  // Scheduling
  scheduledAt?: string;          // When to execute migration
  executeImmediately: boolean;   // Execute immediately if true
  
  // Status
  status: 'planned' | 'in_progress' | 'completed' | 'failed' | 'cancelled';
  affectedUsers: number;         // Number of users to be migrated
  completedUsers: number;        // Number of users already migrated
  
  // Metadata
  createdAt: string;
  createdBy: string;
  executedAt?: string;
  completedAt?: string;
  errors?: string[];
}

// ============================================================================
// ANALYTICS AND REPORTING TYPES
// ============================================================================

export interface TierGroupAnalytics {
  tierGroupId: string;
  period: 'daily' | 'weekly' | 'monthly';
  startDate: string;
  endDate: string;
  
  // Subscription Metrics
  newSubscribers: number;
  cancelledSubscribers: number;
  netSubscribers: number;
  churnRate: number;
  
  // Revenue Metrics
  totalRevenue: number;
  averageRevenuePerUser: number;
  lifetimeValue: number;
  
  // Usage Metrics
  averagePermissionUsage: Record<string, number>;
  featureUtilization: Record<string, number>;
  
  // Comparison Metrics
  conversionFromLowerTiers: number;
  upgradeToHigherTiers: number;
  downgrades: number;
}

export interface TierComparisonReport {
  comparisonPeriod: string;
  tiers: Array<{
    tierGroup: TierGroup;
    metrics: TierGroupAnalytics;
    performance: {
      revenueGrowth: number;
      subscriberGrowth: number;
      retentionRate: number;
      satisfactionScore: number;
    };
  }>;
  
  insights: {
    bestPerforming: string;       // Tier ID
    mostProfitable: string;       // Tier ID
    highestChurn: string;         // Tier ID
    recommendations: string[];    // AI-generated recommendations
  };
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

export interface TierGroupListResponse {
  tierGroups: TierGroup[];
  total: number;
  pagination: {
    page: number;
    limit: number;
    totalPages: number;
  };
}

export interface TierAssignmentListResponse {
  assignments: UserTierAssignment[];
  total: number;
  pagination: {
    page: number;
    limit: number;
    totalPages: number;
  };
}

export interface TierGroupResponse {
  success: boolean;
  data?: TierGroup;
  error?: string;
  message?: string;
}

export interface TierAssignmentResponse {
  success: boolean;
  data?: UserTierAssignment;
  error?: string;
  message?: string;
}

// ============================================================================
// UNIFIED PERMISSION RESOLUTION TYPES
// ============================================================================

export interface UnifiedUserPermissions {
  userId: string;
  
  // All permission sources unified
  effectivePermissions: string[];  // Final resolved permissions array
  
  // Permission Sources
  sources: {
    tierGroups: Array<{
      tierGroupId: string;
      tierGroupName: string;
      permissions: string[];
      expiresAt?: string;
    }>;
    directGroups: Array<{
      groupId: string;
      groupName: string;
      permissions: string[];
      expiresAt?: string;
    }>;
    directPermissions: string[];    // Legacy direct permissions (if any)
  };
  
  // Expiry Tracking
  expiringPermissions: Array<{
    permission: string;
    source: string;
    expiresAt: string;
    daysUntilExpiry: number;
  }>;
  
  // Cache Information
  resolvedAt: string;
  cacheExpiresAt: string;
  version: number;
}

// ============================================================================
// LEGACY COMPATIBILITY TYPES
// ============================================================================

export interface LegacyTierMappings {
  // Maps old package tier strings to new tier group IDs
  FREE: string;
  BASIC: string;
  PRO: string;
  PREMIUM: string;
  ENTERPRISE: string;
}

export interface TierMigrationResult {
  success: boolean;
  migratedUsers: number;
  skippedUsers: number;
  errors: Array<{
    userId: string;
    error: string;
    legacyTier: string;
  }>;
  mapping: LegacyTierMappings;
}

// ============================================================================
// TYPE GUARDS AND UTILITIES
// ============================================================================

export function isTierGroup(obj: any): obj is TierGroup {
  return obj && 
    typeof obj.id === 'string' &&
    typeof obj.name === 'string' &&
    typeof obj.tierDisplay === 'string' &&
    Array.isArray(obj.permissions) &&
    Array.isArray(obj.features) &&
    typeof obj.price === 'number';
}

export function isTierAssignment(obj: any): obj is UserTierAssignment {
  return obj &&
    typeof obj.id === 'string' &&
    typeof obj.userId === 'string' &&
    typeof obj.tierGroupId === 'string' &&
    typeof obj.assignedAt === 'string';
}

export function isActiveTierAssignment(assignment: UserTierAssignment): boolean {
  return assignment.isActive && 
    assignment.status === 'active' && 
    (!assignment.expiresAt || new Date(assignment.expiresAt) > new Date());
}

export function getTierDisplayName(tierGroup: TierGroup): string {
  return tierGroup.tierDisplay || tierGroup.name;
}

export function getTierPrice(tierGroup: TierGroup): string {
  return `$${tierGroup.price.toFixed(2)}/${tierGroup.billingCycle}`;
}

// ============================================================================
// CONSTANTS
// ============================================================================

export const DEFAULT_TIER_GROUPS = {
  FREE: 'free-tier-group',
  BASIC: 'basic-tier-group',
  PRO: 'pro-tier-group',
  ENTERPRISE: 'enterprise-tier-group'
} as const;

export const TIER_CATEGORIES = {
  PERSONAL: 'personal',
  BUSINESS: 'business',
  ENTERPRISE: 'enterprise'
} as const;

export const ASSIGNMENT_SOURCES = {
  PAYMENT: 'payment',
  ADMIN: 'admin',
  PROMOTION: 'promotion',
  MIGRATION: 'migration'
} as const;

export const ASSIGNMENT_STATUS = {
  ACTIVE: 'active',
  EXPIRED: 'expired',
  CANCELLED: 'cancelled',
  SUSPENDED: 'suspended'
} as const;