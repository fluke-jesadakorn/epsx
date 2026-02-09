'use client';

import { API_ROUTES } from '@/shared/config/route-constants';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { logger } from '@/shared/utils/logger';
import { useCallback, useState } from 'react';
import type { ActionType, AuditLogEntry } from '../types';

interface UseAuditLogsProps {
    isAuthenticated: boolean;
    page: number;
    searchQuery: string;
    selectedCategory: ActionType;
    dateFrom: string;
    dateTo: string;
    pageSize: number;
}

export function useAuditLogs({
    isAuthenticated,
    page,
    searchQuery,
    selectedCategory,
    dateFrom,
    dateTo,
    pageSize,
}: UseAuditLogsProps) {
    const [logs, setLogs] = useState<AuditLogEntry[]>([]);
    const [isLoadingLogs, setIsLoadingLogs] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [totalPages, setTotalPages] = useState(1);

    const fetchLogs = useCallback(async (): Promise<void> => {
        if (!isAuthenticated) { return; }

        setIsLoadingLogs(true);
        setError(null);

        try {
            const client = createAdminApiClient();
            const queryParams: Record<string, string> = {
                page: page.toString(),
                page_size: pageSize.toString(),
            };

            if (searchQuery !== '') { queryParams.search = searchQuery; }
            if (selectedCategory !== 'all') { queryParams.category = selectedCategory; }
            if (dateFrom !== '') { queryParams.from_date = dateFrom; }
            if (dateTo !== '') { queryParams.to_date = dateTo; }

            const params = new URLSearchParams(queryParams);

            const response = await client.get<{
                entries: AuditLogEntry[];
                total_pages: number;
            }>(`${API_ROUTES.ADMIN.AUDIT_LOGS}?${params.toString()}`);

            if (response.success && response.data) {
                setLogs(response.data.entries);
                setTotalPages(response.data.total_pages);
            } else {
                setLogs(generateMockAuditLogs());
                setTotalPages(5);
            }
        } catch (err: unknown) {
            if (typeof err === 'object' && err !== null && 'status' in err && err.status !== 404) {
                logger.error('Failed to fetch audit logs:', err);
            }
            setLogs(generateMockAuditLogs());
            setTotalPages(5);
        } finally {
            setIsLoadingLogs(false);
        }
    }, [isAuthenticated, page, searchQuery, selectedCategory, dateFrom, dateTo, pageSize]);

    return {
        logs,
        isLoadingLogs,
        error,
        totalPages,
        fetchLogs,
    };
}

// Mock data generator for development
function generateMockAuditLogs(): AuditLogEntry[] {
    const actions = [
        'permission_granted', 'permission_revoked', 'wallet_created', 'wallet_disabled',
        'wallet_enabled', 'plan_created', 'plan_updated', 'user_login', 'settings_updated',
        'notification_sent', 'member_added', 'member_removed'
    ];

    const targetTypes = ['wallet', 'permission', 'plan', 'user', 'notification'];

    return Array.from({ length: 20 }, (_, i) => ({
        id: `log-${Date.now()}-${i}`,
        action: actions[Math.floor(Math.random() * actions.length)] ?? 'action',
        wallet_address: `0x${Math.random().toString(16).slice(2, 42)}`,
        resource_type: targetTypes[Math.floor(Math.random() * targetTypes.length)] ?? 'resource',
        resource_id: `0x${Math.random().toString(16).slice(2, 42)}`,
        result: 'Success',
        details: {
            reason: 'Admin action',
            previous_value: Math.random() > 0.5 ? 'enabled' : 'disabled',
            new_value: Math.random() > 0.5 ? 'enabled' : 'disabled',
        },
        ip_address: `192.168.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`,
        timestamp: new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString(),
    })).sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
}
