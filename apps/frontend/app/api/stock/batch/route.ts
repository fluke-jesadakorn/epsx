import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialData } from '@/lib/services/stock.service';
import { StockDataCache } from '@/utils/cache/stockDataCache';
import { CacheManager } from '@/utils/cache/cacheManager';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const symbolsParam = searchParams.get('symbols');
    
    if (!symbolsParam) {
      return NextResponse.json(
        { error: 'Symbols parameter is required (comma-separated)' },
        { status: 400 }
      );
    }

    const symbols = symbolsParam.split(',').map(s => s.trim()).filter(Boolean);
    
    if (symbols.length === 0) {
      return NextResponse.json(
        { error: 'At least one valid symbol is required' },
        { status: 400 }
      );
    }

    // Check cache for all symbols first
    const cachedResults = CacheManager.getBulk(symbols);
    const result: Record<string, any> = {};
    const missingSymbols: string[] = [];

    // Collect cached data and identify missing symbols
    symbols.forEach(symbol => {
      const cached = cachedResults[symbol];
      if (cached) {
        result[symbol] = cached;
      } else {
        missingSymbols.push(symbol);
      }
    });

    // If we have missing symbols, fetch them from the server
    if (missingSymbols.length > 0) {
      try {
        // Fetch a larger dataset to increase chances of finding the missing symbols
        const allData = await getStockFinancialData(0, 200);
        
        missingSymbols.forEach(symbol => {
          const symbolData = allData.find(item => item.symbol === symbol);
          if (symbolData) {
            // Cache the found data
            StockDataCache.set(symbol, symbolData);
            result[symbol] = symbolData;
          } else {
            result[symbol] = { error: `No data found for symbol: ${symbol}` };
          }
        });
      } catch (fetchError) {
        console.error('Error fetching missing symbols:', fetchError);
        // Mark missing symbols as errors
        missingSymbols.forEach(symbol => {
          result[symbol] = { error: `Failed to fetch data for symbol: ${symbol}` };
        });
      }
    }

    // Return results for all requested symbols
    return NextResponse.json({
      success: true,
      data: result,
      cached: symbols.filter(s => cachedResults[s] !== null),
      fetched: missingSymbols.filter(s => result[s] && !result[s].error),
      errors: symbols.filter(s => result[s] && result[s].error),
    });

  } catch (error) {
    console.error('Error in batch stock API:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { symbols, action } = body;

    if (!symbols || !Array.isArray(symbols)) {
      return NextResponse.json(
        { error: 'Symbols array is required' },
        { status: 400 }
      );
    }

    switch (action) {
      case 'preload':
        await CacheManager.preloadSymbols(symbols);
        return NextResponse.json({
          message: `Preload initiated for ${symbols.length} symbols`,
          symbols,
        });

      case 'cache_status':
        const cachedResults = CacheManager.getBulk(symbols);
        const status = symbols.map(symbol => ({
          symbol,
          cached: !!cachedResults[symbol],
          expired: cachedResults[symbol] ? StockDataCache.isExpired(symbol) : true,
        }));
        
        return NextResponse.json({ symbols: status });

      default:
        return NextResponse.json(
          { error: 'Invalid action. Supported actions: preload, cache_status' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Error in batch stock POST API:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
