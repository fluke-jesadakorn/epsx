'use server';

import { redirectOnForbidden } from '@/lib/api-error';
import { createAdminApiClient } from '@/shared/api';
import { API_ROUTES } from '@/shared/config/route-constants';
import type { ActionType, AuditLogEntry } from './types';

interface FetchAuditLogsParams {
    page: number;
    pageSize: number;
    search?: string;
    category?: ActionType;
    fromDate?: string;
    toDate?: string;
}

function buildAuditQueryParams(params: FetchAuditLogsParams): Record<string, string> {
    const q: Record<string, string> = {
        page: params.page.toString(),
        page_size: params.pageSize.toString(),
    };
    if (params.search !== undefined && params.search !== '') { q.search = params.search; }
    if (params.category !== undefined && params.category !== 'all') { q.category = params.category; }
    if (params.fromDate !== undefined && params.fromDate !== '') { q.from_date = params.fromDate; }
    if (params.toDate !== undefined && params.toDate !== '') { q.to_date = params.toDate; }
    return q;
}

export async function fetchAuditLogsAction(params: FetchAuditLogsParams): Promise<{
    success: boolean;
    entries: AuditLogEntry[];
    totalPages: number;
    error?: string;
}> {
    const client = createAdminApiClient({ serverSide: true });
    const qs = new URLSearchParams(buildAuditQueryParams(params));

    const res = await client.get<{
        entries: AuditLogEntry[];
        total_pages: number;
    }>(`${API_ROUTES.ADMIN.AUDIT_LOGS}?${qs.toString()}`);

    if (res.success && res.data) {
        return {
            success: true,
            entries: res.data.entries,
            totalPages: res.data.total_pages,
        };
    }

    redirectOnForbidden(res, '/audit-log');

    return {
        success: false,
        entries: [],
        totalPages: 0,
        error: res.error?.message ?? 'Failed to load audit logs',
    };
}
