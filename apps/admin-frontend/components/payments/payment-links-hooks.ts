'use client';

import { useCallback, useEffect, useState } from 'react';
import { copyToClipboard } from '@/lib/utils';
import { createPaymentLinkAction, deletePaymentLinkAction, fetchPaymentLinksAction } from '@/app/payments/actions';

export type PaymentContextType = 'plan' | 'group' | 'product' | 'campaign' | 'custom';

interface PaymentLinksApiResponse {
    payment_links?: PaymentLink[];
}

export interface PaymentLink {
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
    metadata: Record<string, unknown>;
    created_at: string;
    updated_at: string;
}

export interface CreatePaymentLinkForm {
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

interface UsePaymentLinksContext {
    filterType: string;
    filterActive: string;
}

export function usePaymentLinks(ctx: UsePaymentLinksContext) {
    const [paymentLinks, setPaymentLinks] = useState<PaymentLink[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const loadPaymentLinks = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const params: Record<string, string> = {};
            if (ctx.filterType) {
                params.context_type = ctx.filterType;
            }
            if (ctx.filterActive) {
                params.is_active = ctx.filterActive;
            }

            const response = await fetchPaymentLinksAction(params) as any;

            if (response.success && response.data) {
                setPaymentLinks(response.data.payment_links ?? []);
            } else {
                const errorMsg = (response.error?.message ?? response.message ?? 'Failed to load payment links') as string;
                throw new Error(errorMsg);
            }
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setLoading(false);
        }
    }, [ctx.filterType, ctx.filterActive]);

    useEffect(() => {
        void loadPaymentLinks();
    }, [loadPaymentLinks]);

    return { paymentLinks, setPaymentLinks, loading, error, loadPaymentLinks };
}

interface UsePaymentLinkFormContext {
    onSuccess?: () => void;
}

const defaultForm: CreatePaymentLinkForm = {
    context_type: 'plan',
    context_id: '',
    slug: '',
    name: '',
    description: '',
    amount: '',
    currency: 'USDT',
    expires_in_hours: '24',
    max_uses: '',
};

export function usePaymentLinkForm(ctx: UsePaymentLinkFormContext) {
    const [form, setForm] = useState<CreatePaymentLinkForm>(defaultForm);
    const [formLoading, setFormLoading] = useState(false);
    const [formError, setFormError] = useState<string | null>(null);

    const resetForm = useCallback(() => {
        setForm(defaultForm);
        setFormError(null);
    }, []);

    const handleCreateLink = useCallback(
        // eslint-disable-next-line complexity
        async (e: React.FormEvent) => {
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
                    max_uses: form.max_uses !== '' ? parseInt(form.max_uses) : undefined,
                };

                const response = await createPaymentLinkAction(payload) as any;

                if (response.success && response.data) {
                    resetForm();
                    ctx.onSuccess?.();
                } else {
                    const errorMsg = (response.error?.message ?? response.message ?? 'Failed to create payment link') as string;
                    throw new Error(errorMsg);
                }
            } catch (err: unknown) {
                setFormError(err instanceof Error ? err.message : 'Unknown error');
            } finally {
                setFormLoading(false);
            }
        },
        [form, ctx, resetForm]
    );

    return {
        form,
        setForm,
        formLoading,
        formError,
        handleCreateLink,
        resetForm,
    };
}

export function usePaymentLinkActions() {
    const [copiedSlug, setCopiedSlug] = useState<string | null>(null);

    const handleCopyUrl = useCallback(async (link: PaymentLink) => {
        const success = await copyToClipboard(link.url);
        if (success) {
            setCopiedSlug(link.slug);
            setTimeout(() => setCopiedSlug(null), 2000);
        } else {
            // eslint-disable-next-line no-alert
            alert('Failed to copy URL');
        }
    }, []);

    const handleDeleteLink = useCallback(
        async (id: string, onSuccess?: () => void) => {
            // eslint-disable-next-line no-alert
            if (!confirm('Are you sure you want to deactivate this payment link?')) {
                return;
            }

            try {
                const response = await deletePaymentLinkAction(id) as any;

                if (response.success) {
                    onSuccess?.();
                } else {
                    const errorMsg = (response.error?.message ?? response.message ?? 'Failed to deactivate payment link') as string;
                    throw new Error(errorMsg);
                }
            } catch (err: unknown) {
                // eslint-disable-next-line no-alert
                alert(err instanceof Error ? err.message : 'Failed to deactivate');
            }
        },
        []
    );

    return { copiedSlug, handleCopyUrl, handleDeleteLink };
}

export function usePaymentLinkFilters() {
    const [filterType, setFilterType] = useState<string>('');
    const [filterActive, setFilterActive] = useState<string>('');

    const resetFilters = useCallback(() => {
        setFilterType('');
        setFilterActive('');
    }, []);

    return { filterType, setFilterType, filterActive, setFilterActive, resetFilters };
}

export function usePaymentLinkFormatting() {
    const isExpired = useCallback((expiresAt?: string) => {
        if (expiresAt === undefined) {
            return false;
        }
        return new Date(expiresAt) < new Date();
    }, []);

    const formatDate = useCallback((dateString: string) => {
        return new Date(dateString).toLocaleString();
    }, []);

    return { isExpired, formatDate };
}
