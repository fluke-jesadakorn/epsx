/**
 * Audit Log Page
 * Track admin actions, permission changes, and system events
 */
'use client';

import { useRouter } from 'next/navigation';
import React, { useEffect, useState } from 'react';

import { PageAuthRequired, PageHeader, PageLayout, PageSkeleton } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth';

import { AuditLogFilters } from './components/audit-log-filters';
import { AuditLogTable } from './components/audit-log-table';
import { useAuditLogs } from './hooks/use-audit-logs';
import type { ActionType } from './types';

/**
 * Audit Log Page Component
 */
export default function AuditLogPage(): React.JSX.Element {
    const { isAuthenticated, isLoading } = useSharedAuth();
    const router = useRouter();

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
        isAuthenticated,
        page,
        searchQuery,
        selectedCategory,
        dateFrom,
        dateTo,
        pageSize,
    });

    // Redirect if not authenticated
    useEffect(() => {
        if (!isLoading && !isAuthenticated) {
            router.push('/auth');
        }
    }, [isAuthenticated, isLoading, router]);

    useEffect(() => {
        void fetchLogs();
    }, [fetchLogs]);

    // Export logs
    const handleExport = (): void => {
        const csvContent = [
            ['Date', 'Action', 'Actor', 'Target', 'Details'].join(','),
            ...logs.map(log => [
                new Date(log.timestamp).toISOString(),
                log.action,
                log.wallet_address ?? '',
                `${log.resource_type}:${log.resource_id ?? ''}`,
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

    if (isLoading) {
        return <PageSkeleton showHeader showTabs={false} stats={0} rows={10} />;
    }

    if (!isAuthenticated) {
        return <PageAuthRequired />;
    }

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
