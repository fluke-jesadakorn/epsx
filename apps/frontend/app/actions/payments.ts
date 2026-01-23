'use server';

import { PaymentsApi, type PaymentSubmitRequest, type PaymentValidateRequest } from '@/shared/api/payments';
import { getServerActionClient } from '@/shared/utils/server-fetch';

/**
 * Submit a transaction for backend monitoring
 */
export async function submitTransactionAction(request: PaymentSubmitRequest) {
    const client = await getServerActionClient();
    const paymentsApi = new PaymentsApi(client);

    return paymentsApi.submitTransaction(request);
}

/**
 * Get the status of a transaction
 */
export async function getTransactionStatusAction(transactionHash: string) {
    const client = await getServerActionClient();
    const paymentsApi = new PaymentsApi(client);

    return paymentsApi.getTransactionStatus(transactionHash);
}

/**
 * Validate a payment (used in confirmation logic)
 */
export async function validatePaymentAction(request: PaymentValidateRequest) {
    const client = await getServerActionClient();
    const paymentsApi = new PaymentsApi(client);

    return paymentsApi.validatePayment(request);
}

/**
 * Get payment history
 */
export async function getPaymentHistoryAction(params?: {
    limit?: number;
    offset?: number;
    status?: string;
}) {
    const client = await getServerActionClient();
    const paymentsApi = new PaymentsApi(client);

    return paymentsApi.getPaymentHistory(params);
}
