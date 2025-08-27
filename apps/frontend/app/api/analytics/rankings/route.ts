import { NextRequest, NextResponse } from 'next/server';
import type { EPSRankingsResponse, AnalyticsFilters, EPSRanking } from '@/types/analytics';
import type { CardDashboardResponse } from '@/types/financialChartData';
import { serverConfig } from '@/config/env';

// Country name mapping for normalization - maps display names to API values
const countryValueMap: { [key: string]: string } = {
  'United States': 'america',
  'Argentina': 'argentina',
  'Australia': 'australia', 
  'Austria': 'austria',
  'Bahrain': 'bahrain',
  'Bangladesh': 'bangladesh',
  'Belgium': 'belgium',
  'Brazil': 'brazil',
  'Canada': 'canada',
  'Chile': 'chile',
  'China': 'china',
  'Colombia': 'colombia',
  'Cyprus': 'cyprus',
  'Czech Republic': 'czech',
  'Denmark': 'denmark',
  'Egypt': 'egypt',
  'Estonia': 'estonia',
  'Finland': 'finland',
  'France': 'france',
  'Germany': 'germany',
  'Greece': 'greece',
  'Hong Kong': 'hongkong',
  'Hungary': 'hungary',
  'Iceland': 'iceland',
  'India': 'india',
  'Indonesia': 'indonesia',
  'Ireland': 'ireland',
  'Israel': 'israel',
  'Italy': 'italy',
  'Japan': 'japan',
  'Kenya': 'kenya',
  'Kuwait': 'kuwait',
  'Latvia': 'latvia',
  'Lithuania': 'lithuania',
  'Luxembourg': 'luxembourg',
  'Malaysia': 'malaysia',
  'Mexico': 'mexico',
  'Morocco': 'morocco',
  'Netherlands': 'netherlands',
  'New Zealand': 'newzealand',
  'Nigeria': 'nigeria',
  'Norway': 'norway',
  'Pakistan': 'pakistan',
  'Peru': 'peru',
  'Philippines': 'philippines',
  'Poland': 'poland',
  'Portugal': 'portugal',
  'Qatar': 'qatar',
  'Romania': 'romania',
  'Russia': 'russia',
  'Saudi Arabia': 'ksa',
  'Serbia': 'serbia',
  'Singapore': 'singapore',
  'Slovakia': 'slovakia',
  'South Africa': 'rsa',
  'South Korea': 'korea',
  'Spain': 'spain',
  'Sri Lanka': 'srilanka',
  'Sweden': 'sweden',
  'Switzerland': 'switzerland',
  'Taiwan': 'taiwan',
  'Thailand': 'thailand',
  'Tunisia': 'tunisia',
  'Turkey': 'turkey',
  'United Arab Emirates': 'uae',
  'United Kingdom': 'uk',
  'UK': 'uk',
  'Venezuela': 'venezuela',
  'Vietnam': 'vietnam',
};

// Function to normalize country names to lowercase API values
function normalizeCountryName(country: string): string {
  // First check if it's already a mapped value (lowercase)
  if (country && country === country.toLowerCase()) {
    return country;
  }
  
  // Check direct mapping
  if (countryValueMap[country]) {
    return countryValueMap[country];
  }
  
  // Default to lowercase if no mapping found
  return country?.toLowerCase() || '';
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    
    // Extract filters from search params
    const filters: AnalyticsFilters = {
      page: parseInt(searchParams.get('page') || '1'),
      limit: parseInt(searchParams.get('limit') || '20'),
      sort_by: (searchParams.get('sort_by') as any) || 'eps_growth',
      country: searchParams.get('country') || undefined,
      sector: searchParams.get('sector') || undefined,
      min_eps: searchParams.get('min_eps') ? parseFloat(searchParams.get('min_eps')!) : undefined,
      max_eps: searchParams.get('max_eps') ? parseFloat(searchParams.get('max_eps')!) : undefined,
      min_growth: searchParams.get('min_growth') ? parseFloat(searchParams.get('min_growth')!) : undefined,
      max_growth: searchParams.get('max_growth') ? parseFloat(searchParams.get('max_growth')!) : undefined,
      min_market_cap: searchParams.get('min_market_cap') ? parseFloat(searchParams.get('min_market_cap')!) : undefined,
      max_market_cap: searchParams.get('max_market_cap') ? parseFloat(searchParams.get('max_market_cap')!) : undefined,
      min_volume: searchParams.get('min_volume') ? parseFloat(searchParams.get('min_volume')!) : undefined,
      max_volume: searchParams.get('max_volume') ? parseFloat(searchParams.get('max_volume')!) : undefined,
      min_price: searchParams.get('min_price') ? parseFloat(searchParams.get('min_price')!) : undefined,
      max_price: searchParams.get('max_price') ? parseFloat(searchParams.get('max_price')!) : undefined,
      min_pe_ratio: searchParams.get('min_pe_ratio') ? parseFloat(searchParams.get('min_pe_ratio')!) : undefined,
      max_pe_ratio: searchParams.get('max_pe_ratio') ? parseFloat(searchParams.get('max_pe_ratio')!) : undefined,
      min_dividend_yield: searchParams.get('min_dividend_yield') ? parseFloat(searchParams.get('min_dividend_yield')!) : undefined,
      max_dividend_yield: searchParams.get('max_dividend_yield') ? parseFloat(searchParams.get('max_dividend_yield')!) : undefined,
      exchange: searchParams.get('exchange') || undefined,
      stock_type: (searchParams.get('stock_type') as any) || undefined,
    };

    const apiUrl = serverConfig.backendUrl;
    
    // Build query parameters
    const params = new URLSearchParams({
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sort_by: filters.sort_by,
    });

    // Normalize country name to lowercase API value
    if (filters.country) {
      const normalizedCountry = normalizeCountryName(filters.country);
      params.append('country', normalizedCountry);
    }
    if (filters.sector) params.append('sector', filters.sector);
    if (filters.min_eps) params.append('min_eps', filters.min_eps.toString());
    if (filters.max_eps) params.append('max_eps', filters.max_eps.toString());
    if (filters.min_growth) params.append('min_growth', filters.min_growth.toString());
    if (filters.max_growth) params.append('max_growth', filters.max_growth.toString());
    if (filters.min_market_cap) params.append('min_market_cap', filters.min_market_cap.toString());
    if (filters.max_market_cap) params.append('max_market_cap', filters.max_market_cap.toString());
    if (filters.min_volume) params.append('min_volume', filters.min_volume.toString());
    if (filters.max_volume) params.append('max_volume', filters.max_volume.toString());
    if (filters.min_price) params.append('min_price', filters.min_price.toString());
    if (filters.max_price) params.append('max_price', filters.max_price.toString());
    if (filters.min_pe_ratio) params.append('min_pe_ratio', filters.min_pe_ratio.toString());
    if (filters.max_pe_ratio) params.append('max_pe_ratio', filters.max_pe_ratio.toString());
    if (filters.min_dividend_yield) params.append('min_dividend_yield', filters.min_dividend_yield.toString());
    if (filters.max_dividend_yield) params.append('max_dividend_yield', filters.max_dividend_yield.toString());
    if (filters.exchange) params.append('exchange', filters.exchange);
    if (filters.stock_type) params.append('stock_type', filters.stock_type);

    const url = `${apiUrl}/api/v1/analytics/rankings?${params.toString()}`;

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'Cache-Control': 'no-cache',
      },
      next: { revalidate: 60 }, // Cache for 1 minute
    });

    if (!response.ok) {
      console.error(`Failed to fetch EPS rankings: ${response.status} ${response.statusText}`);
      return NextResponse.json({ error: 'Failed to fetch rankings' }, { status: response.status });
    }

    const apiResponse: CardDashboardResponse = await response.json();
    
    // Validate response structure
    if (!apiResponse.success || !apiResponse.data || !Array.isArray(apiResponse.data)) {
      console.error('Invalid API response structure:', apiResponse);
      return NextResponse.json({ error: 'Invalid API response' }, { status: 500 });
    }

    // Transform CardDashboardResponse to EPSRankingsResponse format
    const epsRankings: EPSRanking[] = apiResponse.data.map((card, index) => {
      const latestQuarterly = card.quarterly_performance[0] || {};
      
      const transformedCard = {
        symbol: card.symbol,
        name: `Company ${card.symbol}`, // Placeholder - API doesn't return company name
        country: 'US', // Default - will be extracted from metadata if available
        sector: 'Technology', // Default - API doesn't return sector in this format
        exchange: 'NASDAQ', // Default - API doesn't return exchange
        current_eps: latestQuarterly.eps || null,
        growth_factor: latestQuarterly.eps_growth || null,
        price_current: card.value || null,
        market_cap: null, // Not available in card format
        volume: null, // Not available in card format
        ranking_position: card.rank || index + 1,
        active_status: card.active_status, // Active or Non Active from backend
        quarterly_data: card.quarterly_performance.map(q => ({
          quarter: q.quarter,
          date: q.date,
          price: q.price,
          eps: q.eps,
          eps_growth: q.eps_growth,
          price_growth: q.price_growth,
          volume: 0, // Not available
        })),
      };
      
      return transformedCard;
    });

    const epsResponse: EPSRankingsResponse = {
      data: epsRankings,
      pagination: {
        page: apiResponse.pagination.page,
        limit: apiResponse.pagination.limit,
        total: apiResponse.pagination.total,
        totalPages: apiResponse.pagination.totalPages,
        hasNext: apiResponse.pagination.hasNext,
        hasPrev: apiResponse.pagination.hasPrev,
      },
    };

    return NextResponse.json(epsResponse);
  } catch (error) {
    console.error('Error fetching EPS rankings:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}