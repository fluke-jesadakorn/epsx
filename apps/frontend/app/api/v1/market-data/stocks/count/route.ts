import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialDataCount } from '@/lib/services/stock.service';
import { MarketCountry } from '../../../../../../types/marketCountries';

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);

  const countryParam = searchParams.get('country');
  const quarters = parseInt(searchParams.get('quarters') || '2', 10);

  // Map string to MarketCountry value if valid
  let country: typeof MarketCountry | undefined = MarketCountry;
  if (countryParam && Object.values(MarketCountry).includes(countryParam as any)) {
    country = (MarketCountry as any)[countryParam] || MarketCountry;
  }

  try {
    const count = await getStockFinancialDataCount(country, quarters);

    return NextResponse.json({
      count,
      timestamp: new Date().toISOString(),
      filters: {
        country: countryParam || 'all',
        quarters
      }
    }, { 
      status: 200,
      headers: {
        'Cache-Control': 'public, max-age=300', // 5 minutes cache
      }
    });
  } catch (error) {
    console.error('Count API Error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stock count' },
      { status: 500 }
    );
  }
}
