/**
 * UNIFIED PAYMENTS API CLIENT
 *
 * Payment and subscription management endpoints.
 * Migrated from frontend/app/api/payments/confirm/route.ts
 *
 * Features:
 * - Payment validation and confirmation
 * - Transaction status tracking
 * - Subscription activation
 */

import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

import { createAdminApiClient, createFrontendApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface PaymentConfirmRequest {
    plan_id: string;
    transaction_hash: string;
    amount: number;
    currency: string;
    network?: string;
    wallet_address?: string;
}

export interface PaymentValidateRequest {
    plan_id: string;
    transaction_hash: string;
    amount: number; // In cents (integer)
    currency: string;
    network: string;
    wallet_address: string;
}

export interface PaymentSubmitRequest {
    transaction_hash: string;
    plan_id: string;
    expected_amount: string;
    currency: string;
}

export interface TransactionStatusData {
    transaction_hash: string;
    status: 'pending' | 'confirming' | 'confirmed' | 'failed' | 'expired';
    confirmations: number;
    block_number: number | null;
    error_message: string | null;
    payment_reference: string | null;
    plan_name: string | null;
}

export interface PaymentValidationResult {
    success: boolean;
    data?: {
        transaction_hash: string;
        plan_id: string;
        amount: number;
        currency: string;
        status: string;
        subscription_id?: string;
    };
    message?: string;
    details?: string;
}

export interface PaymentConfirmResult {
    success: boolean;
    message: string;
    data?: {
        payment: PaymentValidationResult['data'];
        subscription: PaymentValidationResult['data'];
    };
    error?: string;
}

// ============================================================================
// PAYMENTS API CLASS
// ============================================================================

export class PaymentsApi {
    private client: UnifiedApiClient;

    constructor(client: UnifiedApiClient) {
        this.client = client;
    }

    /**
     * Validate a payment transaction with the backend
     */
    async validatePayment(request: PaymentValidateRequest): Promise<ApiResponse<PaymentValidationResult>> {
        return this.client.post<PaymentValidationResult>('/api/payments/validate', request);
    }

    /**
     * Submit a transaction for backend monitoring
     */
    async submitTransaction(request: PaymentSubmitRequest): Promise<ApiResponse<TransactionStatusData>> {
        return this.client.post<TransactionStatusData>('/api/payments/submit', request);
    }

    /**
     * Get the status of a transaction
     */
    async getTransactionStatus(transactionHash: string): Promise<ApiResponse<TransactionStatusData>> {
        return this.client.get<TransactionStatusData>(`/api/payments/status/${transactionHash}`);
    }

    /**
     * Get payment history for the current user
     */
    async getPaymentHistory(params?: {
        limit?: number;
        offset?: number;
        status?: string;
    }): Promise<ApiResponse<{ payments: TransactionStatusData[]; total: number }>> {
        return this.client.get<{ payments: TransactionStatusData[]; total: number }>('/api/payments/history', params);
    }
}

/**
 * Create a payments API client for frontend applications
 */
export function createPaymentsClient(options?: {
    baseURL?: string;
    token?: string;
    serverSide?: boolean;
}): PaymentsApi {
    const client = createFrontendApiClient(options);
    return new PaymentsApi(client);
}

/**
 * Create a payments API client for admin applications
 */
export function createAdminPaymentsClient(options?: {
    baseURL?: string;
    token?: string;
    serverSide?: boolean;
}): PaymentsApi {
    const client = createAdminApiClient(options);
    return new PaymentsApi(client);
}
