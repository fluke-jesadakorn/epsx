/**
 * Audit Log Page
 * Track admin actions, permission changes, and system events
 */
'use client';

import React, { useEffect, useState } from 'react';

import { PageHeader, PageLayout } from '@/components/shared';

import { AuditLogFilters } from './components/audit-log-filters';
import { AuditLogTable } from './components/audit-log-table';
import { useAuditLogs } from './hooks/use-audit-logs';
import type { ActionType } from './types';

export default function AuditLogPage(): React.JSX.Element {
    // Filters
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCategory, setSelectedCategory] = useState<ActionType>('all');
    const [dateFrom, setDateFrom] = useState('');
    const [dateTo, setDateTo] = useState('');

    // Pagination
    const [page, setPage] = useState(1);
    const pageSize = 20;

    const {
        logs,
        isLoadingLogs,
        error,
        totalPages,
        fetchLogs,
    } = useAuditLogs({
        isAuthenticated: true,
        page,
        searchQuery,
        selectedCategory,
        dateFrom,
        dateTo,
        pageSize,
    });

    useEffect(() => {
        void fetchLogs();
    }, [fetchLogs]);

    // Export logs
    const handleExport = (): void => {
        const csvContent = [
            ['Date', 'Category', 'Action', 'Resource Type', 'Actor', 'Target', 'Result', 'Details'].join(','),
            ...logs.map(log => [
                new Date(log.timestamp).toISOString(),
                log.category ?? '',
                log.action_raw ?? log.action,
                log.resource_type_raw ?? log.resource_type,
                log.wallet_address ?? '',
                log.resource_id ?? '',
                log.result,
                JSON.stringify(log.details).replace(/,/g, ';')
            ].join(','))
        ].join('\n');

        const blob = new Blob([csvContent], { type: 'text/csv' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `audit-log-${new Date().toISOString().split('T')[0]}.csv`;
        a.click();
        URL.revokeObjectURL(url);
    };

    return (
        <PageLayout>
            {/* Header */}
            <PageHeader
                title="Audit Log"
                subtitle="Track all admin actions, permission changes, and system events"
                icon="FileText"
                gradient="indigo"
            />

            <AuditLogFilters
                searchQuery={searchQuery}
                setSearchQuery={setSearchQuery}
                selectedCategory={selectedCategory}
                setSelectedCategory={setSelectedCategory}
                dateFrom={dateFrom}
                setDateFrom={setDateFrom}
                dateTo={dateTo}
                setDateTo={setDateTo}
                fetchLogs={() => { void fetchLogs(); }}
                handleExport={handleExport}
                isLoadingLogs={isLoadingLogs}
            />

            <AuditLogTable
                isLoadingLogs={isLoadingLogs}
                error={error}
                logs={logs}
                page={page}
                totalPages={totalPages}
                setPage={setPage}
                fetchLogs={() => { void fetchLogs(); }}
            />
        </PageLayout>
    );
}
