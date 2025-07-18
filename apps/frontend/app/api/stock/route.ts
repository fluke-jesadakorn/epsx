import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialData, getStockFinancialDataCount } from '@/lib/services/stock.service';
import { MarketCountry } from '../../../../../types/marketCountries';

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);

  const skip = parseInt(searchParams.get('skip') || '0', 10);
  const limit = parseInt(searchParams.get('limit') || '10', 10);
  const page = parseInt(searchParams.get('page') || '1', 10);
  const countryParam = searchParams.get('country');
  const quarters = parseInt(searchParams.get('quarters') || '2', 10);
  const withPagination = searchParams.get('paginated') === 'true';

  // Map string to MarketCountry value if valid
  let country: typeof MarketCountry | undefined = MarketCountry;
  if (countryParam && Object.values(MarketCountry).includes(countryParam as any)) {
    country = (MarketCountry as any)[countryParam] || MarketCountry;
  }

  try {
    // Calculate skip from page if page is provided
    const actualSkip = page > 1 ? (page - 1) * limit : skip;

    if (withPagination) {
      // Return paginated response with metadata
      const [data, totalCount] = await Promise.all([
        getStockFinancialData(actualSkip, limit, country, quarters),
        getStockFinancialDataCount(country, quarters)
      ]);

      return NextResponse.json({
        data,
        pagination: {
          page: page > 1 ? page : Math.floor(actualSkip / limit) + 1,
          limit,
          total: totalCount,
          totalPages: Math.ceil(totalCount / limit),
          hasNext: (actualSkip + limit) < totalCount,
          hasPrev: actualSkip > 0,
        }
      }, { status: 200 });
    } else {
      // Return simple data array for backward compatibility
      const data = await getStockFinancialData(actualSkip, limit, country, quarters);
      return NextResponse.json(data, { status: 200 });
    }
  } catch (error) {
    console.error('API Error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stock financial data' },
      { status: 500 }
    );
  }
}
