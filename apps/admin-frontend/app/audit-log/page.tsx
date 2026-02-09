/**
 * Audit Log Page
 * Track admin actions, permission changes, and system events
 */
'use client';

import { ChevronLeft, ChevronRight, Download, RefreshCw, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React, { useCallback, useEffect, useState } from 'react';

import { PageAuthRequired, PageHeader, PageLayout, PageSkeleton } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { API_ROUTES } from '@/shared/config/route-constants';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { logger } from '@/shared/utils/logger';

// Types
interface AuditLogEntry {
    id: string;
    action: string;
    wallet_address: string | null;
    resource_type: string;
    resource_id: string | null;
    result: string;
    details: Record<string, unknown> | null;
    additional_data?: Record<string, unknown> | null;
    ip_address?: string;
    user_agent?: string;
    timestamp: string;
    created_at?: string;
}

type ActionType = 'all' | 'permission' | 'wallet' | 'plan' | 'system';

const ACTION_CATEGORIES: Record<ActionType, { label: string; icon: string; color: string }> = {
    all: { label: 'All Actions', icon: '📋', color: 'gray' },
    permission: { label: 'Permissions', icon: '🔐', color: 'green' },
    wallet: { label: 'Wallets', icon: '👛', color: 'blue' },
    plan: { label: 'Plans', icon: '💳', color: 'purple' },
    system: { label: 'System', icon: '⚙️', color: 'orange' },
};

/**
 * Audit Log Page Component
 */
export default function AuditLogPage(): React.JSX.Element {
    const { isAuthenticated, isLoading } = useSharedAuth();
    const router = useRouter();

    const [logs, setLogs] = useState<AuditLogEntry[]>([]);
    const [isLoadingLogs, setIsLoadingLogs] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Filters
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCategory, setSelectedCategory] = useState<ActionType>('all');
    const [dateFrom, setDateFrom] = useState('');
    const [dateTo, setDateTo] = useState('');

    // Pagination
    const [page, setPage] = useState(1);
    const [totalPages, setTotalPages] = useState(1);
    const pageSize = 20;

    // Redirect if not authenticated
    useEffect(() => {
        if (!isLoading && !isAuthenticated) {
            router.push('/auth');
        }
    }, [isAuthenticated, isLoading, router]);

    // Fetch audit logs
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
            if (err instanceof Object && 'status' in err && err.status !== 404) {
                logger.error('Failed to fetch audit logs:', err);
            }
            setLogs(generateMockAuditLogs());
            setTotalPages(5);
        } finally {
            setIsLoadingLogs(false);
        }
    }, [isAuthenticated, page, searchQuery, selectedCategory, dateFrom, dateTo]);

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

interface AuditLogFiltersProps {
    searchQuery: string;
    setSearchQuery: (q: string) => void;
    selectedCategory: ActionType;
    setSelectedCategory: (c: ActionType) => void;
    dateFrom: string;
    setDateFrom: (d: string) => void;
    dateTo: string;
    setDateTo: (d: string) => void;
    fetchLogs: () => void;
    handleExport: () => void;
    isLoadingLogs: boolean;
}

function AuditLogFilters({
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    dateFrom,
    setDateFrom,
    dateTo,
    setDateTo,
    fetchLogs,
    handleExport,
    isLoadingLogs,
}: AuditLogFiltersProps) {
    return (
        <div className="bg-card rounded-2xl p-4 shadow-xl border border-border/20">
            <div className="flex flex-col lg:flex-row gap-4">
                {/* Search */}
                <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search by actor, action, or target..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-10 pr-4 py-3 bg-muted border border-border rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                    />
                </div>

                {/* Category Filter */}
                <div className="flex gap-2 overflow-x-auto pb-2 lg:pb-0">
                    {Object.entries(ACTION_CATEGORIES).map(([key, { label, icon }]) => (
                        <button
                            key={key}
                            onClick={() => setSelectedCategory(key as ActionType)}
                            className={`px-4 py-2 rounded-xl font-medium whitespace-nowrap transition-all ${selectedCategory === key
                                ? 'bg-gradient-to-r from-indigo-500 to-purple-500 text-white shadow-lg'
                                : 'bg-muted text-foreground hover:bg-muted/80'
                                }`}
                        >
                            {icon} {label}
                        </button>
                    ))}
                </div>
            </div>

            {/* Date Range & Actions */}
            <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <div className="flex gap-2 flex-1">
                    <input
                        type="date"
                        value={dateFrom}
                        onChange={(e) => setDateFrom(e.target.value)}
                        className="flex-1 px-4 py-2 bg-muted border border-border rounded-xl"
                        placeholder="From"
                    />
                    <span className="self-center text-muted-foreground">to</span>
                    <input
                        type="date"
                        value={dateTo}
                        onChange={(e) => setDateTo(e.target.value)}
                        className="flex-1 px-4 py-2 bg-muted border border-border rounded-xl"
                        placeholder="To"
                    />
                </div>

                <div className="flex gap-2">
                    <button
                        onClick={fetchLogs}
                        disabled={isLoadingLogs}
                        className="px-4 py-2 bg-indigo-100 dark:bg-indigo-900/50 text-indigo-600 dark:text-indigo-300 rounded-xl hover:bg-indigo-200 dark:hover:bg-indigo-900 transition-colors flex items-center gap-2"
                    >
                        <RefreshCw className={`w-4 h-4 ${isLoadingLogs ? 'animate-spin' : ''}`} />
                        Refresh
                    </button>
                    <button
                        onClick={handleExport}
                        className="px-4 py-2 bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600 dark:text-emerald-300 rounded-xl hover:bg-emerald-200 dark:hover:bg-emerald-900 transition-colors flex items-center gap-2"
                    >
                        <Download className="w-4 h-4" />
                        Export
                    </button>
                </div>
            </div>
        </div>
    );
}

interface AuditLogTableProps {
    isLoadingLogs: boolean;
    error: string | null;
    logs: AuditLogEntry[];
    page: number;
    totalPages: number;
    setPage: React.Dispatch<React.SetStateAction<number>>;
    fetchLogs: () => void;
}

function AuditLogTable({
    isLoadingLogs,
    error,
    logs,
    page,
    totalPages,
    setPage,
    fetchLogs,
}: AuditLogTableProps) {
    return (
        <div className="bg-card rounded-2xl shadow-xl border border-border/20 overflow-hidden">
            {isLoadingLogs ? (
                <div className="p-8 text-center">
                    <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4 text-indigo-500" />
                    <p className="text-muted-foreground">Loading audit logs...</p>
                </div>
            ) : typeof error === 'string' ? (
                <div className="p-8 text-center">
                    <p className="text-red-500 mb-4">{error}</p>
                    <button
                        onClick={fetchLogs}
                        className="px-4 py-2 bg-indigo-500 text-white rounded-xl hover:bg-indigo-600"
                    >
                        Retry
                    </button>
                </div>
            ) : logs.length === 0 ? (
                <div className="p-8 text-center">
                    <div className="text-6xl mb-4">📭</div>
                    <p className="text-muted-foreground">No audit logs found</p>
                </div>
            ) : (
                <>
                    {/* Table Header */}
                    <div className="hidden md:grid grid-cols-12 gap-4 p-4 bg-muted/50 border-b border-border font-medium text-sm text-muted-foreground">
                        <div className="col-span-2">Time</div>
                        <div className="col-span-2">Action</div>
                        <div className="col-span-3">Actor</div>
                        <div className="col-span-3">Target</div>
                        <div className="col-span-2">Details</div>
                    </div>

                    {/* Log Entries */}
                    <div className="divide-y divide-border">
                        {logs.map((log) => (
                            <AuditLogRow key={log.id} log={log} />
                        ))}
                    </div>
                </>
            )}

            {/* Pagination */}
            {logs.length > 0 && (
                <div className="p-4 bg-muted/50 border-t border-border flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">
                        Page {page} of {totalPages}
                    </span>
                    <div className="flex gap-2">
                        <button
                            onClick={() => setPage(p => Math.max(1, p - 1))}
                            disabled={page === 1}
                            className="p-2 rounded-lg bg-card border border-border disabled:opacity-50 hover:bg-muted"
                        >
                            <ChevronLeft className="w-5 h-5" />
                        </button>
                        <button
                            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                            disabled={page === totalPages}
                            className="p-2 rounded-lg bg-card border border-border disabled:opacity-50 hover:bg-muted"
                        >
                            <ChevronRight className="w-5 h-5" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}

// Audit Log Row Component
// eslint-disable-next-line max-lines-per-function
function AuditLogRow({ log }: { log: AuditLogEntry }): React.JSX.Element {
    const [isExpanded, setIsExpanded] = useState(false);

    const getActionIcon = (action: string): string => {
        const iconMap: Record<string, string> = {
            permission: '🔐',
            wallet: '👛',
            plan: '💳',
            login: '🔑',
            auth: '🔑',
            create: '➕',
            delete: '🗑️',
            remove: '🗑️',
            update: '✏️',
            edit: '✏️',
            disable: '🚫',
            enable: '✅',
        };

        const entry = Object.entries(iconMap).find(([key]) => action.includes(key));
        return entry !== undefined ? entry[1] : '📝';
    };

    const getActionColor = (action: string): string => {
        if (action.includes('create') || action.includes('enable')) { return 'text-emerald-600 bg-emerald-100 dark:bg-emerald-900/30'; }
        if (action.includes('delete') || action.includes('disable') || action.includes('remove')) { return 'text-red-600 bg-red-100 dark:bg-red-900/30'; }
        if (action.includes('update') || action.includes('edit')) { return 'text-blue-600 bg-blue-100 dark:bg-blue-900/30'; }
        if (action.includes('permission')) { return 'text-purple-600 bg-purple-100 dark:bg-purple-900/30'; }
        return 'text-muted-foreground bg-muted';
    };

    const formatAddress = (address: string): string => {
        if (address.length > 16) {
            return `${address.slice(0, 6)}...${address.slice(-4)}`;
        }
        return address;
    };

    const formatTime = (dateStr: string): string => {
        const date = new Date(dateStr);
        const now = new Date();
        const diffMs = now.getTime() - date.getTime();
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 1) { return 'Just now'; }
        if (diffMins < 60) { return `${diffMins}m ago`; }
        if (diffHours < 24) { return `${diffHours}h ago`; }
        if (diffDays < 7) { return `${diffDays}d ago`; }
        return date.toLocaleDateString();
    };

    return (
        <div
            className="p-4 hover:bg-muted/50 transition-colors cursor-pointer"
            onClick={() => setIsExpanded(!isExpanded)}
        >
            {/* Desktop Layout */}
            <div className="hidden md:grid grid-cols-12 gap-4 items-center">
                <div className="col-span-2 text-sm text-muted-foreground">
                    {formatTime(log.timestamp)}
                </div>
                <div className="col-span-2">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(log.action)}`}>
                        {getActionIcon(log.action)} {log.action.replace(/_/g, ' ')}
                    </span>
                </div>
                <div className="col-span-3">
                    <code className="text-sm bg-muted px-2 py-1 rounded font-mono">
                        {formatAddress(log.wallet_address ?? 'System')}
                    </code>
                </div>
                <div className="col-span-3 text-sm text-muted-foreground">
                    <span className="font-medium">{log.resource_type}</span>
                    <span className="mx-1">→</span>
                    <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {formatAddress(log.resource_id ?? 'N/A')}
                    </code>
                </div>
                <div className="col-span-2 text-right">
                    <span className="text-muted-foreground text-sm">{isExpanded ? '▼' : '▶'}</span>
                </div>
            </div>

            {/* Mobile Layout */}
            <div className="md:hidden space-y-2">
                <div className="flex items-center justify-between">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(log.action)}`}>
                        {getActionIcon(log.action)} {log.action.replace(/_/g, ' ')}
                    </span>
                    <span className="text-sm text-muted-foreground">{formatTime(log.timestamp)}</span>
                </div>
                <div className="text-sm text-muted-foreground">
                    <span className="font-medium">Actor:</span>{' '}
                    <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {formatAddress(log.wallet_address ?? 'System')}
                    </code>
                </div>
            </div>

            {/* Expanded Details */}
            {isExpanded && (
                <div className="mt-4 pt-4 border-t border-border">
                    <div className="bg-muted/50 rounded-xl p-4 space-y-3">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="text-muted-foreground">Full Actor Address:</span>
                                <code className="block mt-1 bg-muted px-2 py-1 rounded text-xs font-mono break-all">
                                    {log.wallet_address ?? 'System'}
                                </code>
                            </div>
                            <div>
                                <span className="text-muted-foreground">Full Target ID:</span>
                                <code className="block mt-1 bg-muted px-2 py-1 rounded text-xs font-mono break-all">
                                    {log.resource_id ?? 'N/A'}
                                </code>
                            </div>
                            <div>
                                <span className="text-muted-foreground">Timestamp:</span>
                                <p className="mt-1">{new Date(log.timestamp).toLocaleString()}</p>
                            </div>
                            {typeof log.ip_address === 'string' && log.ip_address !== '' && (
                                <div>
                                    <span className="text-muted-foreground">IP Address:</span>
                                    <p className="mt-1">{log.ip_address}</p>
                                </div>
                            )}
                        </div>
                        {log.details && Object.keys(log.details).length > 0 && (
                            <div>
                                <span className="text-muted-foreground text-sm">Details:</span>
                                <pre className="mt-1 bg-muted px-3 py-2 rounded-lg text-xs overflow-x-auto">
                                    {JSON.stringify(log.details, null, 2)}
                                </pre>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

// Mock data generator for development
function generateMockAuditLogs(): AuditLogEntry[] {
    const actions = [
        'permission_granted', 'permission_revoked', 'wallet_created', 'wallet_disabled',
        'wallet_enabled', 'plan_created', 'plan_updated', 'user_login', 'settings_updated',
        'notification_sent', 'plan_created', 'plan_updated', 'member_added', 'member_removed'
    ] as const;

    const targetTypes = ['wallet', 'permission', 'plan', 'user', 'notification'] as const;

    return Array.from({ length: 20 }, (_, i) => ({
        id: `log-${Date.now()}-${i}`,
        action: actions[Math.floor(Math.random() * actions.length)] as string,
        wallet_address: `0x${Math.random().toString(16).slice(2, 42)}`,
        resource_type: targetTypes[Math.floor(Math.random() * targetTypes.length)] as string,
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
