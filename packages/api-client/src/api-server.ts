'use server';

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
import { CookieManager } from './cookie-manager';

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

export async function serverGetStockSymbols(): Promise<ApiResponse<any>> {
  return serverRequest('/api/market-data/symbols');
}

export async function serverGetIndividualStock(symbol: string): Promise<ApiResponse<any>> {
  return serverRequest(
    `/api/market-data/stocks/individual?symbol=${symbol}`
  );
}

export async function serverBatchStocks(symbols: string[]): Promise<ApiResponse<any>> {
  return serverRequest('/api/market-data/stocks/batch', 'POST', {
    symbols,
  });
}

export async function serverGetPremiumRankings(): Promise<ApiResponse<any>> {
  return serverRequest('/api/premium/rankings');
}

export async function serverGetSystemCache(): Promise<ApiResponse<any>> {
  return serverRequest('/api/system/cache');
}

export async function serverGetCurrentUser(): Promise<ApiResponse<UserProfile>> {
  return serverRequest('/auth/me');
}

export async function serverGetAuditLogs(endpoint: string): Promise<ApiResponse<any>> {
  return serverRequest(endpoint);
}

export async function serverCreateMusePayPayment(data: any): Promise<ApiResponse<any>> {
  return serverRequest('/payments/musepay/create', 'POST', data);
}

export async function serverCreateCryptoPayment(data: any): Promise<ApiResponse<any>> {
  return serverRequest(
    '/api/payments/crypto/deposit-address',
    'POST',
    data
  );
}

export async function serverCreateCryptoDepositAddress(data: any): Promise<ApiResponse<any>> {
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
    params.set('limit', (options.maxResults || options.limit)!.toString());
  }
  if (options.pageToken || options.offset) {
    params.set('offset', options.pageToken || options.offset!);
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

export async function serverGetUserStats(): Promise<ApiResponse<any>> {
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
): Promise<ApiResponse<any>> {
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
    `/admin/users${queryString ? `?${queryString}` : ''}`
  );
}

export async function serverGetAdminUser(userId: string): Promise<ApiResponse<AdminUser>> {
  return serverRequest(`/admin/users/${userId}`);
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
  return serverRequest(`/admin/permission-profiles?${params.toString()}`);
}

export async function serverGetAdminPermissionProfile(
  profileId: string
): Promise<ApiResponse<PermissionProfile>> {
  return serverRequest(`/admin/permission-profiles/${profileId}`);
}

export async function serverAssignAdminPermissionProfile(request: {
  profile_id: string;
  user_id: string;
  expires_at?: string;
}): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest('/admin/permission-profiles/assign', 'POST', request);
}

export async function serverGetStockRankingAssignments(
  searchParams?: URLSearchParams
): Promise<
  ApiResponse<{ assignments: StockRankingAssignment[]; total?: number }>
> {
  const params = new URLSearchParams(searchParams);
  return serverRequest(`/admin/stock-ranking/assignments?${params.toString()}`);
}

export async function serverGetStockRankingAssignment(
  assignmentId: string
): Promise<ApiResponse<StockRankingAssignment>> {
  return serverRequest(`/admin/stock-ranking/assignments/${assignmentId}`);
}

export async function serverAssignBulkStockRanking(
  request: StockRankingAssignmentRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest('/admin/stock-ranking/assign-bulk', 'POST', request);
}

export async function serverRevokeStockRankingAssignment(
  assignmentId: string
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest(
    `/admin/stock-ranking/assignments/${assignmentId}/revoke`,
    'POST'
  );
}

export async function serverExtendStockRankingAssignment(
  assignmentId: string,
  request: StockRankingAssignmentExtendRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest(
    `/admin/stock-ranking/assignments/${assignmentId}/extend`,
    'POST',
    request
  );
}

export async function serverUpdateStockRankingAssignment(
  assignmentId: string,
  request: StockRankingAssignmentUpdateRequest
): Promise<ApiResponse<AssignmentResult>> {
  return serverRequest(
    `/admin/stock-ranking/assignments/${assignmentId}`,
    'PUT',
    request
  );
}

export async function serverGetAnalyticsStatistics(): Promise<ApiResponse<AnalyticsStatistics>> {
  return serverRequest('/admin/analytics/statistics');
}

export async function serverGetStockRankingAnalytics(
  searchParams?: URLSearchParams
): Promise<ApiResponse<StockRankingAnalytics>> {
  const params = new URLSearchParams(searchParams);
  return serverRequest(
    `/admin/stock-ranking/analytics?${params.toString()}`
  );
}

export async function serverGetAdminProfile(): Promise<ApiResponse<AdminProfile>> {
  return serverRequest('/admin/auth/profile');
}

export async function serverSoftDeleteUser(
  userId: string,
  request: UserSoftDeleteRequest
): Promise<ApiResponse<{ message: string }>> {
  return serverRequest(`/admin/users/${userId}`, 'DELETE', request);
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
  data?: any
): Promise<ApiResponse<T>> {
  try {
    // Get authentication headers from cookies
    const authHeaders = await CookieManager.buildAuthHeaders();
    
    const requestConfig: RequestInit = {
      method,
      credentials: 'include', // Include cookies for authentication
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders,
      },
    };

    if (data && method !== 'GET') {
      requestConfig.body = JSON.stringify(data);
    }

    const response = await fetch(
      `${getBackendUrl()}${endpoint}`,
      requestConfig
    );

    let responseData;
    const contentType = response.headers.get('content-type');

    if (contentType?.includes('application/json')) {
      responseData = await response.json();
    } else {
      responseData = await response.text();
    }

    if (!response.ok) {
      return {
        error: responseData?.error || responseData || 'Request failed',
        details: responseData?.details || `HTTP ${response.status}`,
      };
    }

    return { data: responseData };
  } catch (error) {
    return {
      error: 'Network error',
      details: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}