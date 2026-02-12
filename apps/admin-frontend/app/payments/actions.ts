'use server';

import { createAdminApiClient } from '@/shared/api';

export async function fetchPaymentsAction(params: Record<string, string>) {
    const client = createAdminApiClient({ serverSide: true });
    return await client.get('/api/payments/admin/list', params);
}

export async function fetchPaymentAnalyticsAction() {
    const client = createAdminApiClient({ serverSide: true });
    return await client.get('/api/payments/admin/analytics');
}

export async function fetchUserAccessAction(params: Record<string, string>) {
    const client = createAdminApiClient({ serverSide: true });
    return await client.get('/api/admin/plans/user-access/list', params);
}

export async function fetchPaymentLinksAction(params: Record<string, string>) {
    const client = createAdminApiClient({ serverSide: true });
    return await client.get('/api/admin/payment-links', params);
}

export async function createPaymentLinkAction(payload: Record<string, unknown>) {
    const client = createAdminApiClient({ serverSide: true });
    return await client.post('/api/admin/payment-links', payload);
}

export async function deletePaymentLinkAction(id: string) {
    const client = createAdminApiClient({ serverSide: true });
    return await client.delete(`/api/admin/payment-links/${id}`);
}
