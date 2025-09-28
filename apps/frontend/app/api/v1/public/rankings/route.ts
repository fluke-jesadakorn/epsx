/**
 * Public Rankings API Route
 * Fetches public (unauthenticated) rankings data
 * Aligns with backend /api/v1/public/analytics/rankings
 */
import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    
    // Extract basic filters for public access
    const params = new URLSearchParams({
      page: searchParams.get('page') || '1',
      limit: Math.min(parseInt(searchParams.get('limit') || '10'), 20).toString(), // Limit public access to 20 max
    });

    // Add basic filters
    if (searchParams.get('country')) {
      params.append('country', searchParams.get('country')!.toLowerCase());
    }
    if (searchParams.get('sector')) {
      params.append('sector', searchParams.get('sector')!);
    }

    const url = `${BACKEND_URL}/api/v1/public/analytics/rankings?${params.toString()}`;

    console.log('🔍 Fetching public rankings data');

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'Cache-Control': 'no-cache',
      },
      next: { revalidate: 300 }, // Cache for 5 minutes for public data
    });

    if (!response.ok) {
      console.error(`Failed to fetch public rankings: ${response.status} ${response.statusText}`);
      return NextResponse.json({ error: 'Failed to fetch rankings' }, { status: response.status });
    }

    const rankingsData = await response.json();
    
    return NextResponse.json(rankingsData);
  } catch (error) {
    console.error('Error fetching public rankings:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}