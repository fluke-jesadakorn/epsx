import { NextRequest, NextResponse } from 'next/server';
import { adminLogger } from '@/lib/logger';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    // Get search params from the original request
    const { searchParams } = new URL(request.url);
    const queryString = searchParams.toString();
    
    // Forward the request to the Rust backend with cookies
    const backendUrl = `${BACKEND_URL}/admin/stats${queryString ? `?${queryString}` : ''}`;
    const cookieHeader = request.headers.get('cookie') || '';
    
    adminLogger.debug('Admin stats proxy: forwarding request', { backendUrl, hasCookies: !!cookieHeader }, 'AdminStatsRoute');
    
    try {
      const response = await fetch(backendUrl, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Cookie': cookieHeader,
        },
      });

      adminLogger.debug('Admin stats proxy: backend response received', { status: response.status }, 'AdminStatsRoute');

      if (!response.ok) {
        const error = await response.text();
        adminLogger.error('Admin stats proxy: backend error', { error, status: response.status }, 'AdminStatsRoute');
        
        // In development, return mock data if backend returns 401 (not authenticated)
        if (process.env.NODE_ENV === 'development' && response.status === 401) {
          adminLogger.info('Admin stats proxy: backend auth failed, returning mock data for development', {}, 'AdminStatsRoute');
          const mockStats = {
            totalUsers: 150,
            verifiedUsers: 120,
            disabledUsers: 5,
            adminUsers: 3,
            verificationRate: 80.0
          };
          return NextResponse.json(mockStats);
        }
        
        return NextResponse.json(
          { error: error || 'Failed to fetch user statistics' },
          { status: response.status }
        );
      }

      const data = await response.json();
      adminLogger.debug('Admin stats proxy: backend data received', { dataKeys: Object.keys(data) }, 'AdminStatsRoute');
      return NextResponse.json(data);
    } catch (fetchError) {
      adminLogger.error('Admin stats proxy: backend not available', { error: fetchError instanceof Error ? fetchError.message : fetchError }, 'AdminStatsRoute');
      
      // For development, return mock data if backend is not available
      if (process.env.NODE_ENV === 'development') {
        adminLogger.info('Admin stats proxy: returning mock data for development', {}, 'AdminStatsRoute');
        const mockStats = {
          totalUsers: 150,
          verifiedUsers: 120,
          disabledUsers: 5,
          adminUsers: 3,
          verificationRate: 80.0
        };
        return NextResponse.json(mockStats);
      }
      
      throw fetchError;
    }
  } catch (error) {
    adminLogger.error('Admin stats proxy error', { error: error instanceof Error ? error.message : error }, 'AdminStatsRoute');
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}