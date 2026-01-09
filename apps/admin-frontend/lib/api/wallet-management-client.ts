/**
 * ADMIN WALLET MANAGEMENT CLIENT
 *
 * Re-exports types from shared and provides admin-specific wallet operations.
 * This wrapper adds admin-frontend specific functionality on top of the shared WalletsApi.
 */

'use client';

import { adminApiClient } from '../api-client';

// Re-export shared types
export {
    createWalletsClient, WalletsApi, type RecentWallet, type WalletStats as SharedWalletStats, type WalletActivity, type WalletInfo,
    type WalletSearchFilters
} from '@/shared/api/wallets';

// Import local types
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

// Re-export for consumers
export type { WalletActivityEvent, WalletData, WalletFilters, WalletPermission, WalletStats, WalletSubscription };

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
    duration_days?: number | null;
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
    if (platforms.size === 0) platforms.add('analytics');

    const permissions: WalletPermission[] = dtoPermissions.map((p, idx) => ({
        id: `perm-${idx}`,
        permission: p.permission,
        platform: detectPlatform(p.permission),
        source: (p.source as PermissionSource) || 'system',
        expiresAt: p.expires_at,
        isActive: p.is_active,
        createdAt: dto.created_at,
    }));

    const subscriptions: WalletSubscription[] = (dto.subscriptions || []).map((s, idx) => ({
        id: `sub-${idx}`,
        planId: s.plan_id,
        planName: s.plan_name,
        status: s.status as 'active' | 'cancelled' | 'expired' | 'paused',
        priceDisplay: '',
        startedAt: s.started_at,
        expiresAt: s.expires_at,
        grantedPermissions: [],
    }));

    let status: 'active' | 'disabled' | 'pending' = 'active';
    if (!dto.is_active) status = 'disabled';

    const disableInfo = dto.metadata?.['disable_info'] as WalletData['disableInfo'] | undefined;
    const label = dto.metadata?.['label'] as string | undefined;
    const note = dto.metadata?.['note'] as string | undefined;

    return {
        walletAddress: dto.wallet_address,
        status,
        disableInfo,
        createdAt: dto.created_at,
        lastAuthAt: dto.last_auth_at,
        platforms: Array.from(platforms),
        permissions,
        subscriptions,
        groups: (dto.groups || []).map(g => ({ groupName: g.group_name, role: g.role })),
        metadata: dto.metadata,
        label,
        note,
    };
}

function detectPlatform(permission: string): Platform {
    if (permission.startsWith('epsx:analytics') || permission.startsWith('epsx:rankings')) return 'analytics';
    if (permission.startsWith('epsx-pay:')) return 'pay';
    if (permission.startsWith('epsx-token:')) return 'token';
    if (permission.startsWith('epsx-markets:')) return 'markets';
    return 'analytics';
}

function mapStatsToFrontend(dto: WalletStatsDto): WalletStats {
    return {
        total: dto.total_users,
        active: dto.active_users,
        disabled: dto.inactive_users,
        subscribed: 0,
        changes: {
            total: dto.new_users_30_days,
            active: Math.round(dto.growth_rate * dto.active_users / 100),
            disabled: 0,
            subscribed: 0,
        },
        platformDistribution: {
            analytics: dto.active_users,
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
    async fetchWallets(filters: WalletFilters, page = 1, limit = 20): Promise<{ wallets: WalletData[]; pagination: WalletListResponse['pagination'] }> {
        const params: Record<string, string> = {
            page: page.toString(),
            limit: limit.toString(),
            sort_by: filters.sortBy,
            sort_order: filters.sortOrder,
        };
        if (filters.search) params['search'] = filters.search;
        if (filters.status !== 'all') params['status'] = filters.status;

        const res = await adminApiClient.get<WalletListResponse>('/api/admin/wallets', params);
        const responseData = res.data?.data || res.data;
        const rawWallets = responseData?.wallets || [];
        const wallets = rawWallets.map(mapWalletDtoToData);
        const rawPagination = responseData?.pagination || res.data?.pagination;
        const pagination = rawPagination || { page: 1, limit: 20, total: wallets.length, total_pages: 1, has_next_page: false, has_previous_page: false };
        return { wallets, pagination };
    },

    async fetchWalletStats(): Promise<WalletStats> {
        const res = await adminApiClient.get<any>('/api/admin/wallets/stats');
        const responseData = res.data?.data || res.data;
        const stats = responseData?.stats || responseData || { total_users: 0, active_users: 0, inactive_users: 0, new_users_30_days: 0, active_users_30_days: 0, growth_rate: 0 };
        return mapStatsToFrontend(stats);
    },

    async fetchWalletDetail(walletAddress: string): Promise<WalletData> {
        const res = await adminApiClient.get<any>(`/api/admin/wallets/${walletAddress}`);
        const responseData = res.data?.data || res.data;
        const walletDto = responseData?.wallet || responseData;
        if (!walletDto || !walletDto.wallet_address) throw new Error('Wallet not found');
        return mapWalletDtoToData(walletDto);
    },

    async assignPermission(data: { walletAddress: string; permissions: string[]; expiresAt?: string; reason?: string }): Promise<void> {
        await adminApiClient.post('/api/admin/permissions/bulk/grant', {
            wallet_addresses: [data.walletAddress],
            permission_strings: data.permissions,
            expires_at: data.expiresAt,
            reason: data.reason,
        });
    },

    async revokePermission(data: { walletAddress: string; permissions: string[] }): Promise<void> {
        await adminApiClient.post('/api/admin/permissions/bulk/revoke', {
            wallet_addresses: [data.walletAddress],
            permission_strings: data.permissions,
        });
    },

    async disableWallet(walletAddress: string, data: DisableWalletRequest): Promise<void> {
        await adminApiClient.post(`/api/admin/wallets/${walletAddress}/disable`, data);
    },

    async enableWallet(walletAddress: string, data: EnableWalletRequest): Promise<void> {
        await adminApiClient.post(`/api/admin/wallets/${walletAddress}/enable`, data);
    },

    async fetchActivityHistory(walletAddress: string, limit = 20): Promise<WalletActivityEvent[]> {
        const res = await adminApiClient.get<{ events: WalletActivityEvent[] }>(`/api/admin/wallets/${walletAddress}/activity`, { limit: limit.toString() });
        return res.data?.events || [];
    },

    async bulkGrantPermissions(data: AssignPermissionRequest): Promise<void> {
        await adminApiClient.post('/api/admin/permissions/bulk/grant', data);
    },

    async bulkRevokePermissions(data: { wallet_addresses: string[]; permission_strings: string[] }): Promise<void> {
        await adminApiClient.post('/api/admin/permissions/bulk/revoke', data);
    },

    async bulkDisable(walletAddresses: string[], reason: string): Promise<void> {
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

    async updateWalletMetadata(walletAddress: string, data: { label?: string | null; note?: string | null }): Promise<void> {
        await adminApiClient.put(`/api/admin/wallets/${walletAddress}`, {
            metadata: { label: data.label ?? undefined, note: data.note ?? undefined },
        });
    },
};

export default walletMgmt;
