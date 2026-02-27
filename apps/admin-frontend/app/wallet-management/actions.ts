'use server';

import type {
    WalletFilters
} from '@/components/wallet/types';
import type {
    DisableWalletRequest,
    EnableWalletRequest,
    WalletListResponse
} from '@/lib/api/wallet-management-client';
import { redirectOnForbidden } from '@/lib/api-error';
import { logout } from '@/lib/auth/auth';
import { createAdminApiClient } from '@/shared/api';
// Mappers moved to @/lib/mappers/wallet
import { mapWalletDtoToData } from '@/lib/mappers/wallet';

// ============================================================================
// SERVER ACTIONS
// ============================================================================

const WALLET_MGMT_ROUTE = '/wallet-management';

// Helper to check and handle auth errors
async function checkAuthError(error?: { code?: string; message?: string } | null) {
    if (!error) { return; }

    // Fix: Remove unnecessary optional chain if message is known to be string when error exists
    // But safely, message might be missing in some error shapes.
    const isUnauthorized = error.code === 'UNAUTHORIZED'
        || error.code === '401'
        || (error.message?.includes('Unauthorized') ?? false);

    if (isUnauthorized) {
        await logout();
    }
}

// Helper to build query params
function buildQueryParams(filters: WalletFilters, page: number, limit: number): Record<string, string> {
    const params: Record<string, string> = {
        page: page.toString(),
        limit: limit.toString(),
        sort_by: filters.sortBy,
        sort_order: filters.sortOrder,
    };
    if (filters.search !== '') { params['search'] = filters.search; }
    if (filters.status !== 'all') { params['status'] = filters.status === 'disabled' ? 'inactive' : filters.status; }
    return params;
}

// Helper to extract data handling different response formats
function extractWalletsData(rawData: WalletListResponse) {
    const internalData = rawData.data;

    // Use internal data if generally available, otherwise fallback to root properties
    // The conditional usage of ?? handles the case where internalData is undefined
    let rawWallets = rawData.wallets;
    if (internalData?.wallets) {
        rawWallets = internalData.wallets;
    }
    const wallets = rawWallets.map(mapWalletDtoToData);

    let pagination = rawData.pagination;
    if (internalData?.pagination) {
        pagination = internalData.pagination;
    }

    pagination ??= {
        page: 1,
        limit: 9,
        total: wallets.length,
        total_pages: 1,
        has_next_page: false,
        has_previous_page: false
    };

    return { wallets, pagination };
}

export async function fetchWalletsAction(filters: WalletFilters, page = 1, limit = 20) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const params = buildQueryParams(filters, page, limit);

    const res = await apiClient.get<WalletListResponse>('/api/admin/wallets', params);

    if (!res.success) {
        redirectOnForbidden(res, WALLET_MGMT_ROUTE);
        await checkAuthError(res.error);
        const msg = res.error?.message ?? 'Failed to fetch wallets';
        const code = res.error?.code ?? 'UNKNOWN';
        return { success: false as const, error: `${msg} (${code})` };
    }

    const rawData = res.data;
    if (!rawData) {
        return { success: false as const, error: 'No data returned' };
    }

    return { success: true as const, ...extractWalletsData(rawData) };
}

export async function updateWalletMetadataAction(walletAddress: string, data: { label?: string | null; note?: string | null }) {
    const apiClient = createAdminApiClient({ serverSide: true });

    const res = await apiClient.put(`/api/admin/wallets/${walletAddress}`, {
        metadata: { label: data.label ?? undefined, note: data.note ?? undefined },
    });

    if (!res.success) {
        redirectOnForbidden(res, WALLET_MGMT_ROUTE);
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to update metadata');
    }
}

export async function disableWalletAction(walletAddress: string, data: DisableWalletRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/disable`, data);

    if (!res.success) {
        redirectOnForbidden(res, WALLET_MGMT_ROUTE);
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to disable wallet');
    }
}

export async function enableWalletAction(walletAddress: string, data: EnableWalletRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/enable`, data);

    if (!res.success) {
        redirectOnForbidden(res, WALLET_MGMT_ROUTE);
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to enable wallet');
    }
}

