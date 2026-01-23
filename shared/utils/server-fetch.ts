import { createAdminApiClient, createFrontendApiClient } from './api-client';

/**
 * Creates a pre-configured API client for Server Actions in the Frontend app.
 * Automatically handles auth tokens from cookies via UnifiedApiClient internal logic.
 */
export async function getServerActionClient() {
    return createFrontendApiClient({ serverSide: true });
}

/**
 * Creates a pre-configured API client for Server Actions in the Admin app.
 */
export async function getAdminServerActionClient() {
    return createAdminApiClient({ serverSide: true });
}
