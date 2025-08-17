import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    
    // Get query parameters
    const page = searchParams.get('page') || '1';
    const limit = searchParams.get('limit') || '12';
    const sort_by = searchParams.get('sort_by') || 'market_cap';
    const country = searchParams.get('country');
    const sector = searchParams.get('sector');
    const min_eps = searchParams.get('min_eps');
    const min_growth = searchParams.get('min_growth');

    // Build backend URL - hardcoded for development since env var loading issue
    const backendUrl = 'http://localhost:8080';
    const params = new URLSearchParams({
      page,
      limit,
      sort_by,
      ...(country && { country }),
      ...(sector && { sector }),
      ...(min_eps && { min_eps }),
      ...(min_growth && { min_growth }),
    });

    const url = `${backendUrl}/api/v1/analytics/rankings?${params.toString()}`;
    
    console.log('🔄 Proxying request to backend:', url);
    console.log('🔧 Available env vars:', { 
      API_URL: process.env.API_URL, 
      NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL 
    });

    // Make request to backend
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('❌ Backend response error:', {
        status: response.status,
        statusText: response.statusText,
        url: url,
        errorText: errorText
      });
      throw new Error(`Backend API error: ${response.status} - ${errorText}`);
    }

    const data = await response.json();
    console.log('✅ Backend response received successfully');

    // Return the data with CORS headers
    return NextResponse.json(data, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type, Authorization',
      },
    });
  } catch (error) {
    console.error('❌ Analytics proxy error:', error);
    return NextResponse.json(
      { success: false, error: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

export async function OPTIONS() {
  return new NextResponse(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  });
}