/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-argument, @typescript-eslint/strict-boolean-expressions */
'use client';

import {
    BarChart3,
    CheckCircle,
    Clock,
    DollarSign,
    ExternalLink,
    RefreshCcw,
    Search,
    XCircle
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { fetchPaymentsAction } from '@/app/payments/actions';
import { getExplorerUrl } from '@/shared/config/constants';
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

            const response = await fetchPaymentsAction(params);

            if (response.success && response.data) {
                setPayments((response.data as any).payments ?? []);
                setStats((response.data as any).summary ?? null);
                setTotalPages((response.data as any).pagination?.total_pages ?? 1);
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
    }, [currentPage, itemsPerPage, filters]);

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
            <div className="max-w-7xl mx-auto space-y-4 animate-pulse">
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <div key={i} className="bg-card rounded-xl h-24 border border-border/20" />
                    ))}
                </div>
                <div className="bg-card rounded-2xl h-96 border border-border/20" />
            </div>
        );
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            <div className="relative max-w-7xl mx-auto">
                {/* Action Bar */}
                <div className="flex items-center gap-3 mb-8">
                    <button
                        onClick={() => void loadPayments()}
                        className="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-xl font-semibold text-sm transition-all"
                    >
                        <RefreshCcw className="w-4 h-4" />
                        Refresh
                    </button>
                    <button
                        onClick={exportPaymentsToCSV}
                        className="flex items-center gap-2 px-4 py-2.5 bg-muted/30 border border-border/40 hover:border-[#7645d9]/40 text-foreground rounded-xl font-semibold text-sm transition-all"
                    >
                        <BarChart3 className="w-4 h-4 text-[#7645d9]" />
                        Export CSV
                    </button>
                </div>

                {/* Stats Grid */}
                {stats && (
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
                        <div className="rounded-xl border border-border/20 bg-card p-5 overflow-hidden">
                            <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3">Total Revenue</div>
                            <div className="text-2xl sm:text-3xl font-black text-[#1fc7d4] tracking-tight mb-1">
                                {formatCurrency(stats.total_amount)}
                            </div>
                            <div className="text-xs text-muted-foreground/60">Platform Total</div>
                        </div>
                        <div className="rounded-xl border border-border/20 bg-card p-5 overflow-hidden">
                            <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3">Successful</div>
                            <div className="text-2xl sm:text-3xl font-black text-[#31d0aa] tracking-tight mb-1">
                                {stats.successful_payments}
                            </div>
                            <div className="text-xs text-muted-foreground/60">Completed</div>
                        </div>
                        <div className="rounded-xl border border-border/20 bg-card p-5 overflow-hidden">
                            <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3">Pending</div>
                            <div className="text-2xl sm:text-3xl font-black text-[#ffb237] tracking-tight mb-1">
                                {stats.pending_payments}
                            </div>
                            <div className="text-xs text-muted-foreground/60">In Progress</div>
                        </div>
                        <div className="rounded-xl border border-border/20 bg-card p-5 overflow-hidden">
                            <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] mb-3">Today</div>
                            <div className="text-2xl sm:text-3xl font-black text-[#ed4b9e] tracking-tight mb-1 truncate">
                                {formatCurrency(stats.revenue_today)}
                            </div>
                            <div className="text-xs text-muted-foreground/60">Current Revenue</div>
                        </div>
                    </div>
                )}

                {/* Filter Section */}
                <div className="rounded-xl border border-border/20 bg-card p-4 mb-6">
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
                                    className="w-full pl-11 pr-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground placeholder-muted-foreground/50 focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm"
                                />
                            </div>
                        </div>

                        <div className="space-y-2">
                            <label className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] ml-2">Status</label>
                            <select
                                value={filters.status}
                                onChange={(e) => setFilters({ ...filters, status: e.target.value })}
                                className="w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm"
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
                                className="w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm"
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
                                className="w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-foreground focus:outline-none focus:border-[#1fc7d4]/50 focus:bg-muted/50 transition-all font-bold text-sm"
                            >
                                <option value="BASIC">Basic</option>
                                <option value="PRO">Pro</option>
                                <option value="ENTERPRISE">Enterprise</option>
                                <option value="WHALE">Whale</option>
                            </select>
                        </div>

                        <button
                            onClick={() => setFilters({ status: '', payment_method: '', date_range: '', plan_template: 'BASIC', search: '' })}
                            className="w-full px-4 py-3 bg-muted/30 border border-border/40 rounded-2xl text-muted-foreground hover:text-foreground hover:bg-muted/50 font-black text-xs uppercase tracking-widest transition-all"
                        >
                            Reset
                        </button>
                    </div>
                </div>

                {/* Error State */}
                {error && (
                    <div className="relative overflow-hidden rounded-2xl bg-destructive/10 p-0.5 mb-6">
                        <div className="bg-destructive/5 rounded-2xl p-4 text-destructive border border-destructive/20">
                            {error}
                        </div>
                    </div>
                )}

                {/* Payments Table */}
                <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
                    <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
                    <div className="p-6 sm:p-8">
                        <div className="flex flex-col sm:flex-row items-center justify-between mb-6 gap-4">
                            <h2 className="text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]">
                                Recent Transactions
                            </h2>
                            <div className="px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground uppercase tracking-widest">
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
                                            <div className="mt-3 flex items-center justify-between">
                                                <span className="text-xs text-muted-foreground">{formatDate(payment.created_at)}</span>
                                                {payment.transaction_hash && (
                                                    <a
                                                        href={getExplorerUrl(payment.transaction_hash, payment.currency as any)}
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                        className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold text-[#1fc7d4] border border-[#1fc7d4]/30 hover:bg-[#1fc7d4]/10 transition-colors"
                                                    >
                                                        <ExternalLink className="w-3 h-3" />
                                                        Explorer
                                                    </a>
                                                )}
                                            </div>
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
                                                        {payment.transaction_hash ? (
                                                            <a
                                                                href={getExplorerUrl(payment.transaction_hash, payment.currency as any)}
                                                                target="_blank"
                                                                rel="noopener noreferrer"
                                                                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold text-[#1fc7d4] border border-[#1fc7d4]/30 hover:bg-[#1fc7d4]/10 transition-colors"
                                                            >
                                                                <ExternalLink className="w-3.5 h-3.5" />
                                                                Explorer
                                                            </a>
                                                        ) : (
                                                            <span className="text-xs text-muted-foreground/40">—</span>
                                                        )}
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
