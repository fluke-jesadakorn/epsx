/**
 * Shared Utilities for Server Actions
 */
import { COOKIES } from '@/shared/auth/cookies';
import { getBackendUrl } from '@/shared/utils/url-resolver';
import { cookies } from 'next/headers';

/**
 * Make an authenticated request to the backend from a server action
 * @param endpoint The API endpoint (e.g., /api/admin/wallets)
 * @param options Request options
 */
export async function makeAuthenticatedRequest<T = unknown>(
    endpoint: string,
    options: RequestInit = {}
): Promise<T> {
    const backendUrl = getBackendUrl('server');
    const cookieStore = await cookies();
    const token = cookieStore.get(COOKIES.access_token)?.value;
    const headers = (token !== undefined && token !== '' ? { Authorization: `Bearer ${token}` } : {}) as Record<string, string>;

    const response = await fetch(`${backendUrl}${endpoint}`, {
        ...options,
        headers: {
            ...headers,
            ...options.headers,
        } as Record<string, string>,
    });

    if (!response.ok) {
        const errorText = await response.text().catch(() => 'Request failed');
        throw new Error(`HTTP ${response.status}: ${errorText}`);
    }

    if (response.status === 204) {
        return undefined as unknown as Promise<T>;
    }

    // Handle empty responses
    const contentType = response.headers.get('content-type');
    if (contentType?.includes('application/json') ?? false) {
        return response.json() as Promise<T>;
    }

    throw new Error('Invalid response format: Expected JSON');
}
