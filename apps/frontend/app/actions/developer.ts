'use server';

import { createUsersClient } from '@/shared/api/users';
import { getServerActionClient } from '@/shared/utils/server-fetch';

/**
 * Fetch current user plans and basic stats
 */
export async function getMyPlansAction() {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.getMyPlans();
}

/**
 * Fetch developer usage stats
 */
export async function getUsageStatsAction() {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.getUsageStats();
}

/**
 * Fetch user API keys
 */
export async function getApiKeysAction(filters?: { limit?: number; offset?: number; status?: string }) {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.getApiKeys(filters);
}

/**
 * Fetch usage history (default 7 days)
 */
export async function getUsageHistoryAction(days = 7) {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.getUsageHistory(days);
}

/**
 * Fetch top endpoints (default 7 days)
 */
export async function getTopEndpointsAction(days = 7) {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.getTopEndpoints(days);
}

/**
 * Create a new API key
 */
export async function createApiKeyAction(body: {
    client_name: string;
    client_description?: string;
    permissions?: string[];
    plan_ids?: string[]
}) {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.createApiKey(body);
}

/**
 * Delete/Revoke an API key
 */
export async function deleteApiKeyAction(key_id: string, reason?: string) {
    const client = getServerActionClient();
    const usersClient = createUsersClient(client);
    return usersClient.deleteApiKey(key_id, reason);
}
