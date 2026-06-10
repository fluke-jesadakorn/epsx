'use server';

import { rethrowRedirect } from '@/lib/api-error';
import { createUsersClient } from '@/shared/api/users';
import { logger } from '@/lib/logger';
import { getServerActionClient } from '@/shared/utils/server-fetch';
import { revalidatePath } from 'next/cache';

export async function getWatchlistAction(): Promise<string[]> {
  try {
    const client = getServerActionClient();
    const api = createUsersClient(client);
    const res = await api.getWatchlist();
    if (res.success && res.data) {
      return res.data.symbols;
    }
    logger.action.error('getWatchlist', res);
  } catch (e) {
    rethrowRedirect(e);
    logger.action.error('getWatchlist', e);
  }
  return [];
}

export async function addToWatchlistAction(symbol: string): Promise<string[] | null> {
  try {
    const client = getServerActionClient();
    const api = createUsersClient(client);
    const res = await api.addToWatchlist(symbol);
    if (res.success && res.data) {
      revalidatePath('/portfolio');
      return res.data.symbols;
    }
    logger.action.error('addToWatchlist', res);
  } catch (e) {
    rethrowRedirect(e);
    logger.action.error('addToWatchlist', e);
  }
  return null;
}

export async function removeFromWatchlistAction(symbol: string): Promise<string[] | null> {
  try {
    const client = getServerActionClient();
    const api = createUsersClient(client);
    const res = await api.removeFromWatchlist(symbol);
    if (res.success && res.data) {
      revalidatePath('/portfolio');
      return res.data.symbols;
    }
    logger.action.error('removeFromWatchlist', res);
  } catch (e) {
    rethrowRedirect(e);
    logger.action.error('removeFromWatchlist', e);
  }
  return null;
}
