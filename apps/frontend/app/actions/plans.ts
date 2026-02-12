'use server';

import { createPlansClient } from '@/shared/api/plans';
import { FREE_PLAN_RANKING_OFFSET, FREE_PLAN_TIER_LEVEL } from '@/shared/config/constants';
import type { PlanAccessData } from '@/shared/types/payment';
import { logger } from '@/shared/utils/logger';
import { getServerActionClient } from '@/shared/utils/server-fetch';

const DEFAULT_FREE_TIER: PlanAccessData = {
  wallet_address: '',
  plan_name: null,
  plan_expires_at: null,
  days_remaining: 0,
  status: 'no_plan',
  ranking_offset: FREE_PLAN_RANKING_OFFSET,
  can_upgrade: true,
  tier_level: FREE_PLAN_TIER_LEVEL,
};

/**
 * Fetch public plans for the pricing section
 */
export async function getPublicPlansAction(filters?: { category?: string; affiliate_code?: string }) {
  const client = getServerActionClient();
  const plansApi = createPlansClient(client);

  // Note: getPublicPlans in PlansApi doesn't currently take affiliate_code
  // but we can pass it if we update the API client, or handle it via query params.
  // For now, let's keep it consistent with the existing implementation.

  return plansApi.getPublicPlans(filters);
}

/**
 * Server action to get current user's plan access data.
 * Returns default free tier config if user is unauthenticated or has no active plan.
 */
export async function getMyPlanAccessAction(): Promise<PlanAccessData> {
  const client = getServerActionClient();
  const plansApi = createPlansClient(client);

  try {
    const response = await plansApi.getMyPlanAccess();
    if (response.success && response.data) {
      return response.data;
    }
  } catch (error) {
    // Return default free tier on any error
    logger.debug('Failed to fetch plan access, returning default free tier:', error);
  }

  return DEFAULT_FREE_TIER;
}
