/**
 * Admin Payment Links Management Page (V2 Dynamic Payments)
 *
 * Enables administrators to create and manage dynamic payment links
 * for plans, groups, products, campaigns, or custom purposes.
 * 
 * Features:
 * - Create payment links with configurable context types
 * - Set expiration (default: 24 hours) and usage limits (default: unlimited)
 * - Copy payment URLs and view QR codes
 * - Track usage statistics per link
 */

'use client';

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

// Types
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
    { value: 'plan', label: 'Plan', description: 'Subscription plan payment' },
    { value: 'group', label: 'Group', description: 'Permission group access' },
    { value: 'product', label: 'Product', description: 'One-time product purchase' },
    { value: 'campaign', label: 'Campaign', description: 'Promotional campaign' },
    { value: 'custom', label: 'Custom', description: 'Custom payment link' },
];

const CURRENCIES = ['USDT', 'USDC', 'BNB'];

const PaymentLinksPage: React.FC = () => {
    const [paymentLinks, setPaymentLinks] = useState<PaymentLink[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [copiedSlug, setCopiedSlug] = useState<string | null>(null);

    // Form state
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

    // Filters
    const [filterType, setFilterType] = useState<string>('');
    const [filterActive, setFilterActive] = useState<string>('');

    // Load payment links
    const loadPaymentLinks = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const params = new URLSearchParams();
            if (filterType) params.append('context_type', filterType);
            if (filterActive) params.append('is_active', filterActive);

            const response = await fetch(`/api/admin/payment-links?${params}`);
            if (!response.ok) {
                throw new Error('Failed to load payment links');
            }

            const data = await response.json();
            setPaymentLinks(data.payment_links || []);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setLoading(false);
        }
    }, [filterType, filterActive]);

    useEffect(() => {
        loadPaymentLinks();
    }, [loadPaymentLinks]);

    // Create payment link
    const handleCreateLink = async (e: React.FormEvent) => {
        e.preventDefault();
        setFormLoading(true);
        setFormError(null);

        try {
            // Calculate expiration date
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

            const response = await fetch('/api/admin/payment-links', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error(errorData.message || 'Failed to create payment link');
            }

            const newLink = await response.json();
            setPaymentLinks((prev) => [newLink, ...prev]);
            setIsModalOpen(false);
            resetForm();
        } catch (err) {
            setFormError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setFormLoading(false);
        }
    };

    // Delete payment link
    const handleDeleteLink = async (id: string) => {
        if (!confirm('Are you sure you want to deactivate this payment link?')) return;

        try {
            const response = await fetch(`/api/admin/payment-links/${id}`, {
                method: 'DELETE',
            });

            if (!response.ok) {
                throw new Error('Failed to deactivate payment link');
            }

            setPaymentLinks((prev) =>
                prev.map((link) => (link.id === id ? { ...link, is_active: false, is_usable: false } : link))
            );
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to deactivate');
        }
    };

    // Copy URL to clipboard
    const handleCopyUrl = async (link: PaymentLink) => {
        try {
            await navigator.clipboard.writeText(link.url);
            setCopiedSlug(link.slug);
            setTimeout(() => setCopiedSlug(null), 2000);
        } catch {
            alert('Failed to copy URL');
        }
    };

    // Reset form
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

    // Format date
    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString();
    };

    // Check if expired
    const isExpired = (expiresAt?: string) => {
        if (!expiresAt) return false;
        return new Date(expiresAt) < new Date();
    };

    // Get status badge
    const getStatusBadge = (link: PaymentLink) => {
        if (!link.is_active) {
            return (
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                    <XCircleIcon className="w-3 h-3 mr-1" />
                    Inactive
                </span>
            );
        }
        if (isExpired(link.expires_at)) {
            return (
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                    <ClockIcon className="w-3 h-3 mr-1" />
                    Expired
                </span>
            );
        }
        if (link.max_uses && link.current_uses >= link.max_uses) {
            return (
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                    <CheckCircleIcon className="w-3 h-3 mr-1" />
                    Max Uses Reached
                </span>
            );
        }
        return (
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                <CheckCircleIcon className="w-3 h-3 mr-1" />
                Active
            </span>
        );
    };

    return (
        <div className="p-6">
            {/* Header */}
            <div className="mb-8 flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold text-gray-900">Payment Links</h1>
                    <p className="mt-2 text-gray-600">
                        Create and manage dynamic payment links for plans, groups, and custom purposes
                    </p>
                </div>
                <button
                    onClick={() => setIsModalOpen(true)}
                    className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                    <PlusIcon className="w-5 h-5 mr-2" />
                    Create Link
                </button>
            </div>

            {/* Filters */}
            <div className="bg-white p-4 rounded-lg shadow mb-6">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">Context Type</label>
                        <select
                            value={filterType}
                            onChange={(e) => setFilterType(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
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
                        <label className="block text-sm font-medium text-gray-700 mb-1">Status</label>
                        <select
                            value={filterActive}
                            onChange={(e) => setFilterActive(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
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
                            className="px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50"
                        >
                            Reset
                        </button>
                    </div>
                </div>
            </div>

            {/* Loading State */}
            {loading && (
                <div className="text-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                    <p className="mt-4 text-gray-600">Loading payment links...</p>
                </div>
            )}

            {/* Error State */}
            {error && !loading && (
                <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
                    {error}
                </div>
            )}

            {/* Payment Links Table */}
            {!loading && !error && (
                <div className="bg-white shadow rounded-lg overflow-hidden">
                    <div className="overflow-x-auto">
                        <table className="min-w-full divide-y divide-gray-200">
                            <thead className="bg-gray-50">
                                <tr>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Name / Slug
                                    </th>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Type
                                    </th>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Amount
                                    </th>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Usage
                                    </th>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Expires
                                    </th>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Status
                                    </th>
                                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Actions
                                    </th>
                                </tr>
                            </thead>
                            <tbody className="bg-white divide-y divide-gray-200">
                                {paymentLinks.length === 0 ? (
                                    <tr>
                                        <td colSpan={7} className="px-6 py-12 text-center text-gray-500">
                                            <LinkIcon className="mx-auto h-12 w-12 text-gray-400 mb-2" />
                                            <p className="text-lg font-medium">No payment links yet</p>
                                            <p className="text-sm">Create your first payment link to get started</p>
                                        </td>
                                    </tr>
                                ) : (
                                    paymentLinks.map((link) => (
                                        <tr key={link.id} className="hover:bg-gray-50">
                                            <td className="px-6 py-4">
                                                <div className="text-sm font-medium text-gray-900">{link.name}</div>
                                                <div className="text-xs text-gray-500 font-mono">{link.slug}</div>
                                            </td>
                                            <td className="px-6 py-4">
                                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 capitalize">
                                                    {link.context_type}
                                                </span>
                                            </td>
                                            <td className="px-6 py-4 text-sm text-gray-900">
                                                {link.amount} {link.currency}
                                            </td>
                                            <td className="px-6 py-4 text-sm text-gray-900">
                                                {link.current_uses}
                                                {link.max_uses ? ` / ${link.max_uses}` : ' / ∞'}
                                            </td>
                                            <td className="px-6 py-4 text-sm text-gray-500">
                                                {link.expires_at ? formatDate(link.expires_at) : 'Never'}
                                            </td>
                                            <td className="px-6 py-4">{getStatusBadge(link)}</td>
                                            <td className="px-6 py-4">
                                                <div className="flex space-x-2">
                                                    <button
                                                        onClick={() => handleCopyUrl(link)}
                                                        className="p-1 text-gray-500 hover:text-blue-600"
                                                        title="Copy URL"
                                                    >
                                                        {copiedSlug === link.slug ? (
                                                            <CheckCircleIcon className="w-5 h-5 text-green-500" />
                                                        ) : (
                                                            <ClipboardDocumentIcon className="w-5 h-5" />
                                                        )}
                                                    </button>
                                                    <button
                                                        className="p-1 text-gray-500 hover:text-purple-600"
                                                        title="Show QR Code"
                                                    >
                                                        <QrCodeIcon className="w-5 h-5" />
                                                    </button>
                                                    {link.is_active && (
                                                        <button
                                                            onClick={() => handleDeleteLink(link.id)}
                                                            className="p-1 text-gray-500 hover:text-red-600"
                                                            title="Deactivate"
                                                        >
                                                            <TrashIcon className="w-5 h-5" />
                                                        </button>
                                                    )}
                                                </div>
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>
            )}

            {/* Create Modal */}
            {isModalOpen && (
                <div className="fixed inset-0 z-50 overflow-y-auto">
                    <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
                        <div className="fixed inset-0 transition-opacity" aria-hidden="true">
                            <div className="absolute inset-0 bg-gray-500 opacity-75"></div>
                        </div>
                        <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
                            <form onSubmit={handleCreateLink}>
                                <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                                    <div className="flex items-center justify-between mb-4">
                                        <h3 className="text-lg leading-6 font-medium text-gray-900">
                                            Create Payment Link
                                        </h3>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsModalOpen(false);
                                                resetForm();
                                            }}
                                            className="text-gray-400 hover:text-gray-500"
                                        >
                                            <XMarkIcon className="w-6 h-6" />
                                        </button>
                                    </div>

                                    {formError && (
                                        <div className="mb-4 bg-red-50 border border-red-200 text-red-700 px-3 py-2 rounded text-sm">
                                            {formError}
                                        </div>
                                    )}

                                    <div className="space-y-4">
                                        {/* Context Type */}
                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                                Context Type *
                                            </label>
                                            <select
                                                value={form.context_type}
                                                onChange={(e) =>
                                                    setForm({ ...form, context_type: e.target.value as PaymentContextType })
                                                }
                                                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                required
                                            >
                                                {CONTEXT_TYPES.map((type) => (
                                                    <option key={type.value} value={type.value}>
                                                        {type.label} - {type.description}
                                                    </option>
                                                ))}
                                            </select>
                                        </div>

                                        {/* Context ID (for plan/group) */}
                                        {(form.context_type === 'plan' || form.context_type === 'group') && (
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 mb-1">
                                                    {form.context_type === 'plan' ? 'Plan ID' : 'Group ID'}
                                                </label>
                                                <input
                                                    type="text"
                                                    value={form.context_id}
                                                    onChange={(e) => setForm({ ...form, context_id: e.target.value })}
                                                    placeholder="UUID of the linked entity"
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                />
                                            </div>
                                        )}

                                        {/* Name */}
                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 mb-1">Name *</label>
                                            <input
                                                type="text"
                                                value={form.name}
                                                onChange={(e) => setForm({ ...form, name: e.target.value })}
                                                placeholder="e.g., Pro Plan Monthly"
                                                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                required
                                            />
                                        </div>

                                        {/* Description */}
                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                                Description
                                            </label>
                                            <textarea
                                                value={form.description}
                                                onChange={(e) => setForm({ ...form, description: e.target.value })}
                                                placeholder="Optional description"
                                                rows={2}
                                                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                            />
                                        </div>

                                        {/* Amount & Currency */}
                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 mb-1">
                                                    Amount *
                                                </label>
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    min="0.01"
                                                    value={form.amount}
                                                    onChange={(e) => setForm({ ...form, amount: e.target.value })}
                                                    placeholder="0.00"
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                    required
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 mb-1">
                                                    Currency
                                                </label>
                                                <select
                                                    value={form.currency}
                                                    onChange={(e) => setForm({ ...form, currency: e.target.value })}
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                >
                                                    {CURRENCIES.map((c) => (
                                                        <option key={c} value={c}>
                                                            {c}
                                                        </option>
                                                    ))}
                                                </select>
                                            </div>
                                        </div>

                                        {/* Expiration & Max Uses */}
                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 mb-1">
                                                    Expires In (hours)
                                                </label>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    value={form.expires_in_hours}
                                                    onChange={(e) => setForm({ ...form, expires_in_hours: e.target.value })}
                                                    placeholder="24"
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                />
                                                <p className="text-xs text-gray-500 mt-1">Leave empty for no expiration</p>
                                            </div>
                                            <div>
                                                <label className="block text-sm font-medium text-gray-700 mb-1">
                                                    Max Uses
                                                </label>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    value={form.max_uses}
                                                    onChange={(e) => setForm({ ...form, max_uses: e.target.value })}
                                                    placeholder="Unlimited"
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                                />
                                                <p className="text-xs text-gray-500 mt-1">Leave empty for unlimited</p>
                                            </div>
                                        </div>

                                        {/* Custom Slug */}
                                        <div>
                                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                                Custom Slug (optional)
                                            </label>
                                            <input
                                                type="text"
                                                value={form.slug}
                                                onChange={(e) => setForm({ ...form, slug: e.target.value })}
                                                placeholder="Auto-generated if empty"
                                                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                                            />
                                        </div>
                                    </div>
                                </div>

                                <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
                                    <button
                                        type="submit"
                                        disabled={formLoading}
                                        className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
                                    >
                                        {formLoading ? 'Creating...' : 'Create Link'}
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setIsModalOpen(false);
                                            resetForm();
                                        }}
                                        className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
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
};

export default PaymentLinksPage;
