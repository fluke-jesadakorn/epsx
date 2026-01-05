'use client';

import { useApiClient } from '@/shared/hooks/useApiClient';
import {
    CheckCircleIcon,
    ClipboardDocumentIcon,
    ClockIcon,
    LinkIcon,
    PlusIcon,
    QrCodeIcon,
    TrashIcon,
    XCircleIcon,
    XMarkIcon,
} from '@heroicons/react/24/outline';
import React, { useCallback, useEffect, useState } from 'react';

type PaymentContextType = 'plan' | 'group' | 'product' | 'campaign' | 'custom';

interface PaymentLink {
    id: string;
    context_type: PaymentContextType;
    context_id?: string;
    slug: string;
    name: string;
    description?: string;
    amount: number;
    currency: string;
    expires_at?: string;
    max_uses?: number;
    current_uses: number;
    is_active: boolean;
    is_usable: boolean;
    url: string;
    link_hash: string;
    created_by: string;
    metadata: Record<string, any>;
    created_at: string;
    updated_at: string;
}

interface CreatePaymentLinkForm {
    context_type: PaymentContextType;
    context_id: string;
    slug: string;
    name: string;
    description: string;
    amount: string;
    currency: string;
    expires_in_hours: string;
    max_uses: string;
}

const CONTEXT_TYPES: { value: PaymentContextType; label: string; description: string }[] = [
    { value: 'plan', label: 'Plan', description: 'Plan payment' },
    { value: 'group', label: 'Group', description: 'Permission group access' },
    { value: 'product', label: 'Product', description: 'One-time product purchase' },
    { value: 'campaign', label: 'Campaign', description: 'Promotional campaign' },
    { value: 'custom', label: 'Custom', description: 'Custom payment link' },
];

const CURRENCIES = ['USDT', 'USDC', 'BNB'];

export function PaymentLinksManagement() {
    const { base } = useApiClient({ platform: 'admin' });
    const [paymentLinks, setPaymentLinks] = useState<PaymentLink[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [copiedSlug, setCopiedSlug] = useState<string | null>(null);
    const [filterType, setFilterType] = useState<string>('');
    const [filterActive, setFilterActive] = useState<string>('');

    const [form, setForm] = useState<CreatePaymentLinkForm>({
        context_type: 'plan',
        context_id: '',
        slug: '',
        name: '',
        description: '',
        amount: '',
        currency: 'USDT',
        expires_in_hours: '24',
        max_uses: '',
    });
    const [formLoading, setFormLoading] = useState(false);
    const [formError, setFormError] = useState<string | null>(null);

    const loadPaymentLinks = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const params: Record<string, string> = {};
            if (filterType) params.context_type = filterType;
            if (filterActive) params.is_active = filterActive;

            const response = await base.get<any>('/api/admin/payment-links', params);

            if (response.success && response.data) {
                setPaymentLinks(response.data.payment_links || []);
            } else {
                throw new Error(response.error || response.message || 'Failed to load payment links');
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setLoading(false);
        }
    }, [base, filterType, filterActive]);

    useEffect(() => {
        loadPaymentLinks();
    }, [loadPaymentLinks]);

    const handleCreateLink = async (e: React.FormEvent) => {
        e.preventDefault();
        setFormLoading(true);
        setFormError(null);

        try {
            const expiresAt = form.expires_in_hours
                ? new Date(Date.now() + parseInt(form.expires_in_hours) * 60 * 60 * 1000).toISOString()
                : undefined;

            const payload = {
                context_type: form.context_type,
                context_id: form.context_id || undefined,
                slug: form.slug || undefined,
                name: form.name,
                description: form.description || undefined,
                amount: parseFloat(form.amount),
                currency: form.currency,
                expires_at: expiresAt,
                max_uses: form.max_uses ? parseInt(form.max_uses) : undefined,
            };

            const response = await base.post<any>('/api/admin/payment-links', payload);

            if (response.success && response.data) {
                const newLink = response.data;
                setPaymentLinks((prev) => [newLink, ...prev]);
                setIsModalOpen(false);
                resetForm();
            } else {
                throw new Error(response.error || response.message || 'Failed to create payment link');
            }
        } catch (err) {
            setFormError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setFormLoading(false);
        }
    };

    const handleDeleteLink = async (id: string) => {
        if (!confirm('Are you sure you want to deactivate this payment link?')) return;

        try {
            const response = await base.delete<any>(`/api/admin/payment-links/${id}`);

            if (response.success) {
                setPaymentLinks((prev) =>
                    prev.map((link) => (link.id === id ? { ...link, is_active: false, is_usable: false } : link))
                );
            } else {
                throw new Error(response.error || response.message || 'Failed to deactivate payment link');
            }
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to deactivate');
        }
    };

    const handleCopyUrl = async (link: PaymentLink) => {
        try {
            await navigator.clipboard.writeText(link.url);
            setCopiedSlug(link.slug);
            setTimeout(() => setCopiedSlug(null), 2000);
        } catch {
            alert('Failed to copy URL');
        }
    };

    const resetForm = () => {
        setForm({
            context_type: 'plan',
            context_id: '',
            slug: '',
            name: '',
            description: '',
            amount: '',
            currency: 'USDT',
            expires_in_hours: '24',
            max_uses: '',
        });
        setFormError(null);
    };

    const isExpired = (expiresAt?: string) => {
        if (!expiresAt) return false;
        return new Date(expiresAt) < new Date();
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString();
    };

    const getLinkStatusBadge = (link: PaymentLink) => {
        if (!link.is_active) {
            return (
                <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-gray-200 text-gray-700">
                    <XCircleIcon className="w-3 h-3 mr-1" />
                    Inactive
                </span>
            );
        }
        if (isExpired(link.expires_at)) {
            return (
                <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-red-400 to-rose-500 text-white">
                    <ClockIcon className="w-3 h-3 mr-1" />
                    Expired
                </span>
            );
        }
        if (link.max_uses && link.current_uses >= link.max_uses) {
            return (
                <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-yellow-400 to-orange-500 text-white">
                    <CheckCircleIcon className="w-3 h-3 mr-1" />
                    Max Uses
                </span>
            );
        }
        return (
            <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-emerald-400 to-green-500 text-white">
                <CheckCircleIcon className="w-3 h-3 mr-1" />
                Active
            </span>
        );
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
                <div className="text-center mb-12">
                    <div className="h-16 bg-gradient-to-r from-purple-400 to-pink-500 rounded-2xl w-96 mx-auto mb-6"></div>
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
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-full blur-lg"></div>
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Hero Section */}
                <div className="text-center mb-8 sm:mb-12">
                    <div className="relative inline-block">
                        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-purple-600 via-pink-600 to-rose-600 bg-clip-text text-transparent mb-4">
                            🔗 Payment Links
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-purple-400 to-pink-500 rounded-full"></div>
                    </div>
                    <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
                        Create and manage dynamic payment links for plans, products, and campaigns
                    </p>
                </div>

                {/* Action Card */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-500/20 to-rose-500/20 p-0.5 cursor-pointer"
                        onClick={() => setIsModalOpen(true)}
                    >
                        <div className="relative bg-gradient-to-br from-purple-400 via-pink-500 to-rose-500 text-white rounded-2xl sm:rounded-3xl">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <PlusIcon className="w-6 h-6" />
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Create Payment Link</h3>
                                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Generate a new payment link for your products or services</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    New Link
                                </div>
                            </div>
                        </div>
                    </div>

                    <div
                        className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-500/20 to-purple-500/20 p-0.5 cursor-pointer"
                        onClick={() => loadPaymentLinks()}
                    >
                        <div className="relative bg-gradient-to-br from-blue-400 via-indigo-500 to-purple-500 text-white rounded-2xl sm:rounded-3xl">
                            <div className="p-6 sm:p-8">
                                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                                    <span className="text-xl sm:text-2xl">🔄</span>
                                </div>
                                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload payment links from the server</p>
                                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                                    Refresh
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Filter Section */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-rose-400/20 p-0.5 mb-6">
                    <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Context Type</label>
                                <select
                                    value={filterType}
                                    onChange={(e) => setFilterType(e.target.value)}
                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                >
                                    <option value="">All Types</option>
                                    {CONTEXT_TYPES.map((type) => (
                                        <option key={type.value} value={type.value}>
                                            {type.label}
                                        </option>
                                    ))}
                                </select>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Status</label>
                                <select
                                    value={filterActive}
                                    onChange={(e) => setFilterActive(e.target.value)}
                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                >
                                    <option value="">All Status</option>
                                    <option value="true">Active</option>
                                    <option value="false">Inactive</option>
                                </select>
                            </div>
                            <div className="flex items-end">
                                <button
                                    onClick={() => {
                                        setFilterType('');
                                        setFilterActive('');
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

                {/* Payment Links Table */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-rose-400/20 p-0.5">
                    <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl overflow-hidden">
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-purple-600 via-pink-600 to-rose-600 bg-clip-text text-transparent">
                                    All Payment Links
                                </h2>
                                <div className="text-sm text-gray-500 dark:text-gray-400">
                                    {paymentLinks.length} links
                                </div>
                            </div>

                            {paymentLinks.length === 0 ? (
                                <div className="text-center py-12 sm:py-16">
                                    <div className="h-20 w-20 bg-gradient-to-br from-purple-200 to-pink-200 dark:from-purple-800 dark:to-pink-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                                        <LinkIcon className="w-10 h-10 text-purple-600 dark:text-purple-400" />
                                    </div>
                                    <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                                        No payment links yet
                                    </h3>
                                    <p className="text-gray-500 dark:text-gray-500">
                                        Create your first payment link to get started
                                    </p>
                                </div>
                            ) : (
                                <>
                                    {/* Mobile Card Layout */}
                                    <div className="block sm:hidden space-y-4">
                                        {paymentLinks.map((link) => (
                                            <div
                                                key={link.id}
                                                className="p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl"
                                            >
                                                <div className="flex items-center justify-between mb-3">
                                                    <div>
                                                        <div className="font-semibold text-gray-900 dark:text-white">{link.name}</div>
                                                        <div className="text-xs font-mono text-gray-500">{link.slug}</div>
                                                    </div>
                                                    {getLinkStatusBadge(link)}
                                                </div>
                                                <div className="grid grid-cols-2 gap-3 mb-3">
                                                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                                                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Amount</div>
                                                        <div className="text-lg font-bold text-purple-600 dark:text-purple-400">
                                                            {link.amount} {link.currency}
                                                        </div>
                                                    </div>
                                                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                                                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Usage</div>
                                                        <div className="text-lg font-bold text-pink-600 dark:text-pink-400">
                                                            {link.current_uses}{link.max_uses ? ` / ${link.max_uses}` : ' / ∞'}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div className="flex gap-2">
                                                    <button
                                                        onClick={() => handleCopyUrl(link)}
                                                        className="flex-1 px-3 py-2 rounded-xl font-semibold bg-gradient-to-r from-purple-400 to-pink-500 text-white text-sm"
                                                    >
                                                        {copiedSlug === link.slug ? '✓ Copied' : 'Copy URL'}
                                                    </button>
                                                    {link.is_active && (
                                                        <button
                                                            onClick={() => handleDeleteLink(link.id)}
                                                            className="px-3 py-2 rounded-xl bg-red-100 text-red-600 hover:bg-red-200"
                                                        >
                                                            <TrashIcon className="w-5 h-5" />
                                                        </button>
                                                    )}
                                                </div>
                                            </div>
                                        ))}
                                    </div>

                                    {/* Desktop Table */}
                                    <div className="hidden sm:block overflow-x-auto">
                                        <table className="min-w-full">
                                            <thead>
                                                <tr className="border-b border-gray-200 dark:border-gray-700">
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Name / Slug</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Type</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Amount</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Usage</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Expires</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</th>
                                                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                                                </tr>
                                            </thead>
                                            <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
                                                {paymentLinks.map((link) => (
                                                    <tr key={link.id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
                                                        <td className="px-4 py-4">
                                                            <div className="text-sm font-semibold text-gray-900 dark:text-white">{link.name}</div>
                                                            <div className="text-xs font-mono text-gray-500">{link.slug}</div>
                                                        </td>
                                                        <td className="px-4 py-4">
                                                            <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-blue-400 to-indigo-500 text-white capitalize">
                                                                {link.context_type}
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-4 text-sm font-semibold text-purple-600 dark:text-purple-400">
                                                            {link.amount} {link.currency}
                                                        </td>
                                                        <td className="px-4 py-4 text-sm text-gray-900 dark:text-white">
                                                            {link.current_uses}{link.max_uses ? ` / ${link.max_uses}` : ' / ∞'}
                                                        </td>
                                                        <td className="px-4 py-4 text-sm text-gray-500 dark:text-gray-400">
                                                            {link.expires_at ? formatDate(link.expires_at) : 'Never'}
                                                        </td>
                                                        <td className="px-4 py-4">{getLinkStatusBadge(link)}</td>
                                                        <td className="px-4 py-4">
                                                            <div className="flex items-center gap-1">
                                                                <button
                                                                    onClick={() => handleCopyUrl(link)}
                                                                    className="p-2 rounded-xl text-gray-500 hover:text-purple-600 hover:bg-purple-50 dark:hover:bg-purple-900/30 transition-colors"
                                                                    title="Copy URL"
                                                                >
                                                                    {copiedSlug === link.slug ? (
                                                                        <CheckCircleIcon className="w-5 h-5 text-green-500" />
                                                                    ) : (
                                                                        <ClipboardDocumentIcon className="w-5 h-5" />
                                                                    )}
                                                                </button>
                                                                <button
                                                                    className="p-2 rounded-xl text-gray-500 hover:text-pink-600 hover:bg-pink-50 dark:hover:bg-pink-900/30 transition-colors"
                                                                    title="Show QR Code"
                                                                >
                                                                    <QrCodeIcon className="w-5 h-5" />
                                                                </button>
                                                                {link.is_active && (
                                                                    <button
                                                                        onClick={() => handleDeleteLink(link.id)}
                                                                        className="p-2 rounded-xl text-gray-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 transition-colors"
                                                                        title="Deactivate"
                                                                    >
                                                                        <TrashIcon className="w-5 h-5" />
                                                                    </button>
                                                                )}
                                                            </div>
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

            {/* Create Payment Link Modal */}
            {isModalOpen && (
                <div className="fixed inset-0 z-50 overflow-y-auto">
                    <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
                        <div className="fixed inset-0 transition-opacity" aria-hidden="true">
                            <div className="absolute inset-0 bg-gray-900/75 backdrop-blur-sm"></div>
                        </div>
                        <div className="inline-block align-bottom bg-white dark:bg-gray-900 rounded-3xl text-left overflow-hidden shadow-2xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
                            <form onSubmit={handleCreateLink}>
                                <div className="px-6 pt-6 pb-4">
                                    <div className="flex items-center justify-between mb-6">
                                        <h3 className="text-xl font-bold bg-gradient-to-r from-purple-600 to-pink-600 bg-clip-text text-transparent">
                                            Create Payment Link
                                        </h3>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsModalOpen(false);
                                                resetForm();
                                            }}
                                            className="p-2 rounded-xl text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800"
                                        >
                                            <XMarkIcon className="w-6 h-6" />
                                        </button>
                                    </div>

                                    {formError && (
                                        <div className="mb-4 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 px-4 py-3 rounded-xl text-sm">
                                            {formError}
                                        </div>
                                    )}

                                    <div className="space-y-4">
                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Context Type *</label>
                                            <select
                                                value={form.context_type}
                                                onChange={(e) => setForm({ ...form, context_type: e.target.value as PaymentContextType })}
                                                className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                required
                                            >
                                                {CONTEXT_TYPES.map((type) => (
                                                    <option key={type.value} value={type.value}>
                                                        {type.label} - {type.description}
                                                    </option>
                                                ))}
                                            </select>
                                        </div>

                                        {(form.context_type === 'plan' || form.context_type === 'group') && (
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                    {form.context_type === 'plan' ? 'Plan ID' : 'Group ID'}
                                                </label>
                                                <input
                                                    type="text"
                                                    value={form.context_id}
                                                    onChange={(e) => setForm({ ...form, context_id: e.target.value })}
                                                    placeholder="UUID of the linked entity"
                                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                />
                                            </div>
                                        )}

                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Name *</label>
                                            <input
                                                type="text"
                                                value={form.name}
                                                onChange={(e) => setForm({ ...form, name: e.target.value })}
                                                placeholder="e.g., Pro Plan Monthly"
                                                className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                required
                                            />
                                        </div>

                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Description</label>
                                            <textarea
                                                value={form.description}
                                                onChange={(e) => setForm({ ...form, description: e.target.value })}
                                                placeholder="Optional description"
                                                rows={2}
                                                className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                            />
                                        </div>

                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Amount *</label>
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    min="0.01"
                                                    value={form.amount}
                                                    onChange={(e) => setForm({ ...form, amount: e.target.value })}
                                                    placeholder="0.00"
                                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                    required
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Currency</label>
                                                <select
                                                    value={form.currency}
                                                    onChange={(e) => setForm({ ...form, currency: e.target.value })}
                                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                >
                                                    {CURRENCIES.map((c) => (
                                                        <option key={c} value={c}>{c}</option>
                                                    ))}
                                                </select>
                                            </div>
                                        </div>

                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Expires In (hours)</label>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    value={form.expires_in_hours}
                                                    onChange={(e) => setForm({ ...form, expires_in_hours: e.target.value })}
                                                    placeholder="24"
                                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                />
                                                <p className="text-xs text-gray-500 mt-1">Leave empty for no expiration</p>
                                            </div>
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Max Uses</label>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    value={form.max_uses}
                                                    onChange={(e) => setForm({ ...form, max_uses: e.target.value })}
                                                    placeholder="Unlimited"
                                                    className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                                />
                                                <p className="text-xs text-gray-500 mt-1">Leave empty for unlimited</p>
                                            </div>
                                        </div>

                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Custom Slug (optional)</label>
                                            <input
                                                type="text"
                                                value={form.slug}
                                                onChange={(e) => setForm({ ...form, slug: e.target.value })}
                                                placeholder="Auto-generated if empty"
                                                className="w-full px-4 py-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                                            />
                                        </div>
                                    </div>
                                </div>

                                <div className="px-6 py-4 bg-gray-50 dark:bg-gray-800/50 flex flex-row-reverse gap-3">
                                    <button
                                        type="submit"
                                        disabled={formLoading}
                                        className="px-6 py-3 font-semibold rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 text-white hover:from-purple-600 hover:to-pink-600 disabled:opacity-50 transition-colors"
                                    >
                                        {formLoading ? 'Creating...' : 'Create Link'}
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setIsModalOpen(false);
                                            resetForm();
                                        }}
                                        className="px-6 py-3 font-semibold rounded-xl bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                                    >
                                        Cancel
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

export default PaymentLinksManagement;
