'use server';

import { createUsersClient } from '@/shared/api/users';
import { logger } from '@/shared/utils/logger';
import { getServerActionClient } from '@/shared/utils/server-fetch';
import { revalidatePath } from 'next/cache';
import type { SymbolCardData } from '@/shared/types/analytics';

export async function getPortfolioOverviewAction(): Promise<{
  watchlist: string[];
  rankings: SymbolCardData[];
}> {
  try {
    const client = getServerActionClient();
    const res = await client.get<{
      watchlist: string[];
      rankings: SymbolCardData[];
    }>('/api/users/portfolio/overview');
    if (res.success && res.data) {
      return res.data;
    }
    logger.debug('Portfolio overview fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch portfolio overview:', e);
  }
  return { watchlist: [], rankings: [] };
}

export async function getWatchlistAction(): Promise<string[]> {
  try {
    const client = getServerActionClient();
    const api = createUsersClient(client);
    const res = await api.getWatchlist();
    if (res.success && res.data) {
      return res.data.symbols;
    }
    logger.debug('Watchlist fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch watchlist:', e);
  }
  return [];
}

export async function addToWatchlistAction(
  symbol: string
): Promise<string[] | null> {
  try {
    const client = getServerActionClient();
    const api = createUsersClient(client);
    const res = await api.addToWatchlist(symbol);
    if (res.success && res.data) {
      revalidatePath('/analytics');
      revalidatePath('/portfolio');
      return res.data.symbols;
    }
    logger.debug('Watchlist add failed:', res);
  } catch (e) {
    logger.debug('Failed to add to watchlist:', e);
  }
  return null;
}

export async function removeFromWatchlistAction(
  symbol: string
): Promise<string[] | null> {
  try {
    const client = getServerActionClient();
    const api = createUsersClient(client);
    const res = await api.removeFromWatchlist(symbol);
    if (res.success && res.data) {
      revalidatePath('/analytics');
      revalidatePath('/portfolio');
      return res.data.symbols;
    }
    logger.debug('Watchlist remove failed:', res);
  } catch (e) {
    logger.debug('Failed to remove from watchlist:', e);
  }
  return null;
}
