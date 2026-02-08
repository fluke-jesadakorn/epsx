'use client';

import { Badge, Button } from '@/components/ui';
import { env } from '@/config/env';
import { getExplorerTxUrl } from '@/lib/contracts/addresses';
import { useApiClient } from '@/shared/hooks/use-api-client';
import { format } from 'date-fns';
import {
    CheckCircle,
    Clock,
    ExternalLink,
    Receipt,
    RotateCcw,
    Search,
    XCircle,
} from 'lucide-react';
import { useEffect, useState } from 'react';

export interface PaymentHistoryItem {
    id: string;
    amount: number;
    currency: string;
    status: 'completed' | 'pending' | 'failed' | 'cancelled' | 'confirming';
    tx_hash?: string;
    plan_name?: string;
    payment_reference: string;
    created_at: string;
    completed_at?: string;
}

export interface PaymentHistoryPagination {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
}

export interface PaymentHistoryData {
    payments: PaymentHistoryItem[];
    pagination: PaymentHistoryPagination;
}

interface PaymentHistoryBody {
    success: boolean;
    data: PaymentHistoryData;
}

interface PaymentHistoryTabProps {
    initialData?: PaymentHistoryData;
}

export function PaymentHistoryTab({ initialData }: PaymentHistoryTabProps) {
    const { base } = useApiClient({ platform: 'frontend' });

    // Initialize state with initialData if provided
    const [payments, setPayments] = useState<PaymentHistoryItem[]>(initialData?.payments || []);
    // If we have initial data (and it has items), we are not loading. 
    // If initial data is undefined, we are loading.
    const [loading, setLoading] = useState(!initialData);
    const [error, setError] = useState<string | null>(null);

    // Pagination
    const [page, setPage] = useState(initialData?.pagination?.page || 1);
    const [totalPages, setTotalPages] = useState(initialData?.pagination?.total_pages || 1);
    const [totalItems, setTotalItems] = useState(initialData?.pagination?.total || 0);

    const fetchPaymentHistory = async (silent = false) => {
        try {
            if (!silent) {
                setLoading(true);
            }
            // Don't clear error on silent updates to avoid flickering if it was already showing error
            if (!silent) {setError(null);}

            const response = await base.get<PaymentHistoryData>('/api/payments/history', {
                page: page.toString(),
                per_page: '10'
            });

            if (response && response.success && response.data) {
                setPayments(response.data.payments);
                setTotalPages(response.data.pagination.total_pages);
                setTotalItems(response.data.pagination.total);
            } else {
                if (!response.success) {
                    throw new Error(response.error?.message || 'API reported failure');
                }
                if (response.success && !response.data) {
                    setPayments([]);
                    setTotalPages(1);
                    setTotalItems(0);
                }
            }
        } catch (err) {
            console.error('Error fetching payment history:', err);
            if (!silent) {setError('Unable to load payment history. Please try again later.');}
        } finally {
            if (!silent) {
                setLoading(false);
            }
        }
    };

    useEffect(() => {
        // Only fetch if we don't have initial data OR if page changed from initial
        // Use a ref or simple check? 
        // Simple check: if initialData provided matches current page, don't fetch on mount?
        // But useEffect runs on mount.

        // If initialData is present and page is 1 (or matching initial), we skip the FIRST fetch.
        // But if user changes page, we must fetch.

        // Actually, just checking if we already have data for this page?
        // But we might want to refresh.

        // Standard pattern: 
        // 1. Initial render uses initialData.
        // 2. useEffect runs. If initialData covers current page and we just mounted, maybe skip?
        // Let's keep it simple: If initialData is provided, we set loading=false. 
        // The useEffect dependency [page] will trigger on mount? No, only if page changes or mounting.

        // If we initialized with data, we might not want to fetch immediately.
        // Let's use a flag or check if payments are populated matching the page?

        // Better approach for Hybrid:
        // Identify if this is the "initial load" where we have data.

        const isInitialLoad = page === initialData?.pagination.page && payments === initialData.payments;

        if (!isInitialLoad) {
            fetchPaymentHistory();
        }
    }, [page, base]);

    // Polling effect for pending/confirming transactions
    useEffect(() => {
        const hasPending = payments.some(p => p.status === 'pending' || p.status === 'confirming');
        if (!hasPending) {return;}

        const intervalId = setInterval(() => {
            fetchPaymentHistory(true);
        }, 5000);

        return () => clearInterval(intervalId);
    }, [payments]);

    const getStatusBadge = (status: string) => {
        switch (status) {
            case 'completed':
                return (
                    <Badge variant="outline" className="bg-green-50/50 dark:bg-green-900/20 text-green-600 border-green-200 dark:border-green-800 text-[10px] font-bold py-0.5">
                        <CheckCircle className="w-3 h-3 mr-1" />
                        COMPLETED
                    </Badge>
                );
            case 'pending':
                return (
                    <Badge variant="outline" className="bg-yellow-50/50 dark:bg-yellow-900/20 text-yellow-600 border-yellow-200 dark:border-yellow-800 text-[10px] font-bold py-0.5">
                        <Clock className="w-3 h-3 mr-1" />
                        PENDING
                    </Badge>
                );
            case 'confirming':
                return (
                    <Badge variant="outline" className="bg-blue-50/50 dark:bg-blue-900/20 text-blue-600 border-blue-200 dark:border-blue-800 text-[10px] font-bold py-0.5 animate-pulse">
                        <Clock className="w-3 h-3 mr-1" />
                        CONFIRMING
                    </Badge>
                );
            default:
                return (
                    <Badge variant="outline" className="bg-red-50/50 dark:bg-red-900/20 text-red-600 border-red-200 dark:border-red-800 text-[10px] font-bold py-0.5">
                        <XCircle className="w-3 h-3 mr-1" />
                        {status.toUpperCase()}
                    </Badge>
                );
        }
    };

    const formatCurrency = (amount: number, currency: string) => {
        try {
            return new Intl.NumberFormat('en-US', {
                style: 'currency',
                currency,
            }).format(amount);
        } catch (_e) {
            // Fallback for non-standard currencies (e.g. USDT)
            return `${amount.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 6 })} ${currency}`;
        }
    };

    const getExplorerLink = (txHash?: string) => {
        if (!txHash) {return null;}
        // Use the configured chain ID from environment to generate the correct explorer URL
        // identifying if we are on local/testnet/mainnet
        return getExplorerTxUrl(Number(env.CHAIN_ID), txHash);
    };

    return (
        <div className="space-y-8">
            <div className="flex justify-between items-center bg-gray-50/50 dark:bg-gray-900/30 p-4 sm:p-5 rounded-3xl border border-gray-100 dark:border-gray-800">
                <div className="flex items-center gap-4">
                    <div className="hidden sm:flex h-12 w-12 items-center justify-center rounded-2xl bg-white dark:bg-gray-800 shadow-sm text-2xl">
                        📜
                    </div>
                    <div>
                        <h2 className="text-xl font-bold text-gray-900 dark:text-white">Payment Records</h2>
                        <p className="text-xs font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-widest mt-0.5">
                            {totalItems} total transactions
                        </p>
                    </div>
                </div>
                <Button
                    onClick={() => fetchPaymentHistory()}
                    variant="ghost"
                    size="icon"
                    className="rounded-full hover:bg-white dark:hover:bg-gray-800 shadow-sm transition-all"
                    disabled={loading}
                >
                    <RotateCcw className={`w-5 h-5 text-gray-400 ${loading ? 'animate-spin' : ''}`} />
                </Button>
            </div>

            {error ? (
                <div className="rounded-[2rem] border-2 border-red-200 bg-red-50/50 p-8 dark:border-red-900/50 dark:bg-red-900/20 text-center">
                    <XCircle className="h-10 w-10 text-red-400 mx-auto mb-4" />
                    <h3 className="text-lg font-bold text-red-800 dark:text-red-200">History Load Failed</h3>
                    <p className="text-sm text-red-700 dark:text-red-300 mt-2">{error}</p>
                    <Button onClick={() => fetchPaymentHistory()} className="mt-4 bg-red-600 hover:bg-red-700">Try Again</Button>
                </div>
            ) : loading && payments.length === 0 ? (
                <div className="grid gap-4">
                    {[1, 2, 3].map((i) => (
                        <div key={i} className="animate-pulse h-24 bg-white/50 dark:bg-gray-800/50 rounded-3xl border-2 border-gray-100 dark:border-gray-700/50" />
                    ))}
                </div>
            ) : payments.length === 0 ? (
                <div className="text-center py-20 bg-white/50 dark:bg-gray-800/30 rounded-[3rem] border-2 border-dashed border-gray-100 dark:border-gray-800">
                    <div className="mx-auto h-20 w-20 text-gray-300 flex items-center justify-center rounded-3xl bg-white dark:bg-gray-800 shadow-inner mb-6">
                        <Search className="h-10 w-10" />
                    </div>
                    <h3 className="text-xl font-bold text-gray-900 dark:text-white">No Transactions Yet</h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2 max-w-xs mx-auto">All your future subscription payments and refunds will appear here.</p>
                </div>
            ) : (
                <div className="grid gap-4">
                    {payments.map((payment) => (
                        <div key={payment.id} className="group relative overflow-hidden bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-5 sm:p-6 shadow-lg border-2 border-gray-100 dark:border-gray-800 hover:border-blue-200 dark:hover:border-blue-800/50 transition-all duration-300">
                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-6 relative z-10">
                                <div className="flex items-center gap-5">
                                    <div className={`h-14 w-14 rounded-2xl flex items-center justify-center text-2xl shadow-sm ${payment.status === 'completed'
                                        ? 'bg-green-50 dark:bg-green-900/20 text-green-600'
                                        : 'bg-gray-50 dark:bg-gray-800 dark:text-gray-400'
                                        }`}>
                                        {payment.currency === 'USDT' ? '₮' : 'Ξ'}
                                    </div>
                                    <div className="space-y-1">
                                        <div className="flex items-center gap-2">
                                            <span className="text-lg font-bold text-gray-900 dark:text-white group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
                                                {payment.plan_name || 'Protocol Access'}
                                            </span>
                                            {getStatusBadge(payment.status)}
                                        </div>
                                        <div className="text-xs font-semibold text-gray-400 dark:text-gray-500 flex items-center gap-2">
                                            <Clock className="w-3 h-3" /> {format(new Date(payment.created_at), 'MMM d, yyyy · h:mm a')}
                                        </div>
                                    </div>
                                </div>

                                <div className="flex items-center justify-between sm:justify-end gap-8 bg-gray-50/30 dark:bg-black/20 p-4 rounded-2xl border border-gray-100 dark:border-gray-800 sm:border-0 sm:bg-transparent sm:p-0">
                                    <div className="text-left sm:text-right">
                                        <div className="text-xl font-black text-gray-900 dark:text-white">
                                            {formatCurrency(payment.amount, payment.currency)}
                                        </div>
                                        <div className="text-[10px] font-mono font-bold text-gray-400 mt-0.5">
                                            REF: {payment.payment_reference.substring(0, 8)}...
                                        </div>
                                    </div>

                                    <div className="flex gap-2">
                                        {payment.tx_hash && (
                                            <a
                                                href={getExplorerLink(payment.tx_hash) || '#'}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="p-3 bg-white dark:bg-gray-800 rounded-xl text-gray-400 hover:text-blue-500 shadow-sm hover:shadow-md transition-all border border-gray-100 dark:border-gray-700"
                                                title="View on Explorer"
                                            >
                                                <ExternalLink className="h-5 w-5" />
                                            </a>
                                        )}
                                        <Button variant="outline" size="icon" className="h-11 w-11 rounded-xl bg-white dark:bg-gray-800 border-gray-100 dark:border-gray-700 shadow-sm" title="Download Receipt">
                                            <Receipt className="h-5 w-5 text-gray-400" />
                                        </Button>
                                    </div>
                                </div>
                            </div>

                            {/* Decorative gradient reveal on hover */}
                            <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-blue-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
                        </div>
                    ))}
                </div>
            )}

            {totalPages > 1 && (
                <div className="flex items-center justify-between bg-white/50 dark:bg-gray-800/40 p-4 px-6 rounded-[2rem] border border-gray-100 dark:border-gray-800">
                    <p className="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-widest">
                        Page <span className="text-gray-900 dark:text-white">{page}</span> of {totalPages}
                    </p>
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage(Math.max(1, page - 1))}
                            disabled={page === 1}
                            className="rounded-xl font-bold text-xs"
                        >
                            Previous
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage(Math.min(totalPages, page + 1))}
                            disabled={page === totalPages}
                            className="rounded-xl font-bold text-xs"
                        >
                            Next
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}
