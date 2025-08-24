import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/server/auth';
import { env } from '@/config/env';
import type { UnifiedUserData } from '@/lib/types/unified-user';

// Get bearer token from custom JWT session
const getBearerToken = async () => {
  const { getJWTFromCookies } = await import('@/lib/server/jwt');
  return await getJWTFromCookies();
};

const BACKEND_URL = env.BACKEND_URL;

export interface UserListFilters {
  search: string;
  status: string;
  role: string;
  page: number;
  limit: number;
  sortBy: string;
  sortOrder: 'asc' | 'desc';
}

export interface UserListResult {
  users: UnifiedUserData[];
  total: number;
  page: number;
  totalPages: number;
  limit: number;
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    
    // Extract filters from search params with enhanced filtering support
    const filters: UserListFilters = {
      search: searchParams.get('search') || '',
      status: searchParams.get('status') || 'all',
      role: searchParams.get('role') || 'all',
      page: parseInt(searchParams.get('page') || '1', 10),
      limit: Math.min(parseInt(searchParams.get('limit') || '20', 10), 100), // Max 100 per page
      sortBy: searchParams.get('sortBy') || 'created_at',
      sortOrder: (searchParams.get('sortOrder') as 'asc' | 'desc') || 'desc',
    };

    // Validate pagination parameters
    if (filters.page < 1) filters.page = 1;
    if (filters.limit < 1) filters.limit = 20;

    const token = await getBearerToken();
    
    if (!token) {
      return NextResponse.json(
        { 
          success: false, 
          error: { 
            code: 'UNAUTHORIZED', 
            message: 'Authentication required. Please log in again.' 
          } 
        }, 
        { status: 401 }
      );
    }
    
    // Build comprehensive query parameters for enhanced backend endpoint
    const params = new URLSearchParams({
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sortBy: filters.sortBy,
      sortOrder: filters.sortOrder,
    });
    
    // Add optional filters
    if (filters.search) params.set('search', filters.search);
    if (filters.status && filters.status !== 'all') params.set('status', filters.status);
    if (filters.role && filters.role !== 'all') params.set('role', filters.role);
    
    // Add additional filters for enhanced search
    const tier = searchParams.get('tier');
    const emailVerified = searchParams.get('emailVerified');
    const createdAfter = searchParams.get('createdAfter');
    const createdBefore = searchParams.get('createdBefore');
    const lastLoginAfter = searchParams.get('lastLoginAfter');
    const lastLoginBefore = searchParams.get('lastLoginBefore');
    
    if (tier && tier !== 'all') params.set('tier', tier);
    if (emailVerified) params.set('email_verified', emailVerified);
    if (createdAfter) params.set('created_after', createdAfter);
    if (createdBefore) params.set('created_before', createdBefore);
    if (lastLoginAfter) params.set('last_login_after', lastLoginAfter);
    if (lastLoginBefore) params.set('last_login_before', lastLoginBefore);
    
    console.log(`Fetching users from backend: ${BACKEND_URL}/api/v1/admin/users/search?${params}`);
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/search?${params}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      next: { revalidate: 30 }, // Cache for 30 seconds for real-time feel
      signal: AbortSignal.timeout(10000), // 10 second timeout
    });
    
    if (!response.ok) {
      const errorText = await response.text().catch(() => 'Unknown error');
      console.error(`Backend API error: ${response.status} ${response.statusText} - ${errorText}`);
      
      if (response.status === 401) {
        return NextResponse.json(
          { 
            success: false, 
            error: { 
              code: 'UNAUTHORIZED', 
              message: 'Session expired. Please log in again.' 
            } 
          }, 
          { status: 401 }
        );
      }
      
      if (response.status === 403) {
        return NextResponse.json(
          { 
            success: false, 
            error: { 
              code: 'FORBIDDEN', 
              message: 'Insufficient permissions to access user data.' 
            } 
          }, 
          { status: 403 }
        );
      }
      
      if (response.status >= 500) {
        return NextResponse.json(
          { 
            success: false, 
            error: { 
              code: 'SERVER_ERROR', 
              message: 'Backend server is currently unavailable. Please try again later.' 
            } 
          }, 
          { status: 503 }
        );
      }
      
      return NextResponse.json(
        { 
          success: false, 
          error: { 
            code: 'API_ERROR', 
            message: `Backend API returned ${response.status}: ${response.statusText}` 
          } 
        }, 
        { status: response.status }
      );
    }
    
    const data = await response.json();
    console.log(`Successfully fetched ${data.users?.length || 0} users from backend`);
    
    // Transform backend response to match frontend expectations
    const result = {
      success: true,
      data: {
        users: data.users || [],
        total: data.total || 0,
        page: data.page || filters.page,
        totalPages: data.total_pages || Math.ceil((data.total || 0) / filters.limit),
        limit: data.limit || filters.limit,
        hasNextPage: data.has_next_page || false,
        searchQuery: filters.search,
        appliedFilters: {
          status: filters.status,
          role: filters.role,
          tier: tier || 'all',
        }
      }
    };

    return NextResponse.json(result);
    
  } catch (error) {
    console.error('User search API error:', error);
    
    if (error.name === 'AbortError') {
      return NextResponse.json(
        { 
          success: false, 
          error: { 
            code: 'TIMEOUT', 
            message: 'Request timeout. The server is taking too long to respond.' 
          } 
        }, 
        { status: 408 }
      );
    }
    
    if (error.code === 'ECONNREFUSED' || error.code === 'ENOTFOUND') {
      return NextResponse.json(
        { 
          success: false, 
          error: { 
            code: 'CONNECTION_ERROR', 
            message: 'Unable to connect to the backend server. Please check your connection.' 
          } 
        }, 
        { status: 503 }
      );
    }
    
    return NextResponse.json(
      { 
        success: false, 
        error: { 
          code: 'INTERNAL_ERROR', 
          message: 'An unexpected error occurred while fetching users.' 
        } 
      }, 
      { status: 500 }
    );
  }
}

