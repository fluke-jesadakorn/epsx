import { NextRequest, NextResponse } from 'next/server';
import { adminLogger } from '@/lib/logger';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || process.env.BACKEND_URL;

if (!BACKEND_URL) {
  throw new Error('BACKEND_URL or NEXT_PUBLIC_API_URL environment variable is required');
}

export async function GET(request: NextRequest) {
  try {
    // Get search params from the original request
    const { searchParams } = new URL(request.url);
    const queryString = searchParams.toString();
    
    // Forward the request to the Rust backend with cookies
    const backendUrl = `${BACKEND_URL}/admin/users${queryString ? `?${queryString}` : ''}`;
    const cookieHeader = request.headers.get('cookie') || '';
    
    adminLogger.info('Admin users proxy: forwarding request', { backendUrl, hasCookies: !!cookieHeader }, 'AdminUsersRoute');
    
    const response = await fetch(backendUrl, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
      },
    });

    adminLogger.debug('Admin users proxy: backend response received', { status: response.status }, 'AdminUsersRoute');

    if (!response.ok) {
      const error = await response.text();
      adminLogger.error('Admin users proxy: backend error', { error }, 'AdminUsersRoute');
      
      return NextResponse.json(
        { error: error || 'Failed to fetch users' },
        { status: response.status }
      );
    }

    const data = await response.json();
    adminLogger.debug('Admin users proxy: backend data received', { userCount: data.users?.length || 0 }, 'AdminUsersRoute');
    return NextResponse.json(data);
  } catch (error) {
    adminLogger.error('Admin users proxy error', { error: error instanceof Error ? error.message : error }, 'AdminUsersRoute');
    return NextResponse.json(
      { error: 'Backend connection failed' },
      { status: 503 }
    );
  }
}