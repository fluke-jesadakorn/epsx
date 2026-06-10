'use client';

import { useCallback, useState } from 'react';
import { fetchAuditLogsAction } from '../actions';
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
            const res = await fetchAuditLogsAction({
                page,
                pageSize,
                search: searchQuery,
                category: selectedCategory,
                fromDate: dateFrom,
                toDate: dateTo,
            });

            if (res.success) {
                setLogs(res.entries);
                setTotalPages(res.totalPages);
            } else {
                setLogs([]);
                setTotalPages(0);
                setError(res.error ?? 'Failed to load audit logs');
            }
        } catch {
            setLogs([]);
            setTotalPages(0);
            setError('Failed to load audit logs');
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
