import { rankStocksByEpsWithChart } from './rankingStocks';
import { MarketCountry } from '../../../../types/marketCountries';
import { cache } from './cacheAdapter';

/**
 * Runs once per day via cron.
 * Fetches top 50 ranked stocks and fills the cache.
 */
export async function refreshTop50Cache(): Promise<void> {
  console.log('[CACHE] Refreshing top-50 ranked stocks…');
  try {
    const data = await rankStocksByEpsWithChart(0, 50, MarketCountry, 4);
    await cache.saveTop50(data);
    console.log(`[CACHE] Saved ${Object.keys(data).length} symbols`);
  } catch (err) {
    console.error('[CACHE] refreshTop50Cache failed:', err);
  }
}

/**
 * Frontend can call this instead of rankStocksByEpsWithChart
 * when it only needs the pre-cached data.
 */
export async function getCachedTop50(): Promise<Record<string, any>> {
  return cache.getTop50();
}
