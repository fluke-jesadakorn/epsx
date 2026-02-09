/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-argument, @typescript-eslint/strict-boolean-expressions */
'use client';

import {
    BarChart3,
    CheckCircle,
    Clock,
    CreditCard,
    DollarSign,
    Eye,
    RefreshCcw,
    Search,
    XCircle
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { useApiClient } from '@/shared/hooks/use-api-client';
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
                setPayments(response.data.payments ?? []);
                setStats(response.data.summary ?? null);
                setTotalPages(response.data.pagination?.total_pages ?? 1);
            } else {
                const resp = response as any;
                const errorMessage = typeof resp.error === 'object'
                    ? resp.error.message ?? JSON.stringify(resp.error)
                    : resp.error ?? resp.message ?? 'Failed to load payments';
                throw new Error(errorMessage);
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
                p.wallet_address ?? '',
                p.plan_name ?? '',
                p.amount?.toString() ?? '',
                p.currency ?? 'USDT',
                p.status ?? '',
                p.transaction_hash ?? '',
                p.payment_reference ?? ''
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
                return 'bg-success/10 text-success border border-success/20';
            case 'failed':
            case 'cancelled':
            case 'expired':
                return 'bg-destructive/10 text-destructive border border-destructive/20';
            case 'pending':
            case 'processing':
                return 'bg-warning/10 text-warning border border-warning/20';
            default:
                return 'bg-muted text-muted-foreground border border-border/50';
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status.toLowerCase()) {
            case 'succeeded':
            case 'completed':
                return <CheckCircle className="w-4 h-4" />;
            case 'failed':
            case 'cancelled':
            case 'expired':
                return <XCircle className="w-4 h-4" />;
            case 'pending':
            case 'processing':
                return <Clock className="w-4 h-4" />;
            default:
                return <Clock className="w-4 h-4" />;
        }
    };

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

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString();
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
                <div className="text-center mb-12">
                    <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" />
                    <div className="h-6 bg-muted rounded-full w-64 mx-auto" />
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <div key={i} className="bg-card rounded-3xl h-32 border border-border/50" />
                    ))}
                </div>
                <div className="bg-card rounded-3xl h-96 border border-border/50" />
            </div>
        );
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-primary/10 rounded-full blur-xl animate-pulse" />
                <div className="absolute top-40 right-32 w-24 h-24 bg-secondary/10 rounded-full blur-lg animate-pulse delay-700" />
                <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-primary/5 rounded-full blur-xl animate-pulse delay-1000" />
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Hero Section */}
                <div className="text-center mb-12">
                    <div className="inline-flex p-4 bg-white/5 rounded-[32px] border border-white/5 text-[#1fc7d4] mb-8 shadow-xl">
                        <CreditCard className="w-12 h-12" />
                    </div>
                    <h1 className="text-4xl sm:text-6xl font-black bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4] bg-clip-text text-transparent mb-4 tracking-tighter">
                        Payments Hub
                    </h1>
                    <p className="text-lg font-bold text-muted-foreground max-w-2xl mx-auto">
                        Manage payments, user access, and payment links
                    </p>
                </div>

                {/* Action Cards */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6 mb-12">
                    <div
                        className="group relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-8 shadow-xl transition-all duration-300 hover:border-[#1fc7d4]/30 cursor-pointer text-center"
                        onClick={() => loadPayments()}
                    >
                        <div className="absolute -right-8 -top-8 w-32 h-32 bg-[#1fc7d4]/5 rounded-full blur-3xl group-hover:bg-[#1fc7d4]/10 transition-colors" />
                        <div className="relative z-10 flex flex-col items-center">
                            <div className="p-4 bg-[#1fc7d4]/10 rounded-[24px] border border-[#1fc7d4]/10 text-[#1fc7d4] mb-6">
                                <RefreshCcw className="w-8 h-8 group-hover:rotate-180 transition-transform duration-500" />
                            </div>
                            <h3 className="text-2xl font-black text-foreground uppercase tracking-tight mb-2">Refresh Data</h3>
                            <p className="text-muted-foreground font-bold mb-8">Reload payment data from the server</p>
                            <button className="w-full max-w-[200px] py-3 rounded-2xl bg-slate-800 hover:bg-slate-700 text-white font-black text-sm uppercase tracking-widest transition-all">
                                Refresh Now
                            </button>
                        </div>
                    </div>

                    <div
                        className="group relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-8 shadow-xl transition-all duration-300 hover:border-[#7645d9]/30 cursor-pointer text-center"
                        onClick={exportPaymentsToCSV}
                    >
                        <div className="absolute -right-8 -top-8 w-32 h-32 bg-[#7645d9]/5 rounded-full blur-3xl group-hover:bg-[#7645d9]/10 transition-colors" />
                        <div className="relative z-10 flex flex-col items-center">
                            <div className="p-4 bg-[#7645d9]/10 rounded-[24px] border border-[#7645d9]/10 text-[#7645d9] mb-6">
                                <BarChart3 className="w-8 h-8" />
                            </div>
                            <h3 className="text-2xl font-black text-foreground uppercase tracking-tight mb-2">Export Analysis</h3>
                            <p className="text-muted-foreground font-bold mb-8">Download detailed report in CSV format</p>
                            <button className="w-full max-w-[200px] py-3 rounded-2xl bg-slate-800 hover:bg-slate-700 text-white font-black text-sm uppercase tracking-widest transition-all">
                                Export CSV Files
                            </button>
                        </div>
                    </div>
                </div>

                {/* Stats Grid */}
                {stats && (
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
                        <div className="group relative bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] p-8 shadow-xl transition-all duration-300 hover:border-[#1fc7d4]/30 overflow-hidden text-center">
                            <div className="absolute -right-6 -top-6 w-20 h-20 bg-[#1fc7d4]/5 rounded-full blur-3xl group-hover:bg-[#1fc7d4]/10 transition-colors" />
                            <div className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-4">Total Revenue</div>
                            <div className="text-3xl sm:text-4xl font-black bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent tracking-tighter mb-2">
                                {formatCurrency(stats.total_amount)}
                            </div>
                            <div className="text-sm font-bold text-muted-foreground/60">Platform Total</div>
                        </div>

                        <div className="group relative bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] p-8 shadow-xl transition-all duration-300 hover:border-[#31d0aa]/30 overflow-hidden text-center">
                            <div className="absolute -right-6 -top-6 w-20 h-20 bg-[#31d0aa]/5 rounded-full blur-3xl group-hover:bg-[#31d0aa]/10 transition-colors" />
                            <div className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-4">Successful</div>
                            <div className="text-3xl sm:text-4xl font-black text-[#31d0aa] tracking-tighter mb-2">
                                {stats.successful_payments}
                            </div>
                            <div className="text-sm font-bold text-muted-foreground/60">Completed</div>
                        </div>

                        <div className="group relative bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] p-8 shadow-xl transition-all duration-300 hover:border-[#ffb237]/30 overflow-hidden text-center">
                            <div className="absolute -right-6 -top-6 w-20 h-20 bg-[#ffb237]/5 rounded-full blur-3xl group-hover:bg-[#ffb237]/10 transition-colors" />
                            <div className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-4">Pending</div>
                            <div className="text-3xl sm:text-4xl font-black text-[#ffb237] tracking-tighter mb-2">
                                {stats.pending_payments}
                            </div>
                            <div className="text-sm font-bold text-muted-foreground/60">In Progress</div>
                        </div>

                        <div className="group relative bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] p-8 shadow-xl transition-all duration-300 hover:border-[#ed4b9e]/30 overflow-hidden text-center">
                            <div className="absolute -right-6 -top-6 w-20 h-20 bg-[#ed4b9e]/5 rounded-full blur-3xl group-hover:bg-[#ed4b9e]/10 transition-colors" />
                            <div className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-4">Today</div>
                            <div className="text-3xl sm:text-4xl font-black text-[#ed4b9e] tracking-tighter mb-2 truncate">
                                {formatCurrency(stats.revenue_today)}
                            </div>
                            <div className="text-sm font-bold text-muted-foreground/60">Current Revenue</div>
                        </div>
                    </div>
                )}

                {/* Filter Section */}
                <div className="relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-6 shadow-xl mb-12">
                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6 items-end">
                        <div className="space-y-2">
                            <label className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2">Search</label>
                            <div className="relative group">
                                <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-[#1fc7d4] transition-colors" />
                                <input
                                    type="text"
                                    placeholder="Reference, wallet..."
                                    value={filters.search}
                                    onChange={(e) => setFilters({ ...filters, search: e.target.value })}
                                    className="w-full pl-11 pr-4 py-3 bg-white/5 border border-white/10 rounded-2xl text-foreground placeholder-muted-foreground/50 focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-white/10 transition-all font-bold text-sm"
                                />
                            </div>
                        </div>

                        <div className="space-y-2">
                            <label className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2">Status</label>
                            <select
                                value={filters.status}
                                onChange={(e) => setFilters({ ...filters, status: e.target.value })}
                                className="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-white/10 transition-all font-bold text-sm"
                            >
                                <option value="">All Status</option>
                                <option value="succeeded">Succeeded</option>
                                <option value="pending">Pending</option>
                                <option value="failed">Failed</option>
                            </select>
                        </div>

                        <div className="space-y-2">
                            <label className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2">Method</label>
                            <select
                                value={filters.payment_method}
                                onChange={(e) => setFilters({ ...filters, payment_method: e.target.value })}
                                className="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-white/10 transition-all font-bold text-sm"
                            >
                                <option value="">All Methods</option>
                                <option value="on_chain">On Chain</option>
                                <option value="on_line">Online</option>
                            </select>
                        </div>

                        <div className="space-y-2">
                            <label className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2">Plan</label>
                            <select
                                value={filters.plan_template}
                                onChange={(e) => setFilters({ ...filters, plan_template: e.target.value as PermissionTemplateName })}
                                className="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-white/10 transition-all font-bold text-sm"
                            >
                                <option value="BASIC">Basic</option>
                                <option value="PRO">Pro</option>
                                <option value="ENTERPRISE">Enterprise</option>
                                <option value="WHALE">Whale</option>
                            </select>
                        </div>

                        <button
                            onClick={() => setFilters({ status: '', payment_method: '', date_range: '', plan_template: 'BASIC', search: '' })}
                            className="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-2xl text-muted-foreground hover:text-foreground hover:bg-white/10 font-black text-xs uppercase tracking-widest transition-all"
                        >
                            Reset
                        </button>
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

                {/* Payments Table */}
                <div className="relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-xl">
                    <div className="p-8">
                        <div className="flex flex-col sm:flex-row items-center justify-between mb-8 gap-4">
                            <h2 className="text-2xl font-black bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent uppercase tracking-tight">
                                Recent Transactions
                            </h2>
                            <div className="px-4 py-1.5 bg-white/5 rounded-full border border-white/10 text-xs font-black text-muted-foreground uppercase tracking-widest leading-none">
                                {payments.length} Payments
                            </div>
                        </div>

                        {payments.length === 0 ? (
                            <div className="text-center py-12 sm:py-16">
                                <div className="h-20 w-20 bg-[#1fc7d4]/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                                    <DollarSign className="w-10 h-10 text-[#1fc7d4]" />
                                </div>
                                <h3 className="text-xl font-semibold text-foreground mb-2">
                                    No payments found
                                </h3>
                                <p className="text-muted-foreground">
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
                                            className="p-4 bg-muted/30 border border-border/50 rounded-2xl"
                                        >
                                            <div className="flex items-center justify-between mb-3">
                                                <span className="font-mono text-xs text-muted-foreground">{payment.payment_reference}</span>
                                                <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(payment.status)}`}>
                                                    {getStatusIcon(payment.status)}
                                                    <span className="ml-1">{payment.status}</span>
                                                </span>
                                            </div>
                                            <div className="grid grid-cols-2 gap-3">
                                                <div className="bg-card rounded-xl p-3 border border-border/50">
                                                    <div className="text-sm font-medium text-muted-foreground">Amount</div>
                                                    <div className="text-lg font-bold text-primary">
                                                        {formatCurrency(payment.amount, payment.currency)}
                                                    </div>
                                                </div>
                                                <div className="bg-card rounded-xl p-3 border border-border/50">
                                                    <div className="text-sm font-medium text-muted-foreground">Plan</div>
                                                    <div className="text-lg font-bold text-secondary">{payment.plan_name}</div>
                                                </div>
                                            </div>
                                            <div className="mt-3 text-xs text-muted-foreground">{formatDate(payment.created_at)}</div>
                                        </div>
                                    ))}
                                </div>

                                {/* Desktop Table */}
                                <div className="hidden sm:block overflow-x-auto">
                                    <table className="min-w-full">
                                        <thead>
                                            <tr className="border-b border-border/50">
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Reference</th>
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Wallet</th>
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Plan</th>
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Amount</th>
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Status</th>
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Created</th>
                                                <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Actions</th>
                                            </tr>
                                        </thead>
                                        <tbody className="divide-y divide-border/50">
                                            {payments.map((payment) => (
                                                <tr key={payment.id} className="hover:bg-muted/30 transition-colors">
                                                    <td className="px-4 py-4 whitespace-nowrap text-sm font-mono text-foreground">
                                                        {payment.payment_reference}
                                                    </td>
                                                    <td className="px-4 py-4 whitespace-nowrap">
                                                        <div className="text-xs font-mono text-muted-foreground">
                                                            {payment.wallet_address ?? 'N/A'}
                                                        </div>
                                                    </td>
                                                    <td className="px-4 py-4 whitespace-nowrap text-sm font-medium text-foreground">
                                                        {payment.plan_name}
                                                    </td>
                                                    <td className="px-6 py-5 whitespace-nowrap text-sm font-bold text-[#1fc7d4]">
                                                        {formatCurrency(payment.amount, payment.currency)}
                                                    </td>
                                                    <td className="px-4 py-4 whitespace-nowrap">
                                                        <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${getStatusColor(payment.status)}`}>
                                                            {getStatusIcon(payment.status)}
                                                            <span className="ml-1">{payment.status}</span>
                                                        </span>
                                                    </td>
                                                    <td className="px-4 py-4 whitespace-nowrap text-sm text-muted-foreground">
                                                        {formatDate(payment.created_at)}
                                                    </td>
                                                    <td className="px-4 py-4 whitespace-nowrap">
                                                        <button className="p-2 rounded-xl text-[#1fc7d4] hover:bg-[#1fc7d4]/10 transition-colors">
                                                            <Eye className="w-5 h-5" />
                                                        </button>
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>

                                {/* Pagination */}
                                {totalPages > 1 && (
                                    <div className="mt-6 flex items-center justify-between border-t border-border/50 pt-6">
                                        <p className="text-sm text-muted-foreground">
                                            Page <span className="font-semibold text-foreground">{currentPage}</span> of <span className="font-semibold text-foreground">{totalPages}</span>
                                        </p>
                                        <div className="flex gap-2">
                                            <button
                                                onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                                                disabled={currentPage === 1}
                                                className="px-4 py-2 text-sm font-medium rounded-xl bg-muted text-muted-foreground hover:bg-muted/80 disabled:opacity-50 disabled:cursor-not-allowed transition-colors border border-border/50"
                                            >
                                                Previous
                                            </button>
                                            <button
                                                onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
                                                disabled={currentPage === totalPages}
                                                className="px-4 py-2 text-sm font-medium rounded-xl bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
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
    );
};

export default PaymentsManagement;
