import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialData } from '@/lib/services/stock.service';
import { StockDataCache } from '@/utils/cache/stockDataCache';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const symbol = searchParams.get('symbol');
    
    if (!symbol) {
      return NextResponse.json(
        { error: 'Symbol parameter is required' },
        { status: 400 }
      );
    }

    // Check per-card cache first
    const cachedData = StockDataCache.get(symbol);
    if (cachedData) {
      return NextResponse.json(cachedData);
    }

    // If not in cache, fetch from server
    // We'll get all data and filter for the specific symbol
    const allData = await getStockFinancialData(0, 20); // Get enough data to find the symbol
    const symbolData = allData.find(item => item.symbol === symbol);

    if (!symbolData) {
      return NextResponse.json(
        { error: `No data found for symbol: ${symbol}` },
        { status: 404 }
      );
    }

    // Cache the individual symbol data
    StockDataCache.set(symbol, symbolData);

    return NextResponse.json(symbolData);
  } catch (error) {
    console.error('Error fetching individual stock data:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
