'use client';

import { UserGroupIcon } from '@heroicons/react/24/outline';
import { useCallback, useEffect, useState } from 'react';

import { useApiClient } from '@/shared/hooks/useApiClient';
import type { PlanAccessData } from '@/shared/types/payment';

/**
 *
 */
export function UserAccessManagement() {
    const { base } = useApiClient({ platform: 'admin' });
    const [userAccess, setUserAccess] = useState<PlanAccessData[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [currentPage, setCurrentPage] = useState(1);
    const [itemsPerPage] = useState(20);

    const loadUserAccess = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const params = {
                page: currentPage.toString(),
                limit: itemsPerPage.toString(),
            };

            const response = await base.get<any>('/api/admin/plans/user-access/list', params);

            if (response.success && response.data) {
                setUserAccess(response.data.users || []);
            } else {
                throw new Error(response.error || response.message || 'Failed to load user access');
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error loading user access');
        } finally {
            setLoading(false);
        }
    }, [base, currentPage, itemsPerPage]);

    useEffect(() => {
        loadUserAccess();
    }, [loadUserAccess]);

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString();
    };

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'active':
                return 'bg-success/10 text-success border border-success/20';
            case 'expiring_soon':
                return 'bg-warning/10 text-warning border border-warning/20';
            case 'expired':
                return 'bg-destructive/10 text-destructive border border-destructive/20';
            default:
                return 'bg-muted text-muted-foreground border border-border/50';
        }
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
                <div className="text-center mb-12">
                    <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6"></div>
                    <div className="h-6 bg-muted rounded-full w-64 mx-auto"></div>
                </div>
                <div className="bg-card rounded-3xl h-96 border border-border/50"></div>
            </div>
        );
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-emerald-400/20 to-teal-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-green-400/20 to-emerald-500/20 rounded-full blur-lg"></div>
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Hero Section */}
                <div className="text-center mb-8 sm:mb-12">
                    <div className="relative inline-block">
                        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent mb-4">
                            👥 User Plan Access
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary/20 rounded-full animate-ping"></div>
                    </div>
                    <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
                        View and manage user subscription access across all plans
                    </p>
                </div>

                {/* Action Card */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5 cursor-pointer"
                        onClick={() => loadUserAccess()}
                    >
                        <div className="relative bg-primary text-primary-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">🔄</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                                <p className="text-primary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload user access data from the server</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Refresh
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5 cursor-pointer">
                        <div className="relative bg-secondary text-secondary-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">📊</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Export Report</h3>
                                <p className="text-secondary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Download user access data as CSV</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Export
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Error State */}
                {error && (
                    <div className="relative overflow-hidden rounded-2xl bg-destructive/10 p-0.5 mb-6">
                        <div className="bg-destructive/5 backdrop-blur-xl rounded-2xl p-4 text-destructive border border-destructive/20">
                            {error}
                        </div>
                    </div>
                )}

                {/* User Access Table */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5">
                    <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl overflow-hidden border border-border/50">
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                                    All Users with Plan Access
                                </h2>
                                <div className="text-sm text-muted-foreground">
                                    {userAccess.length} users
                                </div>
                            </div>

                            {userAccess.length === 0 ? (
                                <div className="text-center py-12 sm:py-16">
                                    <div className="h-20 w-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                                        <UserGroupIcon className="w-10 h-10 text-primary" />
                                    </div>
                                    <h3 className="text-xl font-semibold text-foreground mb-2">
                                        No users with plan access found
                                    </h3>
                                    <p className="text-muted-foreground">
                                        Users with active subscriptions will appear here
                                    </p>
                                </div>
                            ) : (
                                <>
                                    {/* Mobile Card Layout */}
                                    <div className="block sm:hidden space-y-4">
                                        {userAccess.map((user) => (
                                            <div
                                                key={user.wallet_address}
                                                className="p-4 bg-muted/30 border border-border/50 rounded-2xl"
                                            >
                                                <div className="flex items-center justify-between mb-3">
                                                    <div className="font-mono text-xs text-muted-foreground" title={user.wallet_address}>
                                                        {user.wallet_address.substring(0, 10)}...{user.wallet_address.substring(user.wallet_address.length - 4)}
                                                    </div>
                                                    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(user.status)}`}>
                                                        {user.status === 'no_plan' ? 'No Plan' : user.status}
                                                    </span>
                                                </div>
                                                <div className="grid grid-cols-2 gap-3">
                                                    <div className="bg-card/50 rounded-xl p-3 border border-border/50">
                                                        <div className="text-sm font-medium text-muted-foreground">Plan</div>
                                                        <div className="text-lg font-bold text-primary">{user.plan_name || 'None'}</div>
                                                    </div>
                                                    <div className="bg-card/50 rounded-xl p-3 border border-border/50">
                                                        <div className="text-sm font-medium text-muted-foreground">Days Left</div>
                                                        <div className="text-lg font-bold text-secondary">
                                                            {user.days_remaining > 0 ? `${user.days_remaining} days` : '-'}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div className="mt-3 text-xs text-muted-foreground">
                                                    Expires: {user.plan_expires_at ? formatDate(user.plan_expires_at) : 'Never'}
                                                </div>
                                            </div>
                                        ))}
                                    </div>

                                    {/* Desktop Table */}
                                    <div className="hidden sm:block overflow-x-auto">
                                        <table className="min-w-full">
                                            <thead>
                                                <tr className="border-b border-border/50">
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Wallet</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Plan</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Status</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Days Left</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Expires</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Actions</th>
                                                </tr>
                                            </thead>
                                            <tbody className="divide-y divide-border/50">
                                                {userAccess.map((user) => (
                                                    <tr key={user.wallet_address} className="hover:bg-muted/30 transition-colors">
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <div className="text-xs font-mono text-muted-foreground" title={user.wallet_address}>
                                                                {user.wallet_address.substring(0, 10)}...{user.wallet_address.substring(user.wallet_address.length - 4)}
                                                            </div>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm font-semibold text-foreground">
                                                            {user.plan_name || 'No Plan'}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(user.status)}`}>
                                                                {user.status === 'no_plan' ? 'No Plan' : user.status}
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm text-secondary">
                                                            {user.days_remaining > 0 ? `${user.days_remaining} days` : '-'}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm text-muted-foreground">
                                                            {user.plan_expires_at ? formatDate(user.plan_expires_at) : 'Never'}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <button className="px-3 py-1.5 text-sm font-semibold rounded-xl text-primary hover:bg-primary/10 border border-primary/20 transition-all">
                                                                View
                                                            </button>
                                                        </td>
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default UserAccessManagement;
