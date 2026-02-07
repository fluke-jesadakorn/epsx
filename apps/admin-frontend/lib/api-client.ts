/**
 * ADMIN API CLIENT
 *
 * Unified API client for admin-frontend application.
 * Re-exports shared UnifiedApiClient with admin platform configuration.
 */

import {
    ApiError,
    ApiResponse,
    createAdminApiClient,
    isApiError,
    isApiResponse,
    isPaginatedResponse,
    PaginatedResponse,
    UnifiedApiClient,
} from '@/shared/api';

// ============================================================================
// SINGLETON CLIENT INSTANCE
// ============================================================================

let clientInstance: UnifiedApiClient | null = null;

/**
 * Get or create the admin API client singleton
 */
export function getAdminClient(): UnifiedApiClient {
    if (!clientInstance) {
        clientInstance = createAdminApiClient({
            serverSide: typeof window === 'undefined',
        });
    }
    return clientInstance;
}

/**
 * Admin API client singleton instance
 */
export const adminApiClient = getAdminClient();

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

export { createAdminApiClient };

/**
 * Create a new admin API client with custom options
 * @param options
 * @param options.baseURL
 * @param options.token
 * @param options.serverSide
 */
export function createApiClient(options?: {
    baseURL?: string;
    token?: string;
    serverSide?: boolean;
}): UnifiedApiClient {
    return createAdminApiClient(options);
}

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type {
    ApiError, ApiResponse, PaginatedResponse,
    UnifiedApiClient
};

export {
    isApiError,
    isApiResponse,
    isPaginatedResponse
};

