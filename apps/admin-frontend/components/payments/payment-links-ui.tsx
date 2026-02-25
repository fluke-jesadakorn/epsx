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
import React from 'react';
import type { CreatePaymentLinkForm, PaymentContextType, PaymentLink } from './payment-links-hooks';

const CONTEXT_TYPES: { value: PaymentContextType; label: string; description: string }[] = [
    { value: 'plan', label: 'Plan', description: 'Plan payment' },
    { value: 'group', label: 'Group', description: 'Permission group access' },
    { value: 'product', label: 'Product', description: 'One-time product purchase' },
    { value: 'campaign', label: 'Campaign', description: 'Promotional campaign' },
    { value: 'custom', label: 'Custom', description: 'Custom payment link' },
];

const CURRENCIES = ['USDT', 'USDC', 'BNB'];

interface FilterSectionProps {
    filterType: string;
    onFilterTypeChange: (value: string) => void;
    filterActive: string;
    onFilterActiveChange: (value: string) => void;
    onReset: () => void;
}

export function FilterSection({
    filterType,
    onFilterTypeChange,
    filterActive,
    onFilterActiveChange,
    onReset,
}: FilterSectionProps) {
    return (
        <div className="rounded-xl border border-border/20 bg-card p-4 mb-6">
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div>
                        <label className="block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2">Context Type</label>
                        <select
                            value={filterType}
                            onChange={(e) => onFilterTypeChange(e.target.value)}
                            className="w-full px-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-1 focus:ring-[#1fc7d4] transition-all text-sm"
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
                        <label className="block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2">Status</label>
                        <select
                            value={filterActive}
                            onChange={(e) => onFilterActiveChange(e.target.value)}
                            className="w-full px-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-1 focus:ring-[#1fc7d4] transition-all text-sm"
                        >
                            <option value="">All Status</option>
                            <option value="true">Active</option>
                            <option value="false">Inactive</option>
                        </select>
                    </div>
                    <div className="flex items-end">
                        <button
                            onClick={onReset}
                            className="w-full px-4 py-2.5 font-semibold rounded-xl bg-muted hover:bg-muted/80 text-muted-foreground transition-all border border-border/50 text-sm"
                        >
                            Reset
                        </button>
                    </div>
                </div>
            </div>
    );
}

interface ActionCardsProps {
    onCreateClick: () => void;
    onRefreshClick: () => void;
}

export function ActionCards({ onCreateClick, onRefreshClick }: ActionCardsProps) {
    return (
        <>
            <button
                onClick={onCreateClick}
                className="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-xl font-semibold text-sm transition-all"
            >
                <PlusIcon className="w-4 h-4" />
                New Link
            </button>
            <button
                onClick={onRefreshClick}
                className="flex items-center gap-2 px-4 py-2.5 bg-card border border-border/40 hover:border-[#7645d9]/40 text-foreground rounded-xl font-semibold text-sm transition-all"
            >
                Refresh
            </button>
        </>
    );
}

interface LinkStatusBadgeProps {
    link: PaymentLink;
    isExpired: (expiresAt?: string) => boolean;
}

export function LinkStatusBadge({ link, isExpired }: LinkStatusBadgeProps) {
    if (!link.is_active) {
        return (
            <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-muted text-muted-foreground border border-border/50">
                <XCircleIcon className="w-3 h-3 mr-1" />
                Inactive
            </span>
        );
    }
    if (isExpired(link.expires_at)) {
        return (
            <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-destructive/10 text-destructive border border-destructive/20">
                <ClockIcon className="w-3 h-3 mr-1" />
                Expired
            </span>
        );
    }
    if (link.max_uses && link.current_uses >= link.max_uses) {
        return (
            <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-warning/10 text-warning border border-warning/20">
                <CheckCircleIcon className="w-3 h-3 mr-1" />
                Max Uses
            </span>
        );
    }
    return (
        <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-success/10 text-success border border-success/20">
            <CheckCircleIcon className="w-3 h-3 mr-1" />
            Active
        </span>
    );
}

interface PaymentLinksMobileCardsProps {
    links: PaymentLink[];
    copiedSlug: string | null;
    onCopyUrl: (link: PaymentLink) => void;
    onDelete: (id: string) => void;
    isExpired: (expiresAt?: string) => boolean;
}

export function PaymentLinksMobileCards({
    links,
    copiedSlug,
    onCopyUrl,
    onDelete,
    isExpired,
}: PaymentLinksMobileCardsProps) {
    return (
        <div className="block sm:hidden space-y-4">
            {links.map((link) => (
                <div key={link.id} className="p-4 bg-muted/30 border border-border/50 rounded-2xl">
                    <div className="flex items-center justify-between mb-3">
                        <div>
                            <div className="font-semibold text-foreground">{link.name}</div>
                            <div className="text-xs font-mono text-muted-foreground">{link.slug}</div>
                        </div>
                        <LinkStatusBadge link={link} isExpired={isExpired} />
                    </div>
                    <div className="grid grid-cols-2 gap-3 mb-3">
                        <div className="bg-card rounded-xl p-3 border border-border/50">
                            <div className="text-sm font-medium text-muted-foreground">Amount</div>
                            <div className="text-lg font-bold text-primary">
                                {link.amount} {link.currency}
                            </div>
                        </div>
                        <div className="bg-card rounded-xl p-3 border border-border/50">
                            <div className="text-sm font-medium text-muted-foreground">Usage</div>
                            <div className="text-lg font-bold text-secondary">
                                {link.current_uses}
                                {link.max_uses ? ` / ${link.max_uses}` : ' / ∞'}
                            </div>
                        </div>
                    </div>
                    <div className="flex gap-2">
                        <button
                            onClick={() => onCopyUrl(link)}
                            className="flex-1 px-3 py-2 rounded-xl font-semibold bg-primary text-primary-foreground hover:opacity-90 transition-all text-sm"
                        >
                            {copiedSlug === link.slug ? '✓ Copied' : 'Copy URL'}
                        </button>
                        {link.is_active && (
                            <button
                                onClick={() => onDelete(link.id)}
                                className="px-3 py-2 rounded-xl bg-destructive/10 text-destructive hover:bg-destructive/20 border border-destructive/20 transition-colors"
                            >
                                <TrashIcon className="w-5 h-5" />
                            </button>
                        )}
                    </div>
                </div>
            ))}
        </div>
    );
}

interface PaymentLinksTableProps {
    links: PaymentLink[];
    copiedSlug: string | null;
    onCopyUrl: (link: PaymentLink) => void;
    onDelete: (id: string) => void;
    formatDate: (dateString: string) => string;
    isExpired: (expiresAt?: string) => boolean;
}

export function PaymentLinksTable({
    links,
    copiedSlug,
    onCopyUrl,
    onDelete,
    formatDate,
    isExpired,
}: PaymentLinksTableProps) {
    return (
        <div className="hidden sm:block overflow-x-auto">
            <table className="min-w-full">
                <thead>
                    <tr className="border-b border-border/50">
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                            Name / Slug
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Type</th>
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Amount</th>
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Usage</th>
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Expires</th>
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Status</th>
                        <th className="px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">Actions</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-border/50">
                    {links.map((link) => (
                        <tr key={link.id} className="hover:bg-muted/30 transition-colors">
                            <td className="px-4 py-4">
                                <div className="text-sm font-semibold text-foreground">{link.name}</div>
                                <div className="text-xs font-mono text-muted-foreground">{link.slug}</div>
                            </td>
                            <td className="px-4 py-4">
                                <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-primary text-primary-foreground capitalize">
                                    {link.context_type}
                                </span>
                            </td>
                            <td className="px-4 py-4 text-sm font-semibold text-primary">
                                {link.amount} {link.currency}
                            </td>
                            <td className="px-4 py-4 text-sm text-foreground">
                                {link.current_uses}
                                {link.max_uses ? ` / ${link.max_uses}` : ' / ∞'}
                            </td>
                            <td className="px-4 py-4 text-sm text-muted-foreground">
                                {link.expires_at ? formatDate(link.expires_at) : 'Never'}
                            </td>
                            <td className="px-4 py-4">
                                <LinkStatusBadge link={link} isExpired={isExpired} />
                            </td>
                            <td className="px-4 py-4">
                                <div className="flex items-center gap-1">
                                    <button
                                        onClick={() => onCopyUrl(link)}
                                        className="p-2 rounded-xl text-muted-foreground hover:text-primary hover:bg-primary/10 transition-colors"
                                        title="Copy URL"
                                    >
                                        {copiedSlug === link.slug ? (
                                            <CheckCircleIcon className="w-5 h-5 text-success" />
                                        ) : (
                                            <ClipboardDocumentIcon className="w-5 h-5" />
                                        )}
                                    </button>
                                    <button
                                        className="p-2 rounded-xl text-muted-foreground hover:text-secondary hover:bg-secondary/10 transition-colors"
                                        title="Show QR Code"
                                    >
                                        <QrCodeIcon className="w-5 h-5" />
                                    </button>
                                    {link.is_active && (
                                        <button
                                            onClick={() => onDelete(link.id)}
                                            className="p-2 rounded-xl text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
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
    );
}

interface CreatePaymentLinkModalProps {
    isOpen: boolean;
    onClose: () => void;
    form: CreatePaymentLinkForm;
    onFormChange: <K extends keyof CreatePaymentLinkForm>(key: K, value: CreatePaymentLinkForm[K]) => void;
    onSubmit: (e: React.FormEvent) => void;
    isLoading: boolean;
    error: string | null;
}

export function CreatePaymentLinkModal({
    isOpen,
    onClose,
    form,
    onFormChange,
    onSubmit,
    isLoading,
    error,
}: CreatePaymentLinkModalProps) {
    if (!isOpen) {
        return null;
    }

    return (
        <div className="fixed inset-0 z-50 overflow-y-auto">
            <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
                <div className="fixed inset-0 transition-opacity" aria-hidden="true" onClick={onClose}>
                    <div className="absolute inset-0 bg-background/95" />
                </div>
                <div className="inline-block align-bottom bg-card rounded-3xl text-left overflow-hidden shadow-2xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full border border-border/50">
                    <form onSubmit={onSubmit}>
                        <div className="px-6 pt-6 pb-4">
                            <div className="flex items-center justify-between mb-6">
                                <h3 className="text-xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                                    Create Payment Link
                                </h3>
                                <button
                                    type="button"
                                    onClick={onClose}
                                    className="p-2 rounded-xl text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                                >
                                    <XMarkIcon className="w-6 h-6" />
                                </button>
                            </div>

                            {error && (
                                <div className="mb-4 bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 rounded-xl text-sm">
                                    {error}
                                </div>
                            )}

                            <div className="space-y-4">
                                <div>
                                    <label className="block text-sm font-medium text-muted-foreground mb-2">Context Type *</label>
                                    <select
                                        value={form.context_type}
                                        onChange={(e) => onFormChange('context_type', e.target.value as PaymentContextType)}
                                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
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
                                        <label className="block text-sm font-medium text-muted-foreground mb-2">
                                            {form.context_type === 'plan' ? 'Plan ID' : 'Group ID'}
                                        </label>
                                        <input
                                            type="text"
                                            value={form.context_id}
                                            onChange={(e) => onFormChange('context_id', e.target.value)}
                                            placeholder="UUID of the linked entity"
                                            className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                        />
                                    </div>
                                )}

                                <div>
                                    <label className="block text-sm font-medium text-muted-foreground mb-2">Name *</label>
                                    <input
                                        type="text"
                                        value={form.name}
                                        onChange={(e) => onFormChange('name', e.target.value)}
                                        placeholder="e.g., Pro Plan Monthly"
                                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                        required
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-muted-foreground mb-2">Description</label>
                                    <textarea
                                        value={form.description}
                                        onChange={(e) => onFormChange('description', e.target.value)}
                                        placeholder="Optional description"
                                        rows={2}
                                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <label className="block text-sm font-medium text-muted-foreground mb-2">Amount *</label>
                                        <input
                                            type="number"
                                            step="0.01"
                                            min="0.01"
                                            value={form.amount}
                                            onChange={(e) => onFormChange('amount', e.target.value)}
                                            placeholder="0.00"
                                            className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                            required
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium text-muted-foreground mb-2">Currency</label>
                                        <select
                                            value={form.currency}
                                            onChange={(e) => onFormChange('currency', e.target.value)}
                                            className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                        >
                                            {CURRENCIES.map((c) => (
                                                <option key={c} value={c}>
                                                    {c}
                                                </option>
                                            ))}
                                        </select>
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <label className="block text-sm font-medium text-muted-foreground mb-2">Expires In (hours)</label>
                                        <input
                                            type="number"
                                            min="1"
                                            value={form.expires_in_hours}
                                            onChange={(e) => onFormChange('expires_in_hours', e.target.value)}
                                            placeholder="24"
                                            className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                        />
                                        <p className="text-xs text-muted-foreground mt-1">Leave empty for no expiration</p>
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium text-muted-foreground mb-2">Max Uses</label>
                                        <input
                                            type="number"
                                            min="1"
                                            value={form.max_uses}
                                            onChange={(e) => onFormChange('max_uses', e.target.value)}
                                            placeholder="Unlimited"
                                            className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                        />
                                        <p className="text-xs text-muted-foreground mt-1">Leave empty for unlimited</p>
                                    </div>
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-muted-foreground mb-2">Custom Slug (optional)</label>
                                    <input
                                        type="text"
                                        value={form.slug}
                                        onChange={(e) => onFormChange('slug', e.target.value)}
                                        placeholder="Auto-generated if empty"
                                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="px-6 py-4 bg-muted/30 flex flex-row-reverse gap-3 rounded-b-3xl">
                            <button
                                type="submit"
                                disabled={isLoading}
                                className="px-6 py-3 font-semibold rounded-xl bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50 transition-all border border-border/50"
                            >
                                {isLoading ? 'Creating...' : 'Create Link'}
                            </button>
                            <button
                                type="button"
                                onClick={onClose}
                                className="px-6 py-3 font-semibold rounded-xl bg-muted hover:bg-muted/80 text-muted-foreground transition-all border border-border/50"
                            >
                                Cancel
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    );
}

export function LoadingState() {
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

export function EmptyState() {
    return (
        <div className="text-center py-12 sm:py-16">
            <div className="h-20 w-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <LinkIcon className="w-10 h-10 text-primary" />
            </div>
            <h3 className="text-xl font-semibold text-foreground mb-2">No payment links yet</h3>
            <p className="text-muted-foreground">Create your first payment link to get started</p>
        </div>
    );
}
