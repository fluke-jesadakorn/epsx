/**
 * Analytics Rankings API Route
 * Fetches EPS rankings and analytics data
 * Aligns with backend /api/v1/analytics/rankings
 */
import { NextRequest, NextResponse } from 'next/server';
import type { EPSRankingsResponse, AnalyticsFilters, EPSRanking } from '@/types/analytics';
import type { CardDashboardResponse } from '@/types/financialChartData';
import { env } from '@/config/env';
import { cookies } from 'next/headers';

const BACKEND_URL = env.BACKEND_URL;

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

    // Build query parameters
    const params = new URLSearchParams({
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sort_by: filters.sort_by,
    });

    // Add optional filters
    if (filters.country) params.append('country', filters.country.toLowerCase());
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

    const url = `${BACKEND_URL}/api/v1/analytics/rankings?${params.toString()}`;

    // Get Bearer token for authentication if available
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;

    const headers: Record<string, string> = {
      'Accept': 'application/json',
      'Cache-Control': 'no-cache',
    };

    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    let response = await fetch(url, {
      method: 'GET',
      headers,
      next: { revalidate: 60 }, // Cache for 1 minute
    });

    // If authenticated request fails with 401, try public endpoint
    if (!response.ok && response.status === 401) {
      console.log('Authenticated request failed, trying public endpoint...');
      const publicUrl = `${BACKEND_URL}/api/v1/public/analytics/rankings?${params.toString()}`;
      const publicHeaders = {
        'Accept': 'application/json',
        'Cache-Control': 'no-cache',
      };
      
      response = await fetch(publicUrl, {
        method: 'GET',
        headers: publicHeaders,
        next: { revalidate: 60 },
      });
    }

    if (!response.ok) {
      console.error(`Failed to fetch EPS rankings: ${response.status} ${response.statusText}`);
      return NextResponse.json({ error: 'Failed to fetch rankings' }, { status: response.status });
    }

    const apiResponse = await response.json();
    
    // Handle both authenticated (CardDashboardResponse with .data) and public (with .rankings) responses
    const rankingsData = apiResponse.rankings || apiResponse.data;
    
    // Validate response structure
    if (!rankingsData || !Array.isArray(rankingsData)) {
      console.error('Invalid API response structure:', apiResponse);
      return NextResponse.json({ error: 'Invalid API response' }, { status: 500 });
    }

    // Transform to EPSRankingsResponse format
    const epsRankings: EPSRanking[] = rankingsData.map((card: any, index: number) => {
      const latestQuarterly = card.quarterly_performance[0] || {};
      
      return {
        symbol: card.symbol,
        name: `Company ${card.symbol}`,
        country: 'US',
        sector: 'Technology',
        exchange: 'NASDAQ',
        current_eps: latestQuarterly.eps || null,
        growth_factor: latestQuarterly.eps_growth || null,
        price_current: card.value || null,
        market_cap: null,
        volume: null,
        ranking_position: card.rank || index + 1,
        active_status: card.active_status,
        currency: card.currency || 'USD',
        quarterly_data: card.quarterly_performance.map(q => ({
          quarter: q.quarter,
          date: q.date,
          price: q.price,
          eps: q.eps,
          eps_growth: q.eps_growth,
          price_growth: q.price_growth,
          volume: 0,
        })),
      };
    });

    const epsResponse: EPSRankingsResponse = {
      data: epsRankings,
      pagination: {
        page: apiResponse.pagination?.page || 1,
        limit: apiResponse.pagination?.limit || rankingsData.length,
        total: apiResponse.pagination?.total || rankingsData.length,
        totalPages: apiResponse.pagination?.totalPages || 1,
        hasNext: apiResponse.pagination?.hasNext || false,
        hasPrev: apiResponse.pagination?.hasPrev || false,
      },
    };

    return NextResponse.json(epsResponse);
  } catch (error) {
    console.error('Error fetching EPS rankings:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}