'use server';

import { CookieManager } from './cookie-manager';

import type {
  AdminProfile,
  AdminUser,
  AnalyticsStatistics,
  ApiResponse,
  AssignmentResult,
  PermissionProfile,
  PermissionProfileAssignmentRequest,
  RegisterRequest,
  StockRankingAnalytics,
  StockRankingAssignment,
  StockRankingAssignmentExtendRequest,
  StockRankingAssignmentRequest,
  StockRankingAssignmentUpdateRequest,
  UserListOptions,
  UserListResult,
  UserProfile,
  UserSoftDeleteRequest,
} from './types';

// Helper function to get backend URL from environment
function getBackendUrl(): string {
  return process.env.BACKEND_URL || process.env.API_URL || 'http://localhost:8080';
}

/**
 * Server-only API methods that can only be used in Next.js server components
 * and API routes. These methods use the 'use server' directive.
 */

export async function serverRegister(userData: RegisterRequest): Promise<
  ApiResponse<{
    user_id: string;
    email: string;
    verification_sent: boolean;
    message: string;
  }>
> {
  return serverRequest('/api/v1/auth/register', 'POST', userData);
}

export async function serverGetAdminConfig(): Promise<ApiResponse<{ adminUrl: string }>> {
  const adminFrontendUrl =
    process.env.ADMIN_FRONTEND_URL || 'http://localhost:3001';
  return { data: { adminUrl: adminFrontendUrl } };
}

export async function serverGetVapidKey(): Promise<ApiResponse<{ vapidPublicKey: string }>> {
  const vapidPublicKey = process.env.VAPID_PUBLIC_KEY || '';

  if (!vapidPublicKey) {
    return {
      error: 'VAPID key not configured',
      details: 'HTTP 503',
    };
  }

  return { data: { vapidPublicKey } };
}

export async function serverGetStockSymbols(): Promise<ApiResponse<unknown>> {
  return serverRequest('/api/market-data/symbols');
}

export async function serverGetIndividualStock(symbol: string): Promise<ApiResponse<unknown>> {
  return serverRequest(
    `/api/market-data/stocks/individual?symbol=${symbol}`
  );
}

export async function serverBatchStocks(symbols: string[]): Promise<ApiResponse<unknown>> {
  return serverRequest('/api/market-data/stocks/batch', 'POST', {
    symbols,
  });
}

export async function serverGetPremiumRankings(): Promise<ApiResponse<unknown>> {
  return serverRequest('/api/premium/rankings');
}

export async function serverGetSystemCache(): Promise<ApiResponse<unknown>> {
  return serverRequest('/api/system/cache');
}

export async function serverGetCurrentUser(): Promise<ApiResponse<UserProfile>> {
  return serverRequest('/auth/me');
}

export async function serverGetAuditLogs(endpoint: string): Promise<ApiResponse<unknown>> {
  return serverRequest(endpoint);
}

export async function serverCreateMusePayPayment(data: unknown): Promise<ApiResponse<unknown>> {
  return serverRequest('/payments/musepay/create', 'POST', data);
}

export async function serverCreateCryptoPayment(data: unknown): Promise<ApiResponse<unknown>> {
  return serverRequest(
    '/api/payments/crypto/deposit-address',
    'POST',
    data
  );
}

export async function serverCreateCryptoDepositAddress(data: unknown): Promise<ApiResponse<unknown>> {
  return serverRequest(
    '/api/v1/payments/crypto/deposit-address',
    'POST',
    data
  );
}

export async function serverLogin(credentials: {
  token: string;
}): Promise<ApiResponse<UserProfile>> {
  return serverRequest('/api/v1/auth/login', 'POST', credentials);
}

// Admin methods
export async function serverListUsers(
  options: UserListOptions = {}
): Promise<ApiResponse<UserListResult>> {
  const params = new URLSearchParams();
  if (options.maxResults || options.limit) {
    params.set('limit', (options.maxResults || options.limit || 50).toString());
  }
  if (options.pageToken || options.offset) {
    params.set('offset', options.pageToken || options.offset || '0');
  }

  return serverRequest(`/api/admin/users?${params.toString()}`);
}

export async function serverGetUser(uid: string): Promise<ApiResponse<AdminUser>> {
  return serverRequest(`/api/admin/users/${uid}`);
}

export async function serverSetUserRole(
  uid: string,
  role: string,
  reason?: string
): Promise<ApiResponse<void>> {
  return serverRequest(`/api/admin/users/${uid}`, 'PUT', {
    role,
    reason: reason || 'Role updated via admin panel',
  });
}

export async function serverGetUserStats(): Promise<ApiResponse<unknown>> {
  return serverRequest(
    '/api/admin/analytics/user-statistics?include_roles=true&include_tiers=true'
  );
}

export async function serverBulkUpdateUserRoles(
  updates: Array<{
    uid: string;
    role: string;
    reason?: string;
  }>
): Promise<ApiResponse<unknown>> {
  return serverRequest('/api/admin/users/batch-update-roles', 'POST', {
    updates: updates.map(update => ({
      user_id: update.uid,
      role: update.role,
      reason: update.reason || 'Bulk update via admin panel',
    })),
  });
}

export async function serverGetAdminUsers(
  searchParams?: URLSearchParams
): Promise<ApiResponse<{ users: AdminUser[]; total?: number }>> {
  const queryString = searchParams?.toString() || '';
  return serverRequest(
    `/api/admin/users${queryString ? `?${queryString}` : ''}`
  );
}

export async function serverGetAdminUser(userId: string): Promise<ApiResponse<AdminUser>> {
  return serverRequest(`/api/admin/users/${userId}`);
}

export async function serverGetAdminPermissionProfiles(searchParams?: URLSearchParams): Promise<
  ApiResponse<{
    permission_profiles: PermissionProfile[];
    total: number;
    limit: number;
    offset: number;
  }>
> {
  const params = new URLSearchParams(searchParams);
  return serverRequest(`/api/admin/permission-profiles?${params.toString()}`);
}

export async function serverGetAdminPermissionProfile(
  profileId: string
): Promise<ApiResponse<PermissionProfile>> {
  return serverRequest(`/api/admin/permission-profiles/${profileId}`);
}

export async function serverAssignAdminPermissionProfile(request: {
  profile_id: string;
  user_id: string;
  expires_at?: string;
}): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest('/api/admin/permission-profiles/assign', 'POST', request);
}

export async function serverGetStockRankingAssignments(
  searchParams?: URLSearchParams
): Promise<
  ApiResponse<{ assignments: StockRankingAssignment[]; total?: number }>
> {
  const params = new URLSearchParams(searchParams);
  return serverRequest(`/api/admin/stock-ranking/assignments?${params.toString()}`);
}

export async function serverGetStockRankingAssignment(
  assignmentId: string
): Promise<ApiResponse<StockRankingAssignment>> {
  return serverRequest(`/api/admin/stock-ranking/assignments/${assignmentId}`);
}

export async function serverAssignBulkStockRanking(
  request: StockRankingAssignmentRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest('/api/admin/stock-ranking/assign-bulk', 'POST', request);
}

export async function serverRevokeStockRankingAssignment(
  assignmentId: string
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest(
    `/api/admin/stock-ranking/assignments/${assignmentId}/revoke`,
    'POST'
  );
}

export async function serverExtendStockRankingAssignment(
  assignmentId: string,
  request: StockRankingAssignmentExtendRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest(
    `/api/admin/stock-ranking/assignments/${assignmentId}/extend`,
    'POST',
    request
  );
}

export async function serverUpdateStockRankingAssignment(
  assignmentId: string,
  request: StockRankingAssignmentUpdateRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest(
    `/api/admin/stock-ranking/assignments/${assignmentId}`,
    'PUT',
    request
  );
}

export async function serverGetAnalyticsStatistics(): Promise<ApiResponse<AnalyticsStatistics>> {
  return serverRequest('/api/admin/analytics/statistics');
}

export async function serverGetStockRankingAnalytics(
  searchParams?: URLSearchParams
): Promise<ApiResponse<StockRankingAnalytics>> {
  const params = new URLSearchParams(searchParams);
  return serverRequest(
    `/api/admin/stock-ranking/analytics?${params.toString()}`
  );
}

export async function serverGetAdminProfile(): Promise<ApiResponse<AdminProfile>> {
  return serverRequest('/api/admin/auth/profile');
}

export async function serverSoftDeleteUser(
  userId: string,
  request: UserSoftDeleteRequest
): Promise<ApiResponse<{ message: string }>> {
  return serverRequest(`/api/admin/users/${userId}`, 'DELETE', request);
}

export async function serverAssignPermissionProfile(
  request: PermissionProfileAssignmentRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest('/api/admin/permission-profiles/assign', 'POST', request);
}

// Generic server request helper function
async function serverRequest<T>(
  endpoint: string,
  method: string = 'GET',
  data?: unknown
): Promise<ApiResponse<T>> {
  const url = `${getBackendUrl()}${endpoint}`;
  
  try {
    // Get authentication headers and cookies for debugging
    const authHeaders = await CookieManager.buildAuthHeaders();
    const authCookies = await CookieManager.getAuthCookies();
    
    // Build Cookie header manually for server-to-server requests
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();
    const allCookies = cookieStore.getAll();
    const cookieHeader = allCookies.map((c: { name: string; value: string }) => `${c.name}=${c.value}`).join('; ');
    
    console.log('🍪 [serverRequest] Cookie header built:', {
      totalCookies: allCookies.length,
      cookieHeaderLength: cookieHeader.length,
      cookiePreview: cookieHeader.substring(0, 100) + (cookieHeader.length > 100 ? '...' : ''),
    });
    
    // Debug: Log request details
    console.log('🚀 [serverRequest] Request details:', {
      url,
      method,
      endpoint,
      backendUrl: getBackendUrl(),
      authHeaders,
      cookiesFound: {
        session: !!authCookies.session,
        adminSession: !!authCookies.adminSession,
        csrf: !!authCookies.csrf,
        refresh: !!authCookies.refresh,
      },
      cookieValues: {
        session: authCookies.session ? `${authCookies.session.substring(0, 20)}...` : null,
        adminSession: authCookies.adminSession ? `${authCookies.adminSession.substring(0, 20)}...` : null,
        csrf: authCookies.csrf ? `${authCookies.csrf.substring(0, 20)}...` : null,
        refresh: authCookies.refresh ? `${authCookies.refresh.substring(0, 20)}...` : null,
      },
      hasRequestBody: !!data && method !== 'GET',
    });
    
    const requestConfig: RequestInit = {
      method,
      credentials: 'include', // Include cookies for authentication
      headers: {
        'Content-Type': 'application/json',
        'Cookie': cookieHeader, // Manually add cookies for server-to-server
        ...authHeaders,
      },
    };

    if (data && method !== 'GET') {
      requestConfig.body = JSON.stringify(data);
      console.log('📤 [serverRequest] Request body:', JSON.stringify(data, null, 2));
    }

    console.log('📡 [serverRequest] Making request with config:', {
      method: requestConfig.method,
      credentials: requestConfig.credentials,
      headers: requestConfig.headers,
    });

    const response = await fetch(url, requestConfig);

    // Debug: Log response details
    console.log('📨 [serverRequest] Response received:', {
      status: response.status,
      statusText: response.statusText,
      ok: response.ok,
      headers: Object.fromEntries(response.headers.entries()),
      url: response.url,
    });

    let responseData: unknown;
    const contentType = response.headers.get('content-type');

    if (contentType?.includes('application/json')) {
      responseData = await response.json() as unknown;
      console.log('📄 [serverRequest] JSON response data:', responseData);
    } else {
      responseData = await response.text();
      console.log('📄 [serverRequest] Text response data:', responseData);
    }

    if (!response.ok) {
      const errorResponse = {
        error: (responseData as Record<string, unknown>)?.error || responseData || 'Request failed',
        details: (responseData as Record<string, unknown>)?.details || `HTTP ${response.status}`,
      };
      
      console.error('❌ [serverRequest] Request failed:', {
        url,
        status: response.status,
        statusText: response.statusText,
        errorResponse,
        responseData: responseData as unknown,
      });
      
      return errorResponse;
    }

    console.log('✅ [serverRequest] Request successful:', {
      url,
      status: response.status,
      dataType: typeof responseData,
      dataKeys: responseData && typeof responseData === 'object' ? Object.keys(responseData) : 'N/A',
    });

    return { data: responseData as unknown };
  } catch (error) {
    const errorDetails = error instanceof Error ? error.message : 'Unknown error';
    
    console.error('💥 [serverRequest] Network/fetch error:', {
      url,
      method,
      error: errorDetails,
      stack: error instanceof Error ? error.stack : undefined,
    });
    
    return {
      error: 'Network error',
      details: errorDetails,
    };
  }
}