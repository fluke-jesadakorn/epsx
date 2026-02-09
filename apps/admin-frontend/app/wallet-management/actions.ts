'use server';

import type {
    WalletFilters
} from '@/components/wallet/types';
import type {
    DisableWalletRequest,
    EnableWalletRequest,
    WalletListResponse
} from '@/lib/api/wallet-management-client';
import { logout } from '@/lib/auth/auth';
import { createAdminApiClient } from '@/shared/api';
import { redirect } from 'next/navigation';

// Mappers moved to @/lib/mappers/wallet
import { mapWalletDtoToData } from '@/lib/mappers/wallet';

// ============================================================================
// SERVER ACTIONS
// ============================================================================

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
        redirect('/auth');
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
    if (filters.status !== 'all') { params['status'] = filters.status; }
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
        limit: 20,
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
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to fetch wallets');
    }

    const rawData = res.data;
    if (!rawData) {
        throw new Error('Failed to fetch wallets: No data');
    }

    return extractWalletsData(rawData);
}

export async function updateWalletMetadataAction(walletAddress: string, data: { label?: string | null; note?: string | null }) {
    const apiClient = createAdminApiClient({ serverSide: true });

    const res = await apiClient.put(`/api/admin/wallets/${walletAddress}`, {
        metadata: { label: data.label ?? undefined, note: data.note ?? undefined },
    });

    if (!res.success) {
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to update metadata');
    }
}

export async function disableWalletAction(walletAddress: string, data: DisableWalletRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/disable`, data);

    if (!res.success) {
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to disable wallet');
    }
}

export async function enableWalletAction(walletAddress: string, data: EnableWalletRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/enable`, data);

    if (!res.success) {
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to enable wallet');
    }
}

interface ActivityLogEntry {
    id: string;
    action: string;
    timestamp: string;
    wallet_address: string;
    details: unknown;
}

export async function fetchActivityLogsAction(walletAddress?: string, page = 1, limit = 10) {
    const apiClient = createAdminApiClient({ serverSide: true });

    let endpoint = '/api/admin/audit-logs';
    if (walletAddress !== undefined && walletAddress.length > 0) {
        endpoint = `/api/admin/wallets/${walletAddress}/activity`;
    }

    const res = await apiClient.get<Record<string, unknown>>(endpoint, {
        page: page.toString(),
        page_size: limit.toString(),
    });

    if (!res.success) {
        await checkAuthError(res.error);
        throw new Error(res.error?.message ?? 'Failed to fetch activity logs');
    }

    const data = res.data;
    if (!data) {
        return [];
    }

    // Map common format
    const entries = (data['entries'] ?? data['events'] ?? []) as ActivityLogEntry[];

    // Simple mapper for display
    return entries.map((log) => ({
        id: log.id,
        action: log.action,
        timestamp: log.timestamp,
        wallet_address: log.wallet_address,
        details: log.details,
    }));
}
