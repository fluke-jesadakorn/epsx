'use server';

import { redirectOnForbidden } from '@/lib/api-error';
import { createAdminApiClient } from '@/shared/api';
import type { ApiResponse } from '@/shared/utils/api-client';

function check403<T>(res: ApiResponse<T>): ApiResponse<T> {
    redirectOnForbidden(res, '/payments');
    return res;
}

export async function fetchPaymentsAction(params: Record<string, string>) {
    const client = createAdminApiClient({ serverSide: true });
    return check403(await client.get('/api/payments/admin/list', params));
}

export async function fetchUserAccessAction(params: Record<string, string>) {
    const client = createAdminApiClient({ serverSide: true });
    return check403(await client.get('/api/admin/plans/user-access/list', params));
}

export async function fetchPaymentLinksAction(params: Record<string, string>) {
    const client = createAdminApiClient({ serverSide: true });
    return check403(await client.get('/api/admin/payment-links', params));
}

export async function createPaymentLinkAction(payload: Record<string, unknown>) {
    const client = createAdminApiClient({ serverSide: true });
    return check403(await client.post('/api/admin/payment-links', payload));
}

export async function deletePaymentLinkAction(id: string) {
    const client = createAdminApiClient({ serverSide: true });
    return check403(await client.delete(`/api/admin/payment-links/${id}`));
}
