'use client';

import {
    CheckCircleIcon,
    ClockIcon,
    CurrencyDollarIcon,
    EyeIcon,
    XCircleIcon,
} from '@heroicons/react/24/outline';
import { useCallback, useEffect, useState } from 'react';

import { useApiClient } from '@/shared/hooks/useApiClient';
import type {
    PaymentResponse,
    PermissionTemplateName,
} from '@/shared/types/payment';

interface AdminPayment extends PaymentResponse {
    wallet_address: string;
    transaction_hash?: string;
    block_number?: number;
    confirmations: number;
    plan_name: string;
    plan_price: number;
    payment_reference: string;
    metadata?: Record<string, any>;
}

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

interface PaymentFilters {
    status: string;
    payment_method: string;
    date_range: string;
    plan_template: PermissionTemplateName;
    search: string;
}

/**
 *
 */
export function PaymentsManagement() {
    const { base } = useApiClient({ platform: 'admin' });
    const [payments, setPayments] = useState<AdminPayment[]>([]);
    const [stats, setStats] = useState<PaymentStats | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [filters, setFilters] = useState<PaymentFilters>({
        status: '',
        payment_method: '',
        date_range: '',
        plan_template: 'BASIC',
        search: '',
    });

    const [currentPage, setCurrentPage] = useState(1);
    const [totalPages, setTotalPages] = useState(1);
    const [itemsPerPage] = useState(20);

    const loadPayments = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const params = {
                page: currentPage.toString(),
                limit: itemsPerPage.toString(),
                ...(filters.status && { status: filters.status }),
                ...(filters.payment_method && { payment_method: filters.payment_method }),
                ...(filters.search && { search: filters.search }),
            };

            const response = await base.get<any>('/api/payments/admin/list', params);

            if (response.success && response.data) {
                setPayments(response.data.payments || []);
                setStats(response.data.summary || null);
                setTotalPages(response.data.pagination?.total_pages || 1);
            } else {
                throw new Error(response.error || response.message || 'Failed to load payments');
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error loading payments');
        } finally {
            setLoading(false);
        }
    }, [base, currentPage, itemsPerPage, filters]);

    useEffect(() => {
        loadPayments();
    }, [loadPayments]);

    // Export payments to CSV
    const exportPaymentsToCSV = useCallback(() => {
        if (!payments.length) {
            alert('No payments to export');
            return;
        }

        const headers = [
            'Date',
            'Wallet Address',
            'Plan',
            'Amount',
            'Currency',
            'Status',
            'Transaction Hash',
            'Payment Reference'
        ];

        const csvRows = [
            headers.join(','),
            ...payments.map(p => [
                new Date(p.created_at).toISOString(),
                p.wallet_address || '',
                p.plan_name || '',
                p.amount?.toString() || '',
                p.currency || 'USDT',
                p.status || '',
                p.transaction_hash || '',
                p.payment_reference || ''
            ].map(v => `"${String(v).replace(/"/g, '""')}"`).join(','))
        ];

        const csvContent = csvRows.join('\n');
        const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.setAttribute('href', url);
        link.setAttribute('download', `payments_export_${new Date().toISOString().split('T')[0]}.csv`);
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
    }, [payments]);

    const getStatusColor = (status: string) => {
        switch (status.toLowerCase()) {
            case 'succeeded':
            case 'completed':
                return 'bg-gradient-to-r from-emerald-400 to-green-500 text-white';
            case 'failed':
            case 'cancelled':
            case 'expired':
                return 'bg-gradient-to-r from-red-400 to-rose-500 text-white';
            case 'pending':
            case 'processing':
                return 'bg-gradient-to-r from-yellow-400 to-orange-500 text-white';
            default:
                return 'bg-gray-200 text-gray-700';
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status.toLowerCase()) {
            case 'succeeded':
            case 'completed':
                return <CheckCircleIcon className="w-4 h-4" />;
            case 'failed':
            case 'cancelled':
            case 'expired':
                return <XCircleIcon className="w-4 h-4" />;
            case 'pending':
            case 'processing':
                return <ClockIcon className="w-4 h-4" />;
            default:
                return <ClockIcon className="w-4 h-4" />;
        }
    };

    const formatCurrency = (amount: number, currency = 'USD') => {
        try {
            if (currency === 'USDT') {
                return new Intl.NumberFormat('en-US', {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 2,
                }).format(amount) + ' USDT';
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

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString();
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
                <div className="text-center mb-12">
                    <div className="h-16 bg-gradient-to-r from-blue-400 to-purple-500 rounded-2xl w-96 mx-auto mb-6"></div>
                    <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto"></div>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <div key={i} className="bg-gray-300 rounded-3xl h-32"></div>
                    ))}
                </div>
                <div className="bg-gray-200 rounded-3xl h-96"></div>
            </div>
        );
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-purple-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-rose-500/20 rounded-full blur-lg"></div>
                <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-emerald-400/15 to-teal-500/15 rounded-full blur-xl"></div>
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Hero Section */}
                <div className="text-center mb-8 sm:mb-12">
                    <div className="relative inline-block">
                        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent mb-4">
                            💳 Payment Transactions
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full"></div>
                    </div>
                    <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
                        Monitor and manage all payment transactions across the platform
                    </p>
                </div>

                {/* Action Cards */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-500/20 to-pink-500/20 p-0.5 cursor-pointer"
                        onClick={() => loadPayments()}
                    >
                        <div className="relative bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 text-white rounded-2xl sm:rounded-3xl">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">🔄</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload payment data from the server</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Refresh
                                </div>
                            </div>
                        </div>
                    </div>

                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-green-500/20 to-teal-500/20 p-0.5 cursor-pointer hover:opacity-90 transition-opacity"
                        onClick={exportPaymentsToCSV}
                        role="button"
                        tabIndex={0}
                        onKeyDown={(e) => e.key === 'Enter' && exportPaymentsToCSV()}
                    >
                        <div className="relative bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 text-white rounded-2xl sm:rounded-3xl">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">📊</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Export Report</h3>
                                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Download payment data as CSV</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Export CSV
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Stats Grid */}
                {stats && (
                    <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
                            <div className="flex items-center justify-between mb-3 sm:mb-4">
                                <div className="text-xl sm:text-2xl">💰</div>
                                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Revenue</span>
                            </div>
                            <div className="space-y-1">
                                <div className="text-xl sm:text-3xl font-bold text-blue-600 dark:text-blue-400 truncate">
                                    {formatCurrency(stats.total_amount)}
                                </div>
                                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Total</div>
                            </div>
                        </div>

                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-emerald-300/50 dark:border-emerald-700/50">
                            <div className="flex items-center justify-between mb-3 sm:mb-4">
                                <div className="text-xl sm:text-2xl">✅</div>
                                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Success</span>
                            </div>
                            <div className="space-y-1">
                                <div className="text-2xl sm:text-3xl font-bold text-emerald-600 dark:text-emerald-400">{stats.successful_payments}</div>
                                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Completed</div>
                            </div>
                        </div>

                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-yellow-300/50 dark:border-yellow-700/50">
                            <div className="flex items-center justify-between mb-3 sm:mb-4">
                                <div className="text-xl sm:text-2xl">⏳</div>
                                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Pending</span>
                            </div>
                            <div className="space-y-1">
                                <div className="text-2xl sm:text-3xl font-bold text-yellow-600 dark:text-yellow-400">{stats.pending_payments}</div>
                                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">In Progress</div>
                            </div>
                        </div>

                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
                            <div className="flex items-center justify-between mb-3 sm:mb-4">
                                <div className="text-xl sm:text-2xl">📈</div>
                                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Today</span>
                            </div>
                            <div className="space-y-1">
                                <div className="text-xl sm:text-3xl font-bold text-purple-600 dark:text-purple-400 truncate">
                                    {formatCurrency(stats.revenue_today)}
                                </div>
                                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Revenue</div>
                            </div>
                        </div>
                    </div>
                )}

                {/* Filter Section */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5 mb-6">
                    <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Search</label>
                                <input
                                    type="text"
                                    placeholder="Reference, wallet, hash..."
                                    value={filters.search}
                                    onChange={(e) => setFilters({ ...filters, search: e.target.value })}
                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Status</label>
                                <select
                                    value={filters.status}
                                    onChange={(e) => setFilters({ ...filters, status: e.target.value })}
                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                >
                                    <option value="">All Status</option>
                                    <option value="succeeded">Succeeded</option>
                                    <option value="pending">Pending</option>
                                    <option value="failed">Failed</option>
                                    <option value="cancelled">Cancelled</option>
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Method</label>
                                <select
                                    value={filters.payment_method}
                                    onChange={(e) => setFilters({ ...filters, payment_method: e.target.value })}
                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                >
                                    <option value="">All Methods</option>
                                    <option value="on_chain">On Chain</option>
                                    <option value="on_line">Online</option>
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Plan</label>
                                <select
                                    value={filters.plan_template}
                                    onChange={(e) => setFilters({ ...filters, plan_template: e.target.value as PermissionTemplateName })}
                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                >
                                    <option value="BASIC">Basic</option>
                                    <option value="PRO">Pro</option>
                                    <option value="ENTERPRISE">Enterprise</option>
                                    <option value="WHALE">Whale</option>
                                </select>
                            </div>

                            <div className="flex items-end">
                                <button
                                    onClick={() => {
                                        setFilters({
                                            status: '',
                                            payment_method: '',
                                            date_range: '',
                                            plan_template: 'BASIC',
                                            search: '',
                                        });
                                    }}
                                    className="w-full px-4 py-3 font-semibold rounded-xl bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                                >
                                    Reset
                                </button>
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

                {/* Payments Table */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
                    <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl overflow-hidden">
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent">
                                    Recent Transactions
                                </h2>
                                <div className="text-sm text-gray-500 dark:text-gray-400">
                                    {payments.length} payments
                                </div>
                            </div>

                            {payments.length === 0 ? (
                                <div className="text-center py-12 sm:py-16">
                                    <div className="h-20 w-20 bg-gradient-to-br from-blue-200 to-purple-200 dark:from-blue-800 dark:to-purple-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                                        <CurrencyDollarIcon className="w-10 h-10 text-blue-600 dark:text-blue-400" />
                                    </div>
                                    <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                                        No payments found
                                    </h3>
                                    <p className="text-gray-500 dark:text-gray-500">
                                        No payments match your current filters.
                                    </p>
                                </div>
                            ) : (
                                <>
                                    {/* Mobile Card Layout */}
                                    <div className="block sm:hidden space-y-4">
                                        {payments.map((payment) => (
                                            <div
                                                key={payment.id}
                                                className="p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl"
                                            >
                                                <div className="flex items-center justify-between mb-3">
                                                    <span className="font-mono text-xs text-gray-500">{payment.payment_reference}</span>
                                                    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(payment.status)}`}>
                                                        {getStatusIcon(payment.status)}
                                                        <span className="ml-1">{payment.status}</span>
                                                    </span>
                                                </div>
                                                <div className="grid grid-cols-2 gap-3">
                                                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                                                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Amount</div>
                                                        <div className="text-lg font-bold text-blue-600 dark:text-blue-400">
                                                            {formatCurrency(payment.amount, payment.currency)}
                                                        </div>
                                                    </div>
                                                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                                                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Plan</div>
                                                        <div className="text-lg font-bold text-purple-600 dark:text-purple-400">{payment.plan_name}</div>
                                                    </div>
                                                </div>
                                                <div className="mt-3 text-xs text-gray-500">{formatDate(payment.created_at)}</div>
                                            </div>
                                        ))}
                                    </div>

                                    {/* Desktop Table */}
                                    <div className="hidden sm:block overflow-x-auto">
                                        <table className="min-w-full">
                                            <thead>
                                                <tr className="border-b border-gray-200 dark:border-gray-700">
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Reference</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Wallet</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Plan</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Amount</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Created</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                                                </tr>
                                            </thead>
                                            <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
                                                {payments.map((payment) => (
                                                    <tr key={payment.id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm font-mono text-gray-900 dark:text-white">
                                                            {payment.payment_reference}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <div className="text-xs font-mono text-gray-500 dark:text-gray-400">
                                                                {payment.wallet_address || 'N/A'}
                                                            </div>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                            {payment.plan_name}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm font-semibold text-blue-600 dark:text-blue-400">
                                                            {formatCurrency(payment.amount, payment.currency)}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(payment.status)}`}>
                                                                {getStatusIcon(payment.status)}
                                                                <span className="ml-1">{payment.status}</span>
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                            {formatDate(payment.created_at)}
                                                        </td>
                                                        <td className="px-4 py-4 whitespace-nowrap">
                                                            <button className="p-2 rounded-xl text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors">
                                                                <EyeIcon className="w-5 h-5" />
                                                            </button>
                                                        </td>
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>

                                    {/* Pagination */}
                                    {totalPages > 1 && (
                                        <div className="mt-6 flex items-center justify-between border-t border-gray-200 dark:border-gray-700 pt-6">
                                            <p className="text-sm text-gray-500 dark:text-gray-400">
                                                Page <span className="font-semibold">{currentPage}</span> of <span className="font-semibold">{totalPages}</span>
                                            </p>
                                            <div className="flex gap-2">
                                                <button
                                                    onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                                                    disabled={currentPage === 1}
                                                    className="px-4 py-2 text-sm font-medium rounded-xl bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                                >
                                                    Previous
                                                </button>
                                                <button
                                                    onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
                                                    disabled={currentPage === totalPages}
                                                    className="px-4 py-2 text-sm font-medium rounded-xl bg-gradient-to-r from-blue-500 to-purple-500 text-white hover:from-blue-600 hover:to-purple-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                                >
                                                    Next
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default PaymentsManagement;
