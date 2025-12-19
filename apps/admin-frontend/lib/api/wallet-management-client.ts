/**
 * Wallet Management API Client
 * API functions for the Wallet Management Hub
 */

'use client';

import type {
    DisableReasonCategory,
    PermissionSource,
    Platform,
    WalletActivityEvent,
    WalletData,
    WalletFilters,
    WalletPermission,
    WalletStats,
    WalletSubscription,
} from '@/components/wallet/types';
import { adminApiClient } from '../api-client';

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

export interface WalletListResponse {
    wallets: WalletSummaryDto[];
    data?: {
        wallets?: WalletSummaryDto[];
        pagination?: {
            page: number;
            limit: number;
            total: number;
            total_pages: number;
            has_next_page: boolean;
            has_previous_page: boolean;
        };
    };
    pagination?: {
        page: number;
        limit: number;
        total: number;
        total_pages: number;
        has_next_page: boolean;
        has_previous_page: boolean;
    };
}

export interface WalletSummaryDto {
    wallet_address: string;
    is_active: boolean;
    created_at: string;
    last_auth_at?: string;
    metadata?: Record<string, unknown>;
    permissions: Array<{
        permission: string;
        expires_at?: string;
        is_active: boolean;
        source?: string;
    }>;
    groups: Array<{
        group_name: string;
        role?: string;
    }>;
    subscriptions?: Array<{
        plan_id: string;
        plan_name: string;
        status: string;
        started_at: string;
        expires_at?: string;
    }>;
}

export interface WalletStatsDto {
    total_users: number;
    active_users: number;
    inactive_users: number;
    new_users_30_days: number;
    active_users_30_days: number;
    growth_rate: number;
}

export interface DisableWalletRequest {
    duration_days?: number; // undefined = until manual re-enable
    reason_category: DisableReasonCategory;
    reason_details: string;
    affected_platforms: Platform[];
    block_login: boolean;
    pause_subscriptions: boolean;
    notify_user: boolean;
}

export interface EnableWalletRequest {
    platforms_to_enable: Platform[];
    restore_permissions: boolean;
    resume_subscriptions: boolean;
    resolution_note: string;
}

export interface AssignPermissionRequest {
    wallet_addresses: string[];
    permission_strings: string[];
    expires_at?: string;
    reason?: string;
}

// ============================================================================
// MAPPERS
// ============================================================================

function mapWalletDtoToData(dto: WalletSummaryDto): WalletData {
    // Determine platforms from permissions (may be undefined)
    const platforms = new Set<Platform>();
    const dtoPermissions = dto.permissions || [];
    dtoPermissions.forEach(p => {
        if (p.permission.startsWith('epsx:analytics') || p.permission.startsWith('epsx:rankings')) {
            platforms.add('analytics');
        } else if (p.permission.startsWith('epsx-pay:')) {
            platforms.add('pay');
        } else if (p.permission.startsWith('epsx-token:')) {
            platforms.add('token');
        } else if (p.permission.startsWith('epsx-markets:')) {
            platforms.add('markets');
        }
    });

    // Default to analytics if no specific platform detected
    if (platforms.size === 0) {
        platforms.add('analytics');
    }

    // Map permissions
    const permissions: WalletPermission[] = dtoPermissions.map((p, idx) => ({
        id: `perm-${idx}`,
        permission: p.permission,
        platform: detectPlatform(p.permission),
        source: (p.source as PermissionSource) || 'system',
        expiresAt: p.expires_at,
        isActive: p.is_active,
        createdAt: dto.created_at,
    }));

    // Map subscriptions if available
    const subscriptions: WalletSubscription[] = (dto.subscriptions || []).map((s, idx) => ({
        id: `sub-${idx}`,
        planId: s.plan_id,
        planName: s.plan_name,
        status: s.status as 'active' | 'cancelled' | 'expired' | 'paused',
        priceDisplay: '', // Not available from backend
        startedAt: s.started_at,
        expiresAt: s.expires_at,
        grantedPermissions: [], // Would need to be fetched from plan
    }));

    // Determine status
    let status: 'active' | 'disabled' | 'pending' = 'active';
    if (!dto.is_active) {
        status = 'disabled';
    }

    // Parse disable info from metadata if present
    const disableInfo = dto.metadata?.['disable_info'] as WalletData['disableInfo'] | undefined;

    return {
        walletAddress: dto.wallet_address,
        status,
        disableInfo,
        createdAt: dto.created_at,
        lastAuthAt: dto.last_auth_at,
        platforms: Array.from(platforms),
        permissions,
        subscriptions,
        metadata: dto.metadata,
    };
}

function detectPlatform(permission: string): Platform {
    if (permission.startsWith('epsx:analytics') || permission.startsWith('epsx:rankings')) {
        return 'analytics';
    }
    if (permission.startsWith('epsx-pay:')) {
        return 'pay';
    }
    if (permission.startsWith('epsx-token:')) {
        return 'token';
    }
    if (permission.startsWith('epsx-markets:')) {
        return 'markets';
    }
    return 'analytics'; // Default
}

function mapStatsToFrontend(dto: WalletStatsDto): WalletStats {
    return {
        total: dto.total_users,
        active: dto.active_users,
        disabled: dto.inactive_users,
        subscribed: 0, // TODO: Add to backend stats
        changes: {
            total: dto.new_users_30_days,
            active: Math.round(dto.growth_rate * dto.active_users / 100),
            disabled: 0,
            subscribed: 0,
        },
        platformDistribution: {
            analytics: dto.active_users, // TODO: Get real distribution from backend
            pay: 0,
            token: 0,
            markets: 0,
        },
    };
}

// ============================================================================
// WALLET MANAGEMENT API
// ============================================================================

export const walletMgmt = {
    /**
     * Fetch paginated list of wallets with filtering
     */
    async fetchWallets(filters: WalletFilters, page = 1, limit = 20): Promise<{
        wallets: WalletData[];
        pagination: WalletListResponse['pagination'];
    }> {
        const params: Record<string, string> = {
            page: page.toString(),
            limit: limit.toString(),
            sort_by: filters.sortBy,
            sort_order: filters.sortOrder,
        };

        if (filters.search) {
            params['search'] = filters.search;
        }

        if (filters.status !== 'all') {
            params['status'] = filters.status;
        }

        // Platform filter would need backend support
        // if (filters.platform !== 'all') {
        //   params.platform = filters.platform;
        // }

        const res = await adminApiClient.get<WalletListResponse>('/api/v1/admin/wallets', params);

        // Handle AdminApiResponse wrapper: { success, data: { wallets, pagination }, message }
        const responseData = res.data?.data || res.data;
        const rawWallets = responseData?.wallets || [];
        const wallets = rawWallets.map(mapWalletDtoToData);

        const rawPagination = responseData?.pagination || res.data?.pagination;
        const pagination = rawPagination || {
            page: 1,
            limit: 20,
            total: wallets.length,
            total_pages: 1,
            has_next_page: false,
            has_previous_page: false,
        };

        return { wallets, pagination };
    },

    /**
     * Fetch wallet statistics
     */
    async fetchWalletStats(): Promise<WalletStats> {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const res = await adminApiClient.get<any>('/api/v1/admin/wallets/stats');
        // Backend returns { success, data: { stats }, message, metadata }
        const responseData = res.data?.data || res.data;
        const stats = responseData?.stats || responseData || {
            total_users: 0,
            active_users: 0,
            inactive_users: 0,
            new_users_30_days: 0,
            active_users_30_days: 0,
            growth_rate: 0,
        };
        return mapStatsToFrontend(stats);
    },

    /**
     * Fetch detailed wallet information
     */
    async fetchWalletDetail(walletAddress: string): Promise<WalletData> {
        const res = await adminApiClient.get<{ data: WalletSummaryDto }>(
            `/api/v1/admin/wallets/${walletAddress}`
        );

        if (!res.data?.data) {
            throw new Error('Wallet not found');
        }

        return mapWalletDtoToData(res.data.data);
    },

    /**
     * Assign permissions to a wallet
     */
    async assignPermission(data: {
        walletAddress: string;
        permissions: string[];
        expiresAt?: string;
        reason?: string;
    }): Promise<void> {
        await adminApiClient.post('/api/v1/admin/permissions/bulk/grant', {
            wallet_addresses: [data.walletAddress],
            permission_strings: data.permissions,
            expires_at: data.expiresAt,
            reason: data.reason,
        });
    },

    /**
     * Revoke permissions from a wallet
     */
    async revokePermission(data: {
        walletAddress: string;
        permissions: string[];
    }): Promise<void> {
        await adminApiClient.post('/api/v1/admin/permissions/bulk/revoke', {
            wallet_addresses: [data.walletAddress],
            permission_strings: data.permissions,
        });
    },

    /**
     * Temporarily disable a wallet
     */
    async disableWallet(walletAddress: string, data: DisableWalletRequest): Promise<void> {
        await adminApiClient.post(`/api/v1/admin/wallets/${walletAddress}/disable`, data);
    },

    /**
     * Re-enable a disabled wallet
     */
    async enableWallet(walletAddress: string, data: EnableWalletRequest): Promise<void> {
        await adminApiClient.post(`/api/v1/admin/wallets/${walletAddress}/enable`, data);
    },

    /**
     * Fetch activity history for a wallet
     */
    async fetchActivityHistory(walletAddress: string, limit = 20): Promise<WalletActivityEvent[]> {
        const res = await adminApiClient.get<{ events: WalletActivityEvent[] }>(
            `/api/v1/admin/wallets/${walletAddress}/activity`,
            { limit: limit.toString() }
        );

        return res.data?.events || [];
    },

    /**
     * Bulk operations on multiple wallets
     */
    async bulkGrantPermissions(data: AssignPermissionRequest): Promise<void> {
        await adminApiClient.post('/api/v1/admin/permissions/bulk/grant', data);
    },

    async bulkRevokePermissions(data: {
        wallet_addresses: string[];
        permission_strings: string[];
    }): Promise<void> {
        await adminApiClient.post('/api/v1/admin/permissions/bulk/revoke', data);
    },

    async bulkDisable(walletAddresses: string[], reason: string): Promise<void> {
        // Note: Backend would need to support bulk disable
        await Promise.all(
            walletAddresses.map(addr =>
                this.disableWallet(addr, {
                    reason_category: 'other',
                    reason_details: reason,
                    affected_platforms: ['analytics', 'pay', 'token', 'markets'],
                    block_login: true,
                    pause_subscriptions: false,
                    notify_user: false,
                })
            )
        );
    },
};

export default walletMgmt;
