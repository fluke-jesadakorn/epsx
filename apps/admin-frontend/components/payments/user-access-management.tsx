/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument, @typescript-eslint/strict-boolean-expressions */
'use client';

import { UserGroupIcon } from '@heroicons/react/24/outline';
import { useCallback, useEffect, useState } from 'react';

import { fetchUserAccessAction } from '@/app/payments/actions';
import type { PlanAccessData } from '@/shared/types/payment';

/**
 *
 */
export function UserAccessManagement() {
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

            const response = await fetchUserAccessAction(params) as any;

            if (response.success && response.data) {
                setUserAccess(response.data.users ?? []);
            } else {
                throw new Error(response.error?.message ?? response.message ?? 'Failed to load user access');
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error loading user access');
        } finally {
            setLoading(false);
        }
    }, [currentPage, itemsPerPage]);

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
                    <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" />
                    <div className="h-6 bg-muted rounded-full w-64 mx-auto" />
                </div>
                <div className="bg-card rounded-3xl h-96 border border-border/50" />
            </div>
        );
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            <div className="relative max-w-7xl mx-auto">
                {/* Action Bar */}
                <div className="flex items-center gap-3 mb-6">
                    <button
                        onClick={() => void loadUserAccess()}
                        className="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-xl font-semibold text-sm transition-all"
                    >
                        Refresh
                    </button>
                </div>

                {/* Error State */}
                {error && (
                    <div className="relative overflow-hidden rounded-2xl bg-destructive/10 p-0.5 mb-6">
                        <div className="bg-destructive/5 rounded-2xl p-4 text-destructive border border-destructive/20">
                            {error}
                        </div>
                    </div>
                )}

                {/* User Access Table */}
                <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
                    <div className="h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" />
                    <div>
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em]">
                                    All Users with Plan Access
                                </h2>
                                <div className="px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground">
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
                                                        {user.wallet_address.slice(0, 10)}...{user.wallet_address.slice(Math.max(0, user.wallet_address.length - 4))}
                                                    </div>
                                                    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(user.status)}`}>
                                                        {user.status === 'no_plan' ? 'No Plan' : user.status}
                                                    </span>
                                                </div>
                                                <div className="grid grid-cols-2 gap-3">
                                                    <div className="bg-card rounded-xl p-3 border border-border/50">
                                                        <div className="text-sm font-medium text-muted-foreground">Plan</div>
                                                        <div className="text-lg font-bold text-primary">{user.plan_name ?? 'None'}</div>
                                                    </div>
                                                    <div className="bg-card rounded-xl p-3 border border-border/50">
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
                                                                {user.wallet_address.slice(0, 10)}...{user.wallet_address.slice(Math.max(0, user.wallet_address.length - 4))}
                                                            </div>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm font-semibold text-foreground">
                                                            {user.plan_name ?? 'No Plan'}
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
