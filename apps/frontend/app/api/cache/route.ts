import { NextRequest, NextResponse } from 'next/server';
import * as cache from '@/utils/cache';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const action = searchParams.get('action');

    switch (action) {
      case 'stats':
        return NextResponse.json(cache.stats());
        
      case 'symbols':
        return NextResponse.json({ 
          symbols: cache.stats().keys.filter(key => key.startsWith('stock_'))
        });
        
      case 'health':
        const stats = cache.stats();
        const isHealthy = stats.size > 0;
        
        return NextResponse.json({
          status: isHealthy ? 'healthy' : 'empty',
          totalKeys: stats.size,
          symbols: stats.keys.filter(key => key.startsWith('stock_')).length
        });
        
      default:
        return NextResponse.json(cache.stats());
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
      // Basic cleanup - current cache doesn't support this
      return NextResponse.json({
        message: 'Cache cleanup not supported with current implementation',
        totalKeys: cache.stats().size,
      });
    }

    if (symbol) {
      // Clear specific symbol - current cache doesn't support this
      return NextResponse.json({
        message: `Symbol-specific cache clear not supported: ${symbol}`,
      });
    } else {
      // Clear all cache - current cache doesn't support this
      return NextResponse.json({
        message: 'Global cache clear not supported with current implementation',
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
