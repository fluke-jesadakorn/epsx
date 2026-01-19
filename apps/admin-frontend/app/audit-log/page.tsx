/**
 * Audit Log Page
 * Track admin actions, permission changes, and system events
 */
'use client';

import { ChevronLeft, ChevronRight, Download, FileText, RefreshCw, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React, { useCallback, useEffect, useState } from 'react';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import { API_ROUTES } from '@/shared/config/route-constants';
import { createAdminApiClient } from '@/shared/utils/api-client';

// Types
interface AuditLogEntry {
    id: string;
    action: string;
    wallet_address: string | null;
    resource_type: string;
    resource_id: string | null;
    result: string;
    details: Record<string, unknown> | null;
    additional_data?: Record<string, unknown> | null; // For backward compatibility if needed, but we should use one
    ip_address?: string;
    user_agent?: string;
    timestamp: string; // Backend sends timestamp
    created_at?: string; // Legacy support
}

// interface AuditLogFilters {
//     action?: string;
//     actor?: string;
//     target_type?: string;
//     from_date?: string;
//     to_date?: string;
// }

type ActionType = 'all' | 'permission' | 'wallet' | 'plan' | 'system';

const ACTION_CATEGORIES: Record<ActionType, { label: string; icon: string; color: string }> = {
    all: { label: 'All Actions', icon: '📋', color: 'gray' },
    permission: { label: 'Permissions', icon: '🔐', color: 'green' },
    wallet: { label: 'Wallets', icon: '👛', color: 'blue' },
    plan: { label: 'Plans', icon: '💳', color: 'purple' },
    system: { label: 'System', icon: '⚙️', color: 'orange' },
};



function AuditLogSkeleton(): React.JSX.Element {
    return (
        <div className="min-h-screen bg-background p-6">
            <div className="max-w-7xl mx-auto">
                {/* Header skeleton */}
                <div className="mb-8">
                    <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded-xl w-64 mb-4 animate-pulse"></div>
                    <div className="h-6 bg-gray-200 dark:bg-gray-700 rounded-lg w-96 animate-pulse"></div>
                </div>

                {/* Filters skeleton */}
                <div className="grid grid-cols-1 md:grid-cols-5 gap-4 mb-6">
                    {Array.from({ length: 5 }).map((_, i) => (
                        <div key={i} className="h-12 bg-gray-200 dark:bg-gray-700 rounded-xl animate-pulse"></div>
                    ))}
                </div>

                {/* Table skeleton */}
                <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl overflow-hidden">
                    <div className="p-6 space-y-4">
                        {Array.from({ length: 10 }).map((_, i) => (
                            <div key={i} className="flex items-center gap-4 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl animate-pulse">
                                <div className="w-10 h-10 bg-gray-200 dark:bg-gray-600 rounded-full"></div>
                                <div className="flex-1 space-y-2">
                                    <div className="h-4 bg-gray-200 dark:bg-gray-600 rounded w-1/3"></div>
                                    <div className="h-3 bg-gray-200 dark:bg-gray-600 rounded w-1/2"></div>
                                </div>
                                <div className="h-4 bg-gray-200 dark:bg-gray-600 rounded w-24"></div>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
}

/**
 *
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
            const params = new URLSearchParams();

            params.set('page', page.toString());
            params.set('page_size', pageSize.toString());

            if (searchQuery) { params.set('search', searchQuery); }
            if (selectedCategory !== 'all') { params.set('category', selectedCategory); }
            if (dateFrom) { params.set('from_date', dateFrom); }
            if (dateTo) { params.set('to_date', dateTo); }

            const response = await client.get(`${API_ROUTES.ADMIN.AUDIT_LOGS}?${params.toString()}`);

            if (response.success && response.data) {
                setLogs(response.data.entries || []);
                setTotalPages(response.data.total_pages || 1);
            } else {
                // Mock data for development when backend doesn't have this endpoint yet
                setLogs(generateMockAuditLogs());
                setTotalPages(5);
            }
        } catch (err) {
            // Only log errors that aren't 404s (expected when endpoint not yet implemented)
            if (err && typeof err === 'object' && 'status' in err && err.status !== 404) {
                console.error('Failed to fetch audit logs:', err);
            }
            // Use mock data when API is not available
            setLogs(generateMockAuditLogs());
            setTotalPages(5);
        } finally {
            setIsLoadingLogs(false);
        }
    }, [isAuthenticated, page, searchQuery, selectedCategory, dateFrom, dateTo]);

    useEffect(() => {
        fetchLogs();
    }, [fetchLogs]);

    // Export logs
    const handleExport = async (): Promise<void> => {
        const csvContent = [
            ['Date', 'Action', 'Actor', 'Target', 'Details'].join(','),
            ...logs.map(log => [
                new Date(log.timestamp).toISOString(),
                log.action,
                log.wallet_address || '',
                `${log.resource_type}:${log.resource_id || ''}`,
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
        return <AuditLogSkeleton />;
    }

    if (!isAuthenticated) {
        return <AuditLogSkeleton />;
    }

    return (
        <div className="min-h-screen bg-background p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-indigo-400/20 to-purple-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-emerald-400/20 to-teal-500/20 rounded-full blur-lg"></div>
                <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Header */}
                <div className="mb-8 text-center sm:text-left">
                    <div className="relative inline-block">
                        <h1 className="flex items-center justify-center sm:justify-start gap-3 text-4xl sm:text-5xl font-bold mb-4">
                            <span className="text-indigo-500">
                                <FileText className="w-10 h-10 sm:w-12 sm:h-12" />
                            </span>
                            <span className="bg-gradient-to-r from-indigo-600 via-purple-600 to-pink-600 bg-clip-text text-transparent">
                                Audit Log
                            </span>
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-indigo-400 to-purple-500 rounded-full"></div>
                    </div>
                    <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl">
                        Track all admin actions, permission changes, and system events
                    </p>
                </div>

                {/* Filters Bar */}
                <div className="bg-white dark:bg-gray-800 rounded-2xl p-4 mb-6 shadow-xl border border-white/20">
                    <div className="flex flex-col lg:flex-row gap-4">
                        {/* Search */}
                        <div className="relative flex-1">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                            <input
                                type="text"
                                placeholder="Search by actor, action, or target..."
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                className="w-full pl-10 pr-4 py-3 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
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
                                        : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
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
                                className="flex-1 px-4 py-2 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-xl"
                                placeholder="From"
                            />
                            <span className="self-center text-gray-400">to</span>
                            <input
                                type="date"
                                value={dateTo}
                                onChange={(e) => setDateTo(e.target.value)}
                                className="flex-1 px-4 py-2 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-xl"
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

                {/* Audit Log Table */}
                <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-white/20 overflow-hidden">
                    {isLoadingLogs ? (
                        <div className="p-8 text-center">
                            <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4 text-indigo-500" />
                            <p className="text-gray-500">Loading audit logs...</p>
                        </div>
                    ) : error ? (
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
                            <p className="text-gray-500">No audit logs found</p>
                        </div>
                    ) : (
                        <>
                            {/* Table Header */}
                            <div className="hidden md:grid grid-cols-12 gap-4 p-4 bg-gray-50 dark:bg-gray-700/50 border-b border-gray-200 dark:border-gray-700 font-medium text-sm text-gray-600 dark:text-gray-400">
                                <div className="col-span-2">Time</div>
                                <div className="col-span-2">Action</div>
                                <div className="col-span-3">Actor</div>
                                <div className="col-span-3">Target</div>
                                <div className="col-span-2">Details</div>
                            </div>

                            {/* Log Entries */}
                            <div className="divide-y divide-gray-100 dark:divide-gray-700">
                                {logs.map((log) => (
                                    <AuditLogRow key={log.id} log={log} />
                                ))}
                            </div>
                        </>
                    )}

                    {/* Pagination */}
                    {logs.length > 0 && (
                        <div className="p-4 bg-gray-50 dark:bg-gray-700/50 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
                            <span className="text-sm text-gray-500">
                                Page {page} of {totalPages}
                            </span>
                            <div className="flex gap-2">
                                <button
                                    onClick={() => setPage(p => Math.max(1, p - 1))}
                                    disabled={page === 1}
                                    className="p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 disabled:opacity-50 hover:bg-gray-50 dark:hover:bg-gray-700"
                                >
                                    <ChevronLeft className="w-5 h-5" />
                                </button>
                                <button
                                    onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                                    disabled={page === totalPages}
                                    className="p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 disabled:opacity-50 hover:bg-gray-50 dark:hover:bg-gray-700"
                                >
                                    <ChevronRight className="w-5 h-5" />
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

// Audit Log Row Component
function AuditLogRow({ log }: { log: AuditLogEntry }): React.JSX.Element {
    const [isExpanded, setIsExpanded] = useState(false);

    const getActionIcon = (action: string): string => {
        if (action.includes('permission')) { return '🔐'; }
        if (action.includes('wallet')) { return '👛'; }
        if (action.includes('plan')) { return '💳'; }
        if (action.includes('login') || action.includes('auth')) { return '🔑'; }
        if (action.includes('create')) { return '➕'; }
        if (action.includes('delete') || action.includes('remove')) { return '🗑️'; }
        if (action.includes('update') || action.includes('edit')) { return '✏️'; }
        if (action.includes('disable')) { return '🚫'; }
        if (action.includes('enable')) { return '✅'; }
        return '📝';
    };

    const getActionColor = (action: string): string => {
        if (action.includes('create') || action.includes('enable')) { return 'text-emerald-600 bg-emerald-100 dark:bg-emerald-900/30'; }
        if (action.includes('delete') || action.includes('disable') || action.includes('remove')) { return 'text-red-600 bg-red-100 dark:bg-red-900/30'; }
        if (action.includes('update') || action.includes('edit')) { return 'text-blue-600 bg-blue-100 dark:bg-blue-900/30'; }
        if (action.includes('permission')) { return 'text-purple-600 bg-purple-100 dark:bg-purple-900/30'; }
        return 'text-gray-600 bg-gray-100 dark:bg-gray-700';
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
            className="p-4 hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors cursor-pointer"
            onClick={() => setIsExpanded(!isExpanded)}
        >
            {/* Desktop Layout */}
            <div className="hidden md:grid grid-cols-12 gap-4 items-center">
                <div className="col-span-2 text-sm text-gray-500">
                    {formatTime(log.timestamp)}
                </div>
                <div className="col-span-2">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(log.action)}`}>
                        {getActionIcon(log.action)} {log.action.replace(/_/g, ' ')}
                    </span>
                </div>
                <div className="col-span-3">
                    <code className="text-sm bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded font-mono">
                        {formatAddress(log.wallet_address || 'System')}
                    </code>
                </div>
                <div className="col-span-3 text-sm text-gray-600 dark:text-gray-400">
                    <span className="font-medium">{log.resource_type}</span>
                    <span className="mx-1">→</span>
                    <code className="bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded text-xs">
                        {formatAddress(log.resource_id || 'N/A')}
                    </code>
                </div>
                <div className="col-span-2 text-right">
                    <span className="text-gray-400 text-sm">{isExpanded ? '▼' : '▶'}</span>
                </div>
            </div>

            {/* Mobile Layout */}
            <div className="md:hidden space-y-2">
                <div className="flex items-center justify-between">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(log.action)}`}>
                        {getActionIcon(log.action)} {log.action.replace(/_/g, ' ')}
                    </span>
                    <span className="text-sm text-gray-500">{formatTime(log.timestamp)}</span>
                </div>
                <div className="text-sm text-gray-600 dark:text-gray-400">
                    <span className="font-medium">Actor:</span>{' '}
                    <code className="bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded text-xs">
                        {formatAddress(log.wallet_address || 'System')}
                    </code>
                </div>
            </div>

            {/* Expanded Details */}
            {isExpanded && (
                <div className="mt-4 pt-4 border-t border-gray-100 dark:border-gray-700">
                    <div className="bg-gray-50 dark:bg-gray-700/50 rounded-xl p-4 space-y-3">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="text-gray-500 dark:text-gray-400">Full Actor Address:</span>
                                <code className="block mt-1 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs font-mono break-all">
                                    {log.wallet_address || 'System'}
                                </code>
                            </div>
                            <div>
                                <span className="text-gray-500 dark:text-gray-400">Full Target ID:</span>
                                <code className="block mt-1 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs font-mono break-all">
                                    {log.resource_id || 'N/A'}
                                </code>
                            </div>
                            <div>
                                <span className="text-gray-500 dark:text-gray-400">Timestamp:</span>
                                <p className="mt-1">{new Date(log.timestamp).toLocaleString()}</p>
                            </div>
                            {log.ip_address && (
                                <div>
                                    <span className="text-gray-500 dark:text-gray-400">IP Address:</span>
                                    <p className="mt-1">{log.ip_address}</p>
                                </div>
                            )}
                        </div>
                        {log.details && Object.keys(log.details).length > 0 && (
                            <div>
                                <span className="text-gray-500 dark:text-gray-400 text-sm">Details:</span>
                                <pre className="mt-1 bg-gray-100 dark:bg-gray-700 px-3 py-2 rounded-lg text-xs overflow-x-auto">
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
        'notification_sent', 'group_created', 'member_added', 'member_removed'
    ] as const;

    const targetTypes = ['wallet', 'permission', 'plan', 'user', 'group', 'notification'] as const;

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

