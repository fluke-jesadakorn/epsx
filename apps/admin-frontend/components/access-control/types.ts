/**
 * Unified Access Control Types
 * Combines Plans and Groups into a single AccessPolicy abstraction
 */

import type { PermissionPlan as PermissionGroup } from '@/lib/api/plan-management-client';
import type { Plan as SharedPlan } from '@/shared/api/plans';

// ============================================================================
// POLICY TYPES
// ============================================================================

/** Policy type categories - maps to group_type + subscription */
export type PolicyType =
  | 'subscription'
  | 'manual'
  | 'web3_asset'
  | 'dao'
  | 'system';

/** Policy source - where the data originates */
export type PolicySource = 'plan' | 'group';

/** Policy status */
export type PolicyStatus = 'active' | 'inactive';

// ============================================================================
// ACCESS POLICY - UNIFIED TYPE
// ============================================================================

/**
 * Unified AccessPolicy interface
 * Wraps both PlanResponse and PermissionGroup into a single type
 */
export interface AccessPolicy {
  /** Unique identifier (prefixed with source type for uniqueness) */
  id: string;

  /** Display name */
  name: string;

  /** Description */
  description: string;

  /** Policy type category */
  type: PolicyType;

  /** List of permission strings */
  permissions: string[];

  /** Number of users/wallets with this policy */
  memberCount: number;

  /** Whether the policy is currently active */
  isActive: boolean;

  // -------------------------------------------------------------------------
  // Subscription-specific fields (type === 'subscription')
  // -------------------------------------------------------------------------

  /** Pricing information for subscription plans */
  pricing?: {
    amount: number;
    currency: string;
    cycle: string;
  };

  /** Revenue in last 30 days (subscription only) */
  revenue?: number;

  /** Tier level for plan hierarchy (subscription only) */
  tierLevel?: number;

  /** Plan category (standard, api, enterprise) */
  planCategory?: string;

  // -------------------------------------------------------------------------
  // Group-specific fields (type !== 'subscription')
  // -------------------------------------------------------------------------

  /** Default expiry days for assignments */
  expiryDays?: number;

  /** Priority level for permission resolution */
  priorityLevel?: number;

  /** Whether this is a protected system group */
  isSystemGroup?: boolean;

  /** Group slug for URL-friendly identification */
  slug?: string;

  // -------------------------------------------------------------------------
  // Common metadata
  // -------------------------------------------------------------------------

  /** Creation timestamp */
  createdAt: string;

  /** Last update timestamp */
  updatedAt: string;

  /** Original ID from source (plan.id or group.id) */
  sourceId: string;

  /** Source type for API routing */
  sourceType: PolicySource;
}

// ============================================================================
// FILTER TYPES
// ============================================================================

/** Filter options for policy list */
export interface PolicyFilters {
  /** Text search across name/description */
  search: string;

  /** Filter by policy type(s) */
  types: PolicyType[] | 'all';

  /** Filter by status */
  status: 'all' | 'active' | 'inactive';

  /** Sort field */
  sortBy: 'name' | 'members' | 'created_at' | 'revenue' | 'type';

  /** Sort direction */
  sortOrder: 'asc' | 'desc';
}

/** Default filter values */
export const DEFAULT_POLICY_FILTERS: PolicyFilters = {
  search: '',
  types: 'all',
  status: 'all',
  sortBy: 'name',
  sortOrder: 'asc',
};

// ============================================================================
// STATS TYPES
// ============================================================================

/** Aggregated stats for the dashboard */
export interface PolicyStats {
  /** Total number of policies */
  totalPolicies: number;

  /** Count by type */
  byType: Record<PolicyType, number>;

  /** Total active members across all policies */
  totalMembers: number;

  /** Total monthly recurring revenue (subscription only) */
  totalMRR: number;

  /** Number of policies expiring soon */
  expiringSoon: number;

  /** Active subscription plans */
  activeSubscriptions: number;

  /** Active manual groups */
  activeGroups: number;
}

/** Default stats values */
export const DEFAULT_POLICY_STATS: PolicyStats = {
  totalPolicies: 0,
  byType: {
    subscription: 0,
    manual: 0,
    web3_asset: 0,
    dao: 0,
    system: 0,
  },
  totalMembers: 0,
  totalMRR: 0,
  expiringSoon: 0,
  activeSubscriptions: 0,
  activeGroups: 0,
};

// ============================================================================
// TRANSFORM FUNCTIONS
// ============================================================================

const parsePrice = (price: string | number): number => {
  if (typeof price === 'string') {
    return parseFloat(price);
  }
  return price;
};

/**
 * Transform a PlanResponse to AccessPolicy
 */
export function planToPolicy(plan: SharedPlan): AccessPolicy {
  const price = parsePrice(plan.current_price);

  return {
    id: `plan-${plan.id}`,
    name: plan.name,
    description: plan.description ?? '',
    type: 'subscription',
    permissions: plan.permissions ?? [],
    memberCount: plan.member_count ?? 0,
    isActive: plan.is_active === true,

    // Subscription-specific
    pricing: {
      amount: isNaN(price) ? 0 : price,
      currency: (plan.metadata?.currency as string) ?? 'USD',
      cycle: (plan.metadata?.billing_cycle as string) ?? 'monthly',
    },
    revenue: 0, // Not available on SharedPlan directly
    tierLevel: (plan.metadata?.tier_level as number) ?? 0,
    planCategory: (plan.metadata?.plan_category as string) ?? 'standard',

    // Common
    createdAt: plan.created_at,
    updatedAt: plan.updated_at ?? plan.created_at,
    sourceId: String(plan.id),
    sourceType: 'plan',
  };
}

/**
 * Transform a PermissionGroup to AccessPolicy
 */
export function groupToPolicy(group: PermissionGroup): AccessPolicy {
  // Map group_type to PolicyType
  const typeMap: Record<string, PolicyType> = {
    manual: 'manual',
    subscription: 'subscription', // Subscription groups are handled separately
    web3_asset: 'web3_asset',
    dao_membership: 'dao',
    dao: 'dao',
    admin: 'system',
    system: 'system',
  };

  const policyType = typeMap[group.plan_type] ?? 'manual';
  const isSystem =
    group.is_system_plan === true || group.plan_type === 'system';

  return {
    id: `group-${group.id}`,
    name: group.name,
    description: group.description ?? '',
    type: policyType,
    permissions: group.permissions ?? [],
    memberCount: group.member_count ?? 0,
    isActive: group.is_active === true,

    // Group-specific
    expiryDays: group.default_expiry_days,
    priorityLevel: group.priority_level ?? 0,
    isSystemGroup: isSystem,
    slug: group.slug,

    // Common
    createdAt: group.created_at,
    updatedAt: group.updated_at ?? group.created_at,
    sourceId: group.id,
    sourceType: 'group',
  };
}

/**
 * Check if policy is a subscription type
 */
export function isSubscriptionPolicy(policy: AccessPolicy): boolean {
  return policy.sourceType === 'plan' || policy.type === 'subscription';
}

/**
 * Check if policy is a group type
 */
export function isGroupPolicy(policy: AccessPolicy): boolean {
  return policy.sourceType === 'group';
}

/**
 * Get the edit URL for a policy
 */
export function getPolicyEditUrl(policy: AccessPolicy): string {
  if (policy.sourceType === 'plan') {
    // Legacy route kept for plan editing
    return `/subscriptions/plans/${policy.sourceId}/edit`;
  }
  // Unified group management
  return `/wallet-management/groups/${policy.sourceId}`;
}

/**
 * Get the members URL for a policy
 */
export const getPolicyMembersUrl = getPolicyEditUrl;

// ============================================================================
// TYPE CONFIG
// ============================================================================

/** Visual configuration for policy types */
export const POLICY_TYPE_CONFIG: Record<
  PolicyType,
  {
    label: string;
    icon: string;
    gradient: string;
    badgeClass: string;
  }
> = {
  subscription: {
    label: 'Subscription',
    icon: '💳',
    gradient: 'from-blue-500/20 via-indigo-500/20 to-blue-500/20',
    badgeClass:
      'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-800',
  },
  manual: {
    label: 'Manual',
    icon: '👥',
    gradient: 'from-amber-500/20 via-orange-500/20 to-amber-500/20',
    badgeClass:
      'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-900/30 dark:text-amber-400 dark:border-amber-800',
  },
  web3_asset: {
    label: 'Web3 Asset',
    icon: '🔗',
    gradient: 'from-purple-500/20 via-pink-500/20 to-purple-500/20',
    badgeClass:
      'bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800',
  },
  dao: {
    label: 'DAO',
    icon: '🏛️',
    gradient: 'from-emerald-500/20 via-teal-500/20 to-emerald-500/20',
    badgeClass:
      'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800',
  },
  system: {
    label: 'System',
    icon: '⚙️',
    gradient: 'from-gray-500/20 via-slate-500/20 to-gray-500/20',
    badgeClass:
      'bg-gray-50 text-gray-700 border-gray-200 dark:bg-gray-900/30 dark:text-gray-400 dark:border-gray-700',
  },
};
