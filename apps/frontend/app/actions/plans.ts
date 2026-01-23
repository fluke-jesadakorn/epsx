'use server';

import { createPlansClient } from '@/shared/api/plans';
import { getServerActionClient } from '@/shared/utils/server-fetch';

/**
 * Fetch public plans for the pricing section
 */
export async function getPublicPlansAction(filters?: { category?: string; affiliate_code?: string }) {
    const client = await getServerActionClient();
    const plansApi = createPlansClient(client);

    // Note: getPublicPlans in PlansApi doesn't currently take affiliate_code
    // but we can pass it if we update the API client, or handle it via query params.
    // For now, let's keep it consistent with the existing implementation.

    return plansApi.getPublicPlans(filters);
}
