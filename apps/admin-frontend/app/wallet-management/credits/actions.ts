'use server';

import { redirectOnForbidden, rethrowRedirect } from '@/lib/api-error';
import { logout } from '@/lib/auth/auth';
import { createCreditsApi } from '@/shared/api/credits';
import { createAdminApiClient } from '@/shared/utils/api-client';
import type { ApiResponse } from '@/shared/types/api';
import type {
  CreditStats,
  CreditTransactionFilters,
  GrantCreditsRequest,
  RevokeCreditsRequest,
} from '@/shared/types/credits';
import type { UnifiedApiClient } from '@/shared/utils/api-client';
import { logger } from '@/lib/logger';
import { redirect } from 'next/navigation';

async function handleApiError<T>(
  res: ApiResponse<T>,
  errMsg: string,
  defaultVal?: T
): Promise<T> {
  redirectOnForbidden(res, '/wallet-management/credits');

  if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
    await logout();
    redirect('/auth');
  }

  logger.action.error(errMsg, res.error, { code: res.error?.code });

  if (defaultVal !== undefined) {
    return defaultVal;
  }

  throw new Error(res.error?.message ?? errMsg);
}

async function handleAction<T>(
  reqFn: (api: UnifiedApiClient) => Promise<ApiResponse<T>>,
  errMsg: string,
  defaultVal?: T
): Promise<T> {
  const apiClient = createAdminApiClient({ serverSide: true });

  try {
    const res = await reqFn(apiClient);

    if (!res.success) {
      return handleApiError(res, errMsg, defaultVal);
    }

    return res.data ?? (defaultVal as T);
  } catch (error: unknown) {
    rethrowRedirect(error);

    logger.error(`${errMsg}:`, error instanceof Error ? error.message : String(error));

    if (defaultVal !== undefined) {
      return defaultVal;
    }

    throw error as Error;
  }
}

export async function getCreditStatsAction(): Promise<CreditStats> {
  return handleAction(
    (apiClient) => {
      const creditsApi = createCreditsApi(apiClient);
      return creditsApi.adminGetStats();
    },
    'Failed to fetch credit stats',
    {
      total_credits_outstanding: 0,
      total_credits_granted_today: 0,
      total_credits_used_today: 0,
      active_users_with_credits: 0,
      total_transactions_today: 0,
      average_balance: 0,
    } as CreditStats
  );
}

export async function getUserCreditsAction(
  walletAddress: string,
  filters?: CreditTransactionFilters
) {
  return handleAction(
    (apiClient) => {
      const creditsApi = createCreditsApi(apiClient);
      return creditsApi.adminGetUserCredits(walletAddress, filters);
    },
    'Failed to fetch user credits'
  );
}

export async function grantCreditsAction(
  request: GrantCreditsRequest
): Promise<{ transaction_id: string; new_balance: number }> {
  return handleAction(
    (apiClient) => {
      const creditsApi = createCreditsApi(apiClient);
      return creditsApi.adminGrantCredits(request);
    },
    'Failed to grant credits'
  );
}

export async function revokeCreditsAction(
  request: RevokeCreditsRequest
): Promise<{ transaction_id: string; new_balance: number }> {
  return handleAction(
    (apiClient) => {
      const creditsApi = createCreditsApi(apiClient);
      return creditsApi.adminRevokeCredits(request);
    },
    'Failed to revoke credits'
  );
}
