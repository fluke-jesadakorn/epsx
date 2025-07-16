import { NextRequest, NextResponse } from 'next/server';
import { StockDataCache } from '@/utils/cache/stockDataCache';
import { CacheManager } from '@/utils/cache/cacheManager';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const action = searchParams.get('action');

    switch (action) {
      case 'stats':
        return NextResponse.json(CacheManager.getCacheMetrics());
        
      case 'symbols':
        return NextResponse.json({ 
          symbols: StockDataCache.getCachedSymbols() 
        });
        
      case 'health':
        const stats = CacheManager.getStats();
        const metrics = CacheManager.getCacheMetrics();
        const isHealthy = stats.totalEntries > 0 && metrics.hitRate > 0.7;
        
        return NextResponse.json({
          status: isHealthy ? 'healthy' : 'degraded',
          ...metrics,
        });
        
      default:
        return NextResponse.json(CacheManager.getCacheMetrics());
    }
  } catch (error) {
    console.error('Error in cache management API:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function DELETE(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const symbol = searchParams.get('symbol');
    const action = searchParams.get('action');

    if (action === 'cleanup') {
      const beforeSize = StockDataCache.size();
      StockDataCache.cleanup();
      const afterSize = StockDataCache.size();
      
      return NextResponse.json({
        message: 'Cache cleanup completed',
        entriesRemoved: beforeSize - afterSize,
        remainingEntries: afterSize,
      });
    }

    if (symbol) {
      StockDataCache.clear(symbol);
      return NextResponse.json({
        message: `Cache cleared for symbol: ${symbol}`,
      });
    } else {
      StockDataCache.clear();
      return NextResponse.json({
        message: 'All cache cleared',
      });
    }
  } catch (error) {
    console.error('Error clearing cache:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
