/**
 * SHARED WALLET & PLATFORM TYPES
 * Canonical definitions for wallet status and platform identifiers
 */

export type WalletStatus = 'active' | 'inactive' | 'suspended' | 'deleted';

export type UserStatus = WalletStatus | 'pending' | 'trial';

export type Platform = 'admin' | 'epsx' | 'epsx-pay' | 'epsx-token' | 'frontend';

export type PermissionType = 'permanent' | 'temporary' | 'expired';

export interface WalletMetadata {
    displayName?: string;
    notes?: string;
    tags?: string[];
    [key: string]: unknown;
}

export interface ActivityHistoryItem {
    id: string;
    type: string;
    description: string;
    timestamp: string;
    metadata?: Record<string, unknown>;
}
