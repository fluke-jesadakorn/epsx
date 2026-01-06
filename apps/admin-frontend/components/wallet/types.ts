/**
 * Wallet Management Types
 * Shared types for the wallet management system
 */

/** Supported EPSX platforms */
export type Platform = 'analytics' | 'pay' | 'token' | 'markets';

/** Permission assignment source */
export type PermissionSource = 'manual' | 'auto' | 'system';

/** Wallet status */
export type WalletStatus = 'active' | 'disabled' | 'pending';

/** Permission data for a wallet */
export interface WalletPermission {
    id: string;
    permission: string;
    platform: Platform;
    source: PermissionSource;
    /** Plan name if source is 'auto' */
    sourcePlanName?: string;
    /** Admin who assigned if source is 'manual' */
    assignedBy?: string;
    expiresAt?: string;
    isActive: boolean;
    createdAt: string;
}

/** Subscription data for a wallet */
export interface WalletSubscription {
    id: string;
    planId: string;
    planName: string;
    status: 'active' | 'cancelled' | 'expired' | 'paused';
    priceDisplay: string;
    startedAt: string;
    expiresAt?: string;
    /** Permissions auto-granted by this subscription */
    grantedPermissions: string[];
    /** Who assigned this subscription */
    assignedBy?: string;
    /** Billing cycle (e.g., "monthly", "yearly") */
    billingCycle?: string;
    /** Days remaining until expiration */
    daysRemaining?: number;
    /** Whether manual renewal is available */
    canRenew?: boolean;
    /** Price for renewal */
    renewalPrice?: string;
}

/** Activity event for wallet timeline */
export interface WalletActivityEvent {
    id: string;
    type: 'permission_granted' | 'permission_revoked' | 'subscription_started' |
    'subscription_cancelled' | 'wallet_disabled' | 'wallet_enabled' |
    'wallet_created' | 'login';
    description: string;
    performedBy?: string;
    metadata?: Record<string, unknown>;
    timestamp: string;
}

/** Disable reason categories */
export type DisableReasonCategory =
    | 'suspicious_activity'
    | 'tos_violation'
    | 'pending_verification'
    | 'user_request'
    | 'other';

/** Disable info when wallet is disabled */
export interface WalletDisableInfo {
    disabledAt: string;
    disabledBy: string;
    duration: 'permanent' | number; // number = days
    expiresAt?: string;
    reasonCategory: DisableReasonCategory;
    reasonDetails: string;
    affectedPlatforms: Platform[];
}

/** Main wallet data structure */
export interface WalletData {
    walletAddress: string;
    status: WalletStatus;
    disableInfo?: WalletDisableInfo;
    createdAt: string;
    lastAuthAt?: string;
    permissions: WalletPermission[];
    subscriptions: WalletSubscription[];
    /** Platforms this wallet has access to */
    platforms: Platform[];
    metadata?: Record<string, unknown>;
}

/** Stats for dashboard */
export interface WalletStats {
    total: number;
    active: number;
    disabled: number;
    subscribed: number;
    changes: {
        total: number;
        active: number;
        disabled: number;
        subscribed: number;
    };
    platformDistribution: Record<Platform, number>;
}

/** Filter options for wallet list */
export interface WalletFilters {
    search: string;
    platform: Platform | 'all';
    status: WalletStatus | 'all';
    hasSubscription?: boolean;
    sortBy: 'created_at' | 'last_auth_at' | 'wallet_address';
    sortOrder: 'asc' | 'desc';
}
