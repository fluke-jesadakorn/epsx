/**
 * Payment Links API Client
 * 
 * Client for managing dynamic payment links (V2 payment system).
 * Used by admin frontend for CRUD operations.
 */

import type {
    CreatePaymentLinkRequest,
    PaymentContextType,
    PaymentLink,
    UpdatePaymentLinkRequest
} from '../types/payment';

// Admin API base path
const getAdminApiBase = () => {
    if (typeof window !== 'undefined') {
        return '/api/admin';
    }
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL;
    return (typeof backendUrl === 'string' && backendUrl !== '')
        ? `${backendUrl}/api/admin`
        : 'http://localhost:8080/api/admin';
};

const ERR_PAYMENT_LINK_NOT_FOUND = 'Payment link not found';

// Public API base path
const getPublicApiBase = () => {
    if (typeof window !== 'undefined') {
        return '/api/public';
    }
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL;
    return (typeof backendUrl === 'string' && backendUrl !== '')
        ? `${backendUrl}/api/public`
        : 'http://localhost:8080/api/public';
};

/**
 * List payment links response
 */
export interface PaymentLinksListResponse {
    payment_links: PaymentLink[];
    total: number;
    limit: number;
    offset: number;
}

/**
 * List payment links filters
 */
export interface ListPaymentLinksParams {
    context_type?: PaymentContextType;
    is_active?: boolean;
    limit?: number;
    offset?: number;
}

/**
 * Payment Links API Client
 */
export class PaymentLinksAPIClient {
    private adminBase: string;
    private publicBase: string;
    private getAuthHeaders: () => Promise<Record<string, string>>;

    constructor(getAuthHeaders: () => Promise<Record<string, string>>) {
        this.adminBase = getAdminApiBase();
        this.publicBase = getPublicApiBase();
        this.getAuthHeaders = getAuthHeaders;
    }

    /**
     * Create a new payment link (admin only)
     */
    async createPaymentLink(request: CreatePaymentLinkRequest): Promise<PaymentLink> {
        const headers = await this.getAuthHeaders();

        const response = await fetch(`${this.adminBase}/payment-links`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                ...headers,
            },
            body: JSON.stringify(request),
        });

        if (!response.ok) {
            const error = (await response.json().catch(() => ({ message: 'Request failed' }))) as { message?: string };
            throw new Error(error.message ?? `Failed to create payment link: ${response.status}`);
        }

        return response.json() as Promise<PaymentLink>;
    }

    /**
     * List payment links with filters (admin only)
     */
    async listPaymentLinks(params?: ListPaymentLinksParams): Promise<PaymentLinksListResponse> {
        const headers = await this.getAuthHeaders();
        const searchParams = new URLSearchParams();

        if (params?.context_type !== undefined) { searchParams.append('context_type', params.context_type); }
        if (params?.is_active !== undefined) { searchParams.append('is_active', String(params.is_active)); }
        if (params?.limit !== undefined && params.limit !== 0) { searchParams.append('limit', String(params.limit)); }
        if (params?.offset !== undefined) { searchParams.append('offset', String(params.offset)); }

        const searchParamsStr = searchParams.toString();
        const url = `${this.adminBase}/payment-links${searchParamsStr !== '' ? `?${searchParamsStr}` : ''}`;

        const response = await fetch(url, {
            method: 'GET',
            headers,
        });

        if (!response.ok) {
            throw new Error(`Failed to list payment links: ${response.status}`);
        }

        return response.json() as Promise<PaymentLinksListResponse>;
    }

    /**
     * Get payment link by ID (admin only)
     */
    async getPaymentLink(id: string): Promise<PaymentLink> {
        const headers = await this.getAuthHeaders();

        const response = await fetch(`${this.adminBase}/payment-links/${id}`, {
            method: 'GET',
            headers,
        });

        if (!response.ok) {
            if (response.status === 404) {
                throw new Error(ERR_PAYMENT_LINK_NOT_FOUND);
            }
            throw new Error(`Failed to get payment link: ${response.status}`);
        }

        return response.json() as Promise<PaymentLink>;
    }

    /**
     * Get payment link by slug (public endpoint)
     */
    async getPaymentLinkBySlug(slug: string): Promise<PaymentLink> {
        const response = await fetch(`${this.publicBase}/payment-links/${slug}`, {
            method: 'GET',
        });

        if (!response.ok) {
            if (response.status === 404) {
                throw new Error(ERR_PAYMENT_LINK_NOT_FOUND);
            }
            if (response.status === 410) {
                throw new Error('Payment link expired or max uses reached');
            }
            throw new Error(`Failed to get payment link: ${response.status}`);
        }

        return response.json() as Promise<PaymentLink>;
    }

    /**
     * Update payment link (admin only)
     */
    async updatePaymentLink(id: string, request: UpdatePaymentLinkRequest): Promise<PaymentLink> {
        const headers = await this.getAuthHeaders();

        const response = await fetch(`${this.adminBase}/payment-links/${id}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                ...headers,
            },
            body: JSON.stringify(request),
        });

        if (!response.ok) {
            if (response.status === 404) {
                throw new Error(ERR_PAYMENT_LINK_NOT_FOUND);
            }
            throw new Error(`Failed to update payment link: ${response.status}`);
        }

        return response.json() as Promise<PaymentLink>;
    }

    /**
     * Delete (deactivate) payment link (admin only)
     */
    async deletePaymentLink(id: string): Promise<void> {
        const headers = await this.getAuthHeaders();

        const response = await fetch(`${this.adminBase}/payment-links/${id}`, {
            method: 'DELETE',
            headers,
        });

        if (!response.ok) {
            if (response.status === 404) {
                throw new Error(ERR_PAYMENT_LINK_NOT_FOUND);
            }
            throw new Error(`Failed to delete payment link: ${response.status}`);
        }
    }

    /**
     * Record payment usage (admin only, called after successful blockchain payment)
     */
    async recordPaymentUsage(id: string): Promise<PaymentLink> {
        const headers = await this.getAuthHeaders();

        const response = await fetch(`${this.adminBase}/payment-links/${id}/record-usage`, {
            method: 'POST',
            headers,
        });

        if (!response.ok) {
            if (response.status === 404) {
                throw new Error(ERR_PAYMENT_LINK_NOT_FOUND);
            }
            if (response.status === 410) {
                throw new Error('Payment link no longer usable');
            }
            throw new Error(`Failed to record usage: ${response.status}`);
        }

        return response.json() as Promise<PaymentLink>;
    }
}

/**
 * Create a PaymentLinksAPIClient instance
 */
export function createPaymentLinksClient(
    getAuthHeaders: () => Promise<Record<string, string>>
): PaymentLinksAPIClient {
    return new PaymentLinksAPIClient(getAuthHeaders);
}

/**
 * Helper to compute link hash from slug (matches backend/smart contract)
 */
export function computeLinkHash(slug: string): string {
    // This should use keccak256 in the browser
    // For now, return a placeholder - the actual hash is computed backend-side
    return `0x${Array.from(new TextEncoder().encode(slug))
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('')
        .padEnd(64, '0')}`;
}

/**
 * Generate payment URL from slug
 */
export function getPaymentUrl(slug: string, baseUrl?: string): string {
    const appUrl = process.env.NEXT_PUBLIC_APP_URL;
    const base = (baseUrl !== undefined && baseUrl !== '') ? baseUrl : (typeof appUrl === 'string' && appUrl !== '') ? appUrl : 'https://epsx.io';
    return `${base}/payment?link=${encodeURIComponent(slug)}`;
}
