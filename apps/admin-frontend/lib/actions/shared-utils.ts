/**
 * Shared Utilities for Server Actions
 */
import { ServerAuth } from '../server/helpers';

/**
 * Make an authenticated request to the backend from a server action
 * @param endpoint The API endpoint (e.g., /api/admin/wallets)
 * @param options Request options
 */
export async function makeAuthenticatedRequest<T = unknown>(
    endpoint: string,
    options: RequestInit = {}
): Promise<T> {
    const backendUrl = process.env.BACKEND_URL ?? 'http://127.0.0.1:8080';
    const headers = await ServerAuth.getAuthHeaders();

    const response = await fetch(`${backendUrl}${endpoint}`, {
        ...options,
        headers: {
            ...headers,
            ...options.headers,
        },
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
