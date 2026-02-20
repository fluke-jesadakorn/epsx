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

export async function getPublicPlansAction(filters?: { category?: string; affiliate_code?: string }) {
  const client = getServerActionClient();
  const plansApi = createPlansClient(client);
  return plansApi.getPublicPlans(filters);
}

export async function getMyPlanAccessAction(): Promise<PlanAccessData> {
  const client = getServerActionClient();
  const plansApi = createPlansClient(client);

  try {
    const response = await plansApi.getMyPlanAccess();
    if (response.success && response.data) {
      return response.data;
    }
  } catch (error) {
    if (typeof error === 'object' && error !== null && 'digest' in error) {throw error;}
    logger.debug('Failed to fetch plan access, returning default free tier:', error);
  }

  return DEFAULT_FREE_TIER;
}
