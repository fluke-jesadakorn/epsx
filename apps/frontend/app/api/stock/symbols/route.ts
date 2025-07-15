import { NextRequest, NextResponse } from 'next/server';
import { rankStocksByEpsWithChart } from '@/utils/processStocks/rankingStocks';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const limit = parseInt(searchParams.get('limit') || '10');
    const skip = parseInt(searchParams.get('skip') || '0');
    
    // Get the ranking data but only return symbols
    const rankingData = await rankStocksByEpsWithChart(skip, limit);
    
    // Extract just the symbols in ranked order
    const symbols = Object.keys(rankingData);
    
    return NextResponse.json({ symbols, count: symbols.length });
  } catch (error) {
    console.error('Error fetching symbols:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
