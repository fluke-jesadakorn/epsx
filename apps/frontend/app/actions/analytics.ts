'use server';

import type { EPSRankingsResponse, AnalyticsFilters, FilterOptions, EPSRanking } from '@/types/analytics';
import type { CardDashboardResponse } from '@/types/financialChartData';

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

export async function fetchEPSRankings(filters: AnalyticsFilters): Promise<EPSRankingsResponse | null> {
  try {
    const apiUrl = 'https://api.epsx.io';
    
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
    
    console.log('Fetching EPS rankings from:', url);

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
      return null;
    }

    const apiResponse: CardDashboardResponse = await response.json();
    
    // Validate response structure
    if (!apiResponse.success || !apiResponse.data || !Array.isArray(apiResponse.data)) {
      console.error('Invalid API response structure:', apiResponse);
      return null;
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
        qoq_growth: latestQuarterly.eps_growth || null,
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

    return epsResponse;
  } catch (error) {
    console.error('Error fetching EPS rankings:', error);
    return null;
  }
}

// Interface for backend country data
interface CountryData {
  value: string;  // API value (lowercase)
  label: string;  // Display name
}

export async function fetchFilterOptions(): Promise<FilterOptions> {
  try {
    const apiUrl = 'https://api.epsx.io';
    
    // Fetch countries, sectors, exchanges, and stock types in parallel
    const [countriesResponse, sectorsResponse, exchangesResponse, stockTypesResponse] = await Promise.all([
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/countries`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        next: { revalidate: 3600 }, // Cache for 1 hour
      }),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/sectors`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        next: { revalidate: 3600 }, // Cache for 1 hour
      }),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/exchanges`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        next: { revalidate: 3600 }, // Cache for 1 hour
      }),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/stock-types`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
        next: { revalidate: 3600 }, // Cache for 1 hour
      }),
    ]);

    let countries: string[] = [];
    let sectors: string[] = [];
    let exchanges: string[] = [];
    let stock_types: string[] = [];

    if (countriesResponse.ok) {
      const countriesData = await countriesResponse.json();
      if (Array.isArray(countriesData.countries)) {
        // Handle new format with value/label objects
        if (countriesData.countries.length > 0 && typeof countriesData.countries[0] === 'object') {
          countries = (countriesData.countries as CountryData[]).map((c: CountryData) => c.label);
        } else {
          // Fallback for old format (just strings)
          countries = countriesData.countries;
        }
      }
    }

    if (sectorsResponse.ok) {
      const sectorsData = await sectorsResponse.json();
      sectors = Array.isArray(sectorsData.sectors) ? sectorsData.sectors : [];
    }

    if (exchangesResponse.ok) {
      const exchangesData = await exchangesResponse.json();
      exchanges = Array.isArray(exchangesData.exchanges) ? exchangesData.exchanges : [];
    }

    if (stockTypesResponse.ok) {
      const stockTypesData = await stockTypesResponse.json();
      stock_types = Array.isArray(stockTypesData.stock_types) ? stockTypesData.stock_types : [];
    }

    // Fallback data if API calls fail
    if (countries.length === 0) {
      countries = ['United States', 'Canada', 'United Kingdom', 'Germany', 'France', 'Japan', 'Australia'];
    }

    if (sectors.length === 0) {
      sectors = [
        'Technology',
        'Healthcare',
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ];
    }

    if (exchanges.length === 0) {
      exchanges = ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'];
    }

    if (stock_types.length === 0) {
      stock_types = ['common', 'preferred', 'reit', 'etf'];
    }

    return { countries, sectors, exchanges, stock_types };
  } catch (error) {
    console.error('Error fetching filter options:', error);
    
    // Return fallback data with proper display names
    return {
      countries: ['United States', 'Canada', 'United Kingdom', 'Germany', 'France', 'Japan', 'Australia'],
      sectors: [
        'Technology',
        'Healthcare', 
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ],
      exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
      stock_types: ['common', 'preferred', 'reit', 'etf'],
    };
  }
}