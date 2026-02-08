'use server';

import type {
    PermissionSource,
    Platform,
    WalletData,
    WalletFilters,
    WalletPermission,
    WalletSubscription
} from '@/components/wallet/types';
import type {
    DisableWalletRequest,
    EnableWalletRequest,
    WalletListResponse,
    WalletSummaryDto
} from '@/lib/api/wallet-management-client';
import { logout } from '@/lib/auth/auth';
import { createAdminApiClient } from '@/shared/api';
import { redirect } from 'next/navigation';

// ============================================================================
// MAPPERS (Duplicated from client to ensure server-side isolation)
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
    if (platforms.size === 0) { platforms.add('analytics'); }

    const permissions: WalletPermission[] = dtoPermissions.map((p, idx) => ({
        id: `perm-${idx}`,
        permission: p.permission,
        platform: detectPlatform(p.permission),
        source: (p.source as PermissionSource) ?? 'system',
        expiresAt: p.expires_at,
        isActive: p.is_active,
        createdAt: dto.created_at,
    }));

    const subscriptions: WalletSubscription[] = (dto.subscriptions || []).map((s, idx) => ({
        id: `sub-${idx}`,
        planId: s.plan_id,
        planName: s.plan_name,
        status: s.status as WalletSubscription['status'],
        priceDisplay: '',
        startedAt: s.started_at,
        expiresAt: s.expires_at,
        grantedPermissions: [],
    }));

    let status: 'active' | 'disabled' | 'pending' = 'active';
    if (!dto.is_active) { status = 'disabled'; }

    const disableInfo = dto.metadata?.['disable_info'] as WalletData['disableInfo'];
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
        plans: (dto.groups || []).map(g => ({ planName: g.group_name, role: g.role })),
        metadata: dto.metadata,
        label,
        note,
    };
}

function detectPlatform(permission: string): Platform {
    if (permission.startsWith('epsx:analytics') || permission.startsWith('epsx:rankings')) { return 'analytics'; }
    if (permission.startsWith('epsx-pay:')) { return 'pay'; }
    if (permission.startsWith('epsx-token:')) { return 'token'; }
    if (permission.startsWith('epsx-markets:')) { return 'markets'; }
    return 'analytics';
}

// ============================================================================
// SERVER ACTIONS
// ============================================================================

export async function fetchWalletsAction(filters: WalletFilters, page = 1, limit = 20) {
    const apiClient = createAdminApiClient({ serverSide: true });

    const params: Record<string, string> = {
        page: page.toString(),
        limit: limit.toString(),
        sort_by: filters.sortBy ?? 'created_at',
        sort_order: filters.sortOrder ?? 'desc',
    };
    if (filters.search) { params['search'] = filters.search; }
    if (filters.status && filters.status !== 'all') { params['status'] = filters.status; }

    const res = await apiClient.get<WalletListResponse>('/api/admin/wallets', params);

    if (!res.success) {
        // Gracefully handle 401 for client-side modal trigger or server-side redirect
        if (res.error?.code === 'UNAUTHORIZED' || res.error?.code === '401' || res.error?.message?.includes('Unauthorized')) {
            await logout();
            redirect('/auth');
        }
        throw new Error(res.error?.message ?? 'Failed to fetch wallets');
    }

    if (!res.data) {
        throw new Error('Failed to fetch wallets: No data');
    }

    const responseData = (res.data as any).data ?? res.data; // Handle potential double wrapping
    const rawWallets = (responseData as any).wallets ?? [];
    const wallets = rawWallets.map(mapWalletDtoToData);
    const pagination = (responseData as any).pagination ?? { page: 1, limit: 20, total: wallets.length, total_pages: 1, has_next_page: false, has_previous_page: false };

    return { wallets, pagination };
}

export async function updateWalletMetadataAction(walletAddress: string, data: { label?: string | null; note?: string | null }) {
    const apiClient = createAdminApiClient({ serverSide: true });

    const res = await apiClient.put(`/api/admin/wallets/${walletAddress}`, {
        metadata: { label: data.label ?? undefined, note: data.note ?? undefined },
    });

    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        throw new Error(res.error?.message ?? 'Failed to update metadata');
    }
}

export async function disableWalletAction(walletAddress: string, data: DisableWalletRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/disable`, data);

    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        throw new Error(res.error?.message ?? 'Failed to disable wallet');
    }
}

export async function enableWalletAction(walletAddress: string, data: EnableWalletRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/enable`, data);

    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
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
    const params: Record<string, string> = {
        page: page.toString(),
        page_size: limit.toString(),
    };

    const endpoint = walletAddress
        ? `/api/admin/wallets/${walletAddress}/activity`
        : '/api/admin/audit-logs';

    const res = await apiClient.get<Record<string, unknown>>(endpoint, params);

    if (!res.success || !res.data) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        throw new Error(res.error?.message ?? 'Failed to fetch activity logs');
    }

    // Map common format
    const logs = ((res.data.entries as ActivityLogEntry[] | undefined) ?? (res.data.events as ActivityLogEntry[] | undefined) ?? []) as ActivityLogEntry[];

    // Simple mapper for display
    return logs.map((log) => ({
        id: log.id,
        action: log.action,
        timestamp: log.timestamp,
        wallet_address: log.wallet_address,
        details: log.details,
    }));
}
