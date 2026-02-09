'use client';

import { ChartBarIcon } from '@heroicons/react/24/outline';
import { useCallback, useEffect, useState } from 'react';

import { useApiClient } from '@/shared/hooks/use-api-client';

interface PaymentStats {
    total_payments: number;
    total_amount: number;
    successful_payments: number;
    failed_payments: number;
    pending_payments: number;
    average_payment_amount: number;
    payments_today: number;
    revenue_today: number;
}

/**
 *
 */
export function PaymentAnalytics() {
    const { base } = useApiClient({ platform: 'admin' });
    const [stats, setStats] = useState<PaymentStats | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const loadAnalytics = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const response = await base.get<any>('/api/payments/admin/analytics');

            if (response.success && response.data) {
                setStats(response.data.analytics?.summary ?? null);
            } else {
                throw new Error(response.error ?? response.message ?? 'Failed to load analytics');
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error loading analytics');
        } finally {
            setLoading(false);
        }
    }, [base]);

    useEffect(() => {
        loadAnalytics();
    }, [loadAnalytics]);

    const formatCurrency = (amount: number, currency = 'USD') => {
        try {
            if (currency === 'USDT') {
                return `${new Intl.NumberFormat('en-US', {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 2,
                }).format(amount)  } USDT`;
            }

            return new Intl.NumberFormat('en-US', {
                style: 'currency',
                currency,
            }).format(amount);
        } catch (e) {
            return `${new Intl.NumberFormat('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
            }).format(amount)} ${currency}`;
        }
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
                <div className="text-center mb-12">
                    <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" />
                    <div className="h-6 bg-muted rounded-full w-64 mx-auto" />
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="bg-card rounded-3xl h-64 border border-border/50" />
                    <div className="bg-card rounded-3xl h-64 border border-border/50" />
                </div>
            </div>
        );
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-indigo-400/20 to-purple-500/20 rounded-full blur-xl" />
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-violet-400/20 to-fuchsia-500/20 rounded-full blur-lg" />
                <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-blue-400/15 to-indigo-500/15 rounded-full blur-xl" />
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Hero Section */}
                <div className="text-center mb-8 sm:mb-12">
                    <div className="relative inline-block">
                        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent mb-4">
                            📊 Payment Analytics
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary/20 rounded-full animate-ping" />
                    </div>
                    <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
                        Comprehensive insights into your payment performance and revenue
                    </p>
                </div>

                {/* Action Card */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5 cursor-pointer"
                        onClick={() => loadAnalytics()}
                    >
                        <div className="relative bg-primary text-primary-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">🔄</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Analytics</h3>
                                <p className="text-primary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload analytics data from the server</p>
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
                                    <span className="text-xl sm:text-2xl">📈</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Export Report</h3>
                                <p className="text-secondary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Download detailed analytics report</p>
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

                {stats ? (
                    <div className="space-y-6">
                        {/* Main Stats Grid */}
                        <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6">
                            <div className="bg-card rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-primary/20">
                                <div className="flex items-center justify-between mb-3 sm:mb-4">
                                    <div className="text-xl sm:text-2xl">💰</div>
                                    <span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-xl sm:text-3xl font-bold text-primary truncate">
                                        {formatCurrency(stats.total_amount)}
                                    </div>
                                    <div className="text-xs sm:text-sm text-muted-foreground">Revenue</div>
                                </div>
                            </div>

                            <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-success/20">
                                <div className="flex items-center justify-between mb-3 sm:mb-4">
                                    <div className="text-xl sm:text-2xl">✅</div>
                                    <span className="text-xs sm:text-sm font-medium text-muted-foreground">Success</span>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-2xl sm:text-3xl font-bold text-success">{stats.successful_payments}</div>
                                    <div className="text-xs sm:text-sm text-muted-foreground">Transactions</div>
                                </div>
                            </div>

                            <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-warning/20">
                                <div className="flex items-center justify-between mb-3 sm:mb-4">
                                    <div className="text-xl sm:text-2xl">⏳</div>
                                    <span className="text-xs sm:text-sm font-medium text-muted-foreground">Pending</span>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-2xl sm:text-3xl font-bold text-warning">{stats.pending_payments}</div>
                                    <div className="text-xs sm:text-sm text-muted-foreground">Awaiting</div>
                                </div>
                            </div>

                            <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-destructive/20">
                                <div className="flex items-center justify-between mb-3 sm:mb-4">
                                    <div className="text-xl sm:text-2xl">❌</div>
                                    <span className="text-xs sm:text-sm font-medium text-muted-foreground">Failed</span>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-2xl sm:text-3xl font-bold text-destructive">{stats.failed_payments}</div>
                                    <div className="text-xs sm:text-sm text-muted-foreground">Rejected</div>
                                </div>
                            </div>
                        </div>

                        {/* Detailed Analytics */}
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            {/* Transaction Overview */}
                            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5">
                                <div className="relative bg-card rounded-2xl sm:rounded-3xl p-6 sm:p-8 border border-border/50">
                                    <h3 className="text-xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent mb-6">
                                        Transaction Overview
                                    </h3>
                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="bg-primary/5 rounded-2xl p-4 border border-primary/10">
                                            <div className="text-sm font-medium text-primary mb-2">Total Transactions</div>
                                            <div className="text-3xl font-bold text-foreground">{stats.total_payments}</div>
                                        </div>
                                        <div className="bg-success/5 rounded-2xl p-4 border border-success/10">
                                            <div className="text-sm font-medium text-success mb-2">Success Rate</div>
                                            <div className="text-3xl font-bold text-foreground">
                                                {stats.total_payments > 0 ? Math.round((stats.successful_payments / stats.total_payments) * 100) : 0}%
                                            </div>
                                        </div>
                                        <div className="bg-warning/5 rounded-2xl p-4 border border-warning/10">
                                            <div className="text-sm font-medium text-warning mb-2">Pending</div>
                                            <div className="text-3xl font-bold text-foreground">{stats.pending_payments}</div>
                                        </div>
                                        <div className="bg-destructive/5 rounded-2xl p-4 border border-destructive/10">
                                            <div className="text-sm font-medium text-destructive mb-2">Failed</div>
                                            <div className="text-3xl font-bold text-foreground">{stats.failed_payments}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {/* Revenue Overview */}
                            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5">
                                <div className="relative bg-card rounded-2xl sm:rounded-3xl p-6 sm:p-8 border border-border/50">
                                    <h3 className="text-xl font-bold bg-gradient-to-r from-secondary via-primary to-secondary bg-clip-text text-transparent mb-6">
                                        Revenue Overview
                                    </h3>
                                    <div className="space-y-4">
                                        <div className="flex items-center justify-between p-4 bg-primary/5 rounded-2xl border border-primary/10">
                                            <span className="text-sm font-medium text-muted-foreground">Total Revenue</span>
                                            <span className="text-2xl font-bold text-primary">{formatCurrency(stats.total_amount)}</span>
                                        </div>
                                        <div className="flex items-center justify-between p-4 bg-success/5 rounded-2xl border border-success/10">
                                            <span className="text-sm font-medium text-muted-foreground">Revenue Today</span>
                                            <span className="text-xl font-bold text-success">{formatCurrency(stats.revenue_today)}</span>
                                        </div>
                                        <div className="flex items-center justify-between p-4 bg-secondary/5 rounded-2xl border border-secondary/10">
                                            <span className="text-sm font-medium text-muted-foreground">Avg. Transaction</span>
                                            <span className="text-xl font-bold text-secondary">{formatCurrency(stats.average_payment_amount)}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Coming Soon Section */}
                        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-muted/20 p-0.5">
                            <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-6 sm:p-8 border border-border/50">
                                <div className="flex items-center gap-3 mb-4">
                                    <ChartBarIcon className="w-6 h-6 text-primary" />
                                    <h3 className="text-xl font-bold text-foreground">
                                        Historical Analytics
                                    </h3>
                                </div>
                                <p className="text-muted-foreground">
                                    Detailed historical charts and trend analysis coming soon. This will include revenue trends,
                                    transaction volume over time, and conversion metrics.
                                </p>
                            </div>
                        </div>
                    </div>
                ) : (
                    <div className="text-center py-12 sm:py-16">
                        <div className="h-20 w-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                            <ChartBarIcon className="w-10 h-10 text-primary" />
                        </div>
                        <h3 className="text-xl font-semibold text-foreground mb-2">
                            No analytics data available
                        </h3>
                        <p className="text-muted-foreground">
                            Analytics data will appear once payments are processed
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}

export default PaymentAnalytics;
