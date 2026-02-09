/**
 * ADMIN WALLET MANAGEMENT CLIENT
 *
 * Re-exports types from shared and provides admin-specific wallet operations.
 * This wrapper adds admin-frontend specific functionality on top of the shared WalletsApi.
 */

'use client';

import { extractData } from '@/shared/api';
import { adminApiClient } from '../api-client';

// Import local types
import type {
    DisableReasonCategory,
    Platform,
    WalletActivityEvent,
    WalletData,
    WalletFilters,
    WalletPermission,
    WalletStats,
    WalletSubscription
} from '@/components/wallet/types';
import { mapWalletDtoToData } from '@/lib/mappers/wallet';

// Re-export shared types
export {
    createWalletsClient, WalletsApi, type RecentWallet, type WalletStats as SharedWalletStats, type WalletActivity, type WalletInfo,
    type WalletSearchFilters
} from '@/shared/api/wallets';

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

// Mappers moved to @/lib/mappers/wallet

// ============================================================================
// WALLET MANAGEMENT API
// ============================================================================

export const walletMgmt = {
    getWallet: async (address: string) => {
        const res = await adminApiClient.get<Record<string, unknown>>(`/api/admin/wallets/${address}`);
        const data = extractData<Record<string, unknown>>(res);
        if (!data) {
            throw new Error('Wallet not found');
        }
        const dto = (data.wallet ?? data) as unknown as WalletSummaryDto;
        return mapWalletDtoToData(dto);
    },
    updateWalletMetadata: async (address: string, data: { label?: string | null, note?: string | null }) => {
        await adminApiClient.put(`/api/admin/wallets/${address}`, { metadata: data });
    },
    disableWallet: async (address: string, data: DisableWalletRequest) => {
        await adminApiClient.post(`/api/admin/wallets/${address}/disable`, data);
    },
    enableWallet: async (address: string, data: EnableWalletRequest) => {
        await adminApiClient.post(`/api/admin/wallets/${address}/enable`, data);
    },
    // Permissions
    grantPermission: async (address: string, permission: string, expiresAt?: string) => {
        await adminApiClient.post('/api/admin/permissions/direct/grant', { wallet_address: address, permission_string: permission, expires_at: expiresAt });
    },
    revokePermission: async (address: string, permission: string) => {
        await adminApiClient.post('/api/admin/permissions/direct/revoke', { wallet_address: address, permission_string: permission });
    },
    searchWallets: async (query: string, limit = 10) => {
        const res = await adminApiClient.get<WalletListResponse>('/api/admin/wallets', { search: query, limit: limit.toString() });
        const data = extractData<WalletListResponse>(res);
        return data?.data ?? data; // Handle potential wrapping
    },
    fetchWallets: async (filters: WalletFilters, page = 1, limit = 50) => {
        const queryParams: Record<string, string> = {
            page: page.toString(),
            limit: limit.toString(),
            search: filters.search,
            status: filters.status,
            platform: filters.platform,
            sort_by: filters.sortBy,
            sort_order: filters.sortOrder,
        };
        const res = await adminApiClient.get<WalletListResponse>('/api/admin/wallets', queryParams);
        const data = extractData<WalletListResponse>(res);
        const responseData = data?.data ?? data;

        return {
            wallets: (responseData?.wallets ?? []).map(mapWalletDtoToData),
            pagination: responseData?.pagination
        };
    },
    fetchWalletStats: async (): Promise<WalletStats> => {
        const res = await adminApiClient.get<WalletStatsDto>('/api/admin/wallets/stats');
        const data = extractData<WalletStatsDto>(res);

        return {
            total: data?.total_users ?? 0,
            active: data?.active_users ?? 0,
            disabled: data?.inactive_users ?? 0,
            subscribed: data?.active_users_30_days ?? 0,
            changes: {
                total: data?.new_users_30_days ?? 0,
                active: 0,
                disabled: 0,
                subscribed: 0
            },
            platformDistribution: {
                analytics: 0,
                pay: 0,
                token: 0,
                markets: 0
            }
        };
    }
};
