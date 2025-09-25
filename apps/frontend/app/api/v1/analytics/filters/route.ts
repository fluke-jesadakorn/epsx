/**
 * Analytics Filter Options API Route
 * Fetches available filter options for analytics
 * Aligns with backend /api/v1/analytics/filters
 */
import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';
import { cookies } from 'next/headers';

const BACKEND_URL = env.BACKEND_URL;

export async function GET(request: NextRequest) {
  try {
    console.log('🔍 Fetching analytics filter options');

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

    const response = await fetch(`${BACKEND_URL}/api/v1/analytics/filters`, {
      method: 'GET',
      headers,
      next: { revalidate: 300 }, // Cache for 5 minutes
    });

    if (!response.ok) {
      console.error(`Failed to fetch filter options: ${response.status} ${response.statusText}`);
      return NextResponse.json({ error: 'Failed to fetch filter options' }, { status: response.status });
    }

    const filterData = await response.json();
    
    return NextResponse.json(filterData);
  } catch (error) {
    console.error('Error fetching filter options:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}