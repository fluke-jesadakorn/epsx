/**
 * Shared Utilities for Server Actions
 */
import { ServerAuth } from '../server/helpers';

/**
 * Make an authenticated request to the backend from a server action
 * @param endpoint The API endpoint (e.g., /api/admin/wallets)
 * @param options Request options
 */
export async function makeAuthenticatedRequest(
    endpoint: string,
    options: RequestInit = {}
): Promise<any> {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
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

    // Handle empty responses
    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('application/json')) {
        return response.json();
    }

    return null;
}
