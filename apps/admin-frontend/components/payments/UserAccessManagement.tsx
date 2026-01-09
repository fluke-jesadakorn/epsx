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
                return 'bg-gradient-to-r from-emerald-400 to-green-500 text-white';
            case 'expiring_soon':
                return 'bg-gradient-to-r from-yellow-400 to-orange-500 text-white';
            case 'expired':
                return 'bg-gradient-to-r from-red-400 to-rose-500 text-white';
            default:
                return 'bg-gray-200 text-gray-700';
        }
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
                <div className="text-center mb-12">
                    <div className="h-16 bg-gradient-to-r from-emerald-400 to-teal-500 rounded-2xl w-96 mx-auto mb-6"></div>
                    <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto"></div>
                </div>
                <div className="bg-gray-200 rounded-3xl h-96"></div>
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
                        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-emerald-600 via-teal-600 to-green-600 bg-clip-text text-transparent mb-4">
                            👥 User Plan Access
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-emerald-400 to-teal-500 rounded-full"></div>
                    </div>
                    <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
                        View and manage user subscription access across all plans
                    </p>
                </div>

                {/* Action Card */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-teal-500/20 to-green-500/20 p-0.5 cursor-pointer"
                        onClick={() => loadUserAccess()}
                    >
                        <div className="relative bg-gradient-to-br from-emerald-400 via-teal-500 to-green-500 text-white rounded-2xl sm:rounded-3xl">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">🔄</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload user access data from the server</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Refresh
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-500/20 to-purple-500/20 p-0.5">
                        <div className="relative bg-gradient-to-br from-blue-400 via-indigo-500 to-purple-500 text-white rounded-2xl sm:rounded-3xl">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">📊</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Export Report</h3>
                                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Download user access data as CSV</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Export
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Error State */}
                {error && (
                    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-red-400/20 to-rose-400/20 p-0.5 mb-6">
                        <div className="bg-red-50 dark:bg-red-900/30 backdrop-blur-xl rounded-2xl p-4 text-red-700 dark:text-red-300">
                            {error}
                        </div>
                    </div>
                )}

                {/* User Access Table */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-teal-400/20 to-green-400/20 p-0.5">
                    <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl overflow-hidden">
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-emerald-600 via-teal-600 to-green-600 bg-clip-text text-transparent">
                                    All Users with Plan Access
                                </h2>
                                <div className="text-sm text-gray-500 dark:text-gray-400">
                                    {userAccess.length} users
                                </div>
                            </div>

                            {userAccess.length === 0 ? (
                                <div className="text-center py-12 sm:py-16">
                                    <div className="h-20 w-20 bg-gradient-to-br from-emerald-200 to-teal-200 dark:from-emerald-800 dark:to-teal-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                                        <UserGroupIcon className="w-10 h-10 text-emerald-600 dark:text-emerald-400" />
                                    </div>
                                    <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                                        No users with plan access found
                                    </h3>
                                    <p className="text-gray-500 dark:text-gray-500">
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
                                                className="p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl"
                                            >
                                                <div className="flex items-center justify-between mb-3">
                                                    <div className="font-mono text-xs text-gray-500" title={user.wallet_address}>
                                                        {user.wallet_address.substring(0, 10)}...{user.wallet_address.substring(user.wallet_address.length - 4)}
                                                    </div>
                                                    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(user.status)}`}>
                                                        {user.status === 'no_plan' ? 'No Plan' : user.status}
                                                    </span>
                                                </div>
                                                <div className="grid grid-cols-2 gap-3">
                                                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                                                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Plan</div>
                                                        <div className="text-lg font-bold text-emerald-600 dark:text-emerald-400">{user.plan_name || 'None'}</div>
                                                    </div>
                                                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                                                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Days Left</div>
                                                        <div className="text-lg font-bold text-teal-600 dark:text-teal-400">
                                                            {user.days_remaining > 0 ? `${user.days_remaining} days` : '-'}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div className="mt-3 text-xs text-gray-500">
                                                    Expires: {user.plan_expires_at ? formatDate(user.plan_expires_at) : 'Never'}
                                                </div>
                                            </div>
                                        ))}
                                    </div>

                                    {/* Desktop Table */}
                                    <div className="hidden sm:block overflow-x-auto">
                                        <table className="min-w-full">
                                            <thead>
                                                <tr className="border-b border-gray-200 dark:border-gray-700">
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Wallet</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Plan</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Days Left</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Expires</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                                                </tr>
                                            </thead>
                                            <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
                                                {userAccess.map((user) => (
                                                    <tr key={user.wallet_address} className="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <div className="text-xs font-mono text-gray-500 dark:text-gray-400" title={user.wallet_address}>
                                                                {user.wallet_address.substring(0, 10)}...{user.wallet_address.substring(user.wallet_address.length - 4)}
                                                            </div>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm font-semibold text-gray-900 dark:text-white">
                                                            {user.plan_name || 'No Plan'}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(user.status)}`}>
                                                                {user.status === 'no_plan' ? 'No Plan' : user.status}
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                            {user.days_remaining > 0 ? `${user.days_remaining} days` : '-'}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                            {user.plan_expires_at ? formatDate(user.plan_expires_at) : 'Never'}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <button className="px-3 py-1.5 text-sm font-medium rounded-xl text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-900/30 transition-colors">
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
