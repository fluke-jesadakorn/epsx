import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialData, getStockFinancialDataCount } from '@/lib/services/stock.service';
import { MarketCountry } from '../../../../../../types/marketCountries';

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);

  const page = parseInt(searchParams.get('page') || '1', 10);
  const limit = parseInt(searchParams.get('limit') || '10', 10);
  const countryParam = searchParams.get('country');
  const quarters = parseInt(searchParams.get('quarters') || '2', 10);

  // Validate pagination parameters
  if (page < 1) {
    return NextResponse.json(
      { error: 'Page must be greater than 0' },
      { status: 400 }
    );
  }

  if (limit < 1 || limit > 100) {
    return NextResponse.json(
      { error: 'Limit must be between 1 and 100' },
      { status: 400 }
    );
  }

  // Map string to MarketCountry value if valid
  let country: typeof MarketCountry | undefined = MarketCountry;
  if (countryParam && Object.values(MarketCountry).includes(countryParam as any)) {
    country = (MarketCountry as any)[countryParam] || MarketCountry;
  }

  try {
    // Calculate skip from page
    const skip = (page - 1) * limit;

    // Fetch data and count in parallel
    const [data, totalCount] = await Promise.all([
      getStockFinancialData(skip, limit, country, quarters),
      getStockFinancialDataCount(country, quarters)
    ]);

    // Calculate pagination metadata
    const totalPages = Math.ceil(totalCount / limit);
    const hasNext = page < totalPages;
    const hasPrev = page > 1;

    return NextResponse.json({
      data,
      pagination: {
        page,
        limit,
        total: totalCount,
        totalPages,
        hasNext,
        hasPrev,
        // Additional helpful metadata
        startIndex: skip + 1,
        endIndex: Math.min(skip + limit, totalCount),
        currentPageSize: data.length
      }
    }, { 
      status: 200,
      headers: {
        'Cache-Control': 'public, max-age=300', // 5 minutes cache
      }
    });
  } catch (error) {
    console.error('Pagination API Error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch paginated stock data' },
      { status: 500 }
    );
  }
}
