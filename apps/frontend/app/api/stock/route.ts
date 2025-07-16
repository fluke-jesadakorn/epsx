import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialData } from '@/lib/services/stock.service';
import { MarketCountry } from '../../../../../types/marketCountries';

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);

  const skip = parseInt(searchParams.get('skip') || '0', 10);
  const limit = parseInt(searchParams.get('limit') || '10', 10);
  const countryParam = searchParams.get('country');
  const quarters = parseInt(searchParams.get('quarters') || '2', 10);

  // Map string to MarketCountry value if valid
  let country: typeof MarketCountry | undefined = MarketCountry;
  if (countryParam && Object.values(MarketCountry).includes(countryParam as any)) {
    country = (MarketCountry as any)[countryParam] || MarketCountry;
  }

  try {
    const data = await getStockFinancialData(skip, limit, country, quarters);
    return NextResponse.json(data, { status: 200 });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to fetch stock financial data' },
      { status: 500 }
    );
  }
}
