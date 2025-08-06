import { CookieManager } from './cookie-manager';

import type {
  ApiResponse,
  CountResponse,
  CreatePaymentRequest,
  CreatePaymentResponse,
  EnhancedRegisterRequest,
  LoginRequest,
  NotificationListResponse,
  NotificationPreferences,
  PaginatedResponse,
  PasswordChangeRequest,
  PasswordResetRequest,
  PaymentStatusResponse,
  Permission,
  PermissionCheckRequest,
  PermissionCheckResponse,
  PermissionProfile,
  PortfolioItem,
  PriceAlert,
  PriceAlertCreateRequest,
  ProfileUpdateRequest,
  PushSubscriptionRequest,
  RegisterRequest,
  RegistrationResponse,
  RequestConfig,
  Role,
  StockFinancialData,
  StockItem,
  StockRanking,
  UserPermissionStatus,
  UserProfile,
  WatchlistAddRequest,
} from './types';

// Helper function to get backend URL from environment
function getBackendUrl(): string {
  if (typeof window === 'undefined') {
    // Server-side: use environment variables
    return (
      process.env.BACKEND_URL || process.env.API_URL || 'http://localhost:8080'
    );
  } else {
    // Client-side: use backend URL directly (no proxy)
    return 'http://localhost:8080';
  }
}

export class ApiClient {
  private readonly baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || getBackendUrl();
  }

  /**
   * Make a generic HTTP request with automatic auth headers
   */
  private async request<T>(
    endpoint: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    const isServerSide = typeof window === 'undefined';
    
    try {
      console.log(`🚀 [ApiClient] Making ${isServerSide ? 'server-side' : 'client-side'} request:`, {
        url,
        method: config.method || 'GET',
        endpoint,
        baseUrl: this.baseUrl,
        isServerSide,
      });

      // Build headers with auth
      const authHeaders = isServerSide
        ? await CookieManager.buildAuthHeaders() // Server-side
        : CookieManager.client.buildAuthHeaders(); // Client-side

      console.log(`🔑 [ApiClient] Auth headers built:`, authHeaders);

      const headers = {
        'Content-Type': 'application/json',
        ...authHeaders,
        ...config.headers,
      };

      const requestConfig: RequestInit = {
        method: config.method || 'GET',
        credentials: 'include', // Always include cookies
        ...config,
        headers, // Keep headers last to prevent override
      };

      console.log(`📡 [ApiClient] Request config:`, {
        method: requestConfig.method,
        credentials: requestConfig.credentials,
        headers: requestConfig.headers,
        hasBody: !!requestConfig.body,
      });

      if (requestConfig.body) {
        console.log(`📤 [ApiClient] Request body:`, requestConfig.body);
      }

      const response = await fetch(url, requestConfig);

      console.log(`📨 [ApiClient] Response received:`, {
        status: response.status,
        statusText: response.statusText,
        ok: response.ok,
        headers: Object.fromEntries(response.headers.entries()),
        url: response.url,
      });

      // Handle non-JSON responses
      const contentType = response.headers.get('content-type');
      let data: unknown;

      if (contentType?.includes('application/json')) {
        data = await response.json() as unknown;
        console.log(`📄 [ApiClient] JSON response data:`, data);
      } else {
        data = await response.text();
        console.log(`📄 [ApiClient] Text response data:`, data);
      }

      if (!response.ok) {
        // Enhanced error handling for new backend responses
        let errorMessage: string;
        let errorDetails: string;
        
        // Handle different error response formats from backend
        if (typeof data === 'object' && data !== null) {
          const errorData = data as Record<string, unknown>;
          errorMessage = (errorData.error as string) || (errorData.message as string) || 'Request failed';
          errorDetails = (errorData.details as string) || (errorData.reason as string) || `HTTP ${response.status}`;
        } else if (typeof data === 'string') {
          errorMessage = data;
          errorDetails = `HTTP ${response.status}`;
        } else {
          // Fallback error messages based on status codes
          switch (response.status) {
            case 401:
              errorMessage = 'Authentication required or session expired';
              errorDetails = 'Please log in again';
              break;
            case 403:
              errorMessage = 'Access denied';
              errorDetails = 'You do not have permission to perform this action';
              break;
            case 404:
              errorMessage = 'Resource not found';
              errorDetails = 'The requested resource could not be found';
              break;
            case 409:
              errorMessage = 'Conflict';
              errorDetails = 'The request conflicts with existing data';
              break;
            case 429:
              errorMessage = 'Rate limit exceeded';
              errorDetails = 'Please wait before trying again';
              break;
            case 500:
              errorMessage = 'Server error';
              errorDetails = 'An internal server error occurred';
              break;
            default:
              errorMessage = 'Request failed';
              errorDetails = `HTTP ${response.status}`;
          }
        }

        const errorResponse = {
          error: errorMessage,
          details: errorDetails,
        };
        
        console.error(`❌ [ApiClient] Request failed:`, {
          url,
          status: response.status,
          statusText: response.statusText,
          errorResponse,
          responseData: data as unknown,
        });
        
        return errorResponse;
      }

      console.log(`✅ [ApiClient] Request successful:`, {
        url,
        status: response.status,
        dataType: typeof data,
        dataKeys: data && typeof data === 'object' ? Object.keys(data) : 'N/A',
      });

      return { data: data as T };
    } catch (error) {
      const errorDetails = error instanceof Error ? error.message : 'Unknown error';
      
      console.error(`💥 [ApiClient] Network/fetch error:`, {
        url,
        method: config.method || 'GET',
        error: errorDetails,
        stack: error instanceof Error ? error.stack : undefined,
        isServerSide,
      });
      
      return {
        error: 'Network error',
        details: errorDetails,
      };
    }
  }

  // Core HTTP methods
  async get<T>(
    endpoint: string,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'GET', headers });
  }

  async post<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async patch<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'DELETE',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  // Authentication methods - updated for new backend session system
  async login(credentials: LoginRequest): Promise<ApiResponse<UserProfile>> {
    // Backend expects LoginRequest format with credentials
    const loginPayload = {
      credentials: {
        email: credentials.email,
        password: credentials.password
      }
    };
    
    const response = await this.post<{
      user: UserProfile;
      session: {
        session_id: string;
        expires_at: string;
      };
    }>('/api/v1/auth/login', loginPayload);

    if (response.error) {
      return response;
    }

    // Store session_id for future requests (client-side only)
    if (typeof window !== 'undefined' && response.data?.session?.session_id) {
      localStorage.setItem('session_id', response.data.session.session_id);
      localStorage.setItem('session_expires', response.data.session.expires_at);
    }

    // Return just the user profile for backward compatibility
    return { data: response.data?.user };
  }

  async register(userData: RegisterRequest): Promise<
    ApiResponse<{
      user_id: string;
      email: string;
      verification_sent: boolean;
      message: string;
    }>
  > {
    // Transform to backend's expected format
    const registerPayload = {
      email: userData.email,
      password: userData.password,
      display_name: userData.name || undefined,
      package_tier: 'basic'
    };

    return this.post('/api/v1/auth/register', registerPayload);
  }

  async enhancedRegister(
    userData: EnhancedRegisterRequest
  ): Promise<ApiResponse<RegistrationResponse>> {
    const registerPayload = {
      email: userData.email,
      password: userData.password,
      display_name: undefined, // EnhancedRegisterRequest doesn't have name field
      package_tier: userData.package_tier || 'basic'
    };

    return this.post<RegistrationResponse>('/api/v1/auth/register', registerPayload);
  }

  async logout(): Promise<ApiResponse<void>> {
    const response = await this.post<void>('/api/v1/auth/logout');
    
    // Clear stored session data (client-side only)
    if (typeof window !== 'undefined') {
      localStorage.removeItem('session_id');
      localStorage.removeItem('session_expires');
    }
    
    return response;
  }

  async getCurrentUser(): Promise<ApiResponse<UserProfile>> {
    const response = await this.get<{
      user_id: string;
      email: string;
      roles: string[];
      permissions: string[];
      subscription_tier: string;
      package_tier: string;
      created_at: string;
      updated_at: string;
      display_name?: string;
      photo_url?: string;
      email_verified: boolean;
      is_active: boolean;
    }>('/api/v1/auth/profile');

    if (response.error) {
      return response;
    }

    // Transform backend response to frontend UserProfile format
    const backendData = response.data;
    if (!backendData) {
      return { error: 'Invalid response data' };
    }
    
    const userProfile: UserProfile = {
      id: backendData.user_id,
      email: backendData.email,
      name: backendData.display_name || backendData.email,
      role: backendData.roles?.[0] || 'user',
      subscriptionTier: backendData.subscription_tier || 'free',
      packageTier: backendData.package_tier || backendData.subscription_tier || 'free',
      isActive: backendData.is_active ?? true,
      emailVerified: backendData.email_verified ?? false,
      permissions: backendData.permissions || [],
      createdAt: backendData.created_at,
      updatedAt: backendData.updated_at,
      photoUrl: backendData.photo_url
    };

    return { data: userProfile };
  }

  async refreshSession(): Promise<ApiResponse<{ expires_at: string }>> {
    return this.post('/api/v1/auth/refresh');
  }

  async resetPassword(
    request: PasswordResetRequest
  ): Promise<ApiResponse<{ message: string; reset_sent: boolean }>> {
    return this.post('/api/v1/auth/password-reset', request);
  }

  async updateProfile(
    request: ProfileUpdateRequest
  ): Promise<ApiResponse<UserProfile>> {
    return this.patch<UserProfile>('/api/v1/auth/profile', request);
  }

  async changePassword(
    request: PasswordChangeRequest
  ): Promise<ApiResponse<{ message: string }>> {
    return this.patch('/api/v1/auth/password', request);
  }

  // Payment methods
  async createPayment(
    data: CreatePaymentRequest
  ): Promise<ApiResponse<CreatePaymentResponse>> {
    return this.post<CreatePaymentResponse>('/api/v1/payment/create', data);
  }

  async getPaymentStatus(
    paymentId?: string
  ): Promise<ApiResponse<PaymentStatusResponse>> {
    const endpoint = paymentId
      ? `/api/v1/payment/${paymentId}/status`
      : '/api/v1/payment/status';
    return this.get<PaymentStatusResponse>(endpoint);
  }

  async verifyPayment(
    transactionId: string
  ): Promise<ApiResponse<{ verified: boolean }>> {
    return this.post<{ verified: boolean }>('/api/v1/payment/verify', {
      transactionId,
    });
  }

  async cancelPayment(paymentId: string): Promise<ApiResponse<void>> {
    return this.post<void>('/api/v1/payment/cancel', { paymentId });
  }

  // Stock/Market data methods
  async getStocks(
    params: {
      page?: number;
      limit?: number;
      country?: string;
      quarters?: number;
      skip?: number;
      paginated?: boolean;
    } = {}
  ): Promise<
    ApiResponse<StockFinancialData[] | PaginatedResponse<StockFinancialData>>
  > {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        searchParams.set(key, value.toString());
      }
    });

    return this.get<
      StockFinancialData[] | PaginatedResponse<StockFinancialData>
    >(`/api/v1/market-data/stocks?${searchParams.toString()}`);
  }

  async getBatchStocks(symbols: string[]): Promise<
    ApiResponse<{
      data: Record<string, StockFinancialData>;
      cached: string[];
      fetched: string[];
      errors: string[];
      success: boolean;
    }>
  > {
    return this.get(
      `/api/v1/market-data/stocks/batch?symbols=${symbols.join(',')}`
    );
  }

  async preloadStocks(symbols: string[]): Promise<ApiResponse<unknown>> {
    return this.post('/api/v1/market-data/stocks/batch', {
      symbols,
      action: 'preload',
    });
  }

  async checkStockCacheStatus(
    symbols: string[]
  ): Promise<ApiResponse<{ symbols: string[] }>> {
    return this.post('/api/v1/market-data/stocks/batch', {
      symbols,
      action: 'cache_status',
    });
  }

  async getPaginatedStocks(
    params: {
      page?: number;
      limit?: number;
      country?: string;
      quarters?: number;
    } = {}
  ): Promise<ApiResponse<PaginatedResponse<StockFinancialData>>> {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        searchParams.set(key, value.toString());
      }
    });

    return this.get<PaginatedResponse<StockFinancialData>>(
      `/api/v1/market-data/stocks/paginated?${searchParams.toString()}`
    );
  }

  async getStockCount(
    params: {
      country?: string;
      quarters?: number;
    } = {}
  ): Promise<ApiResponse<CountResponse>> {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        searchParams.set(key, value.toString());
      }
    });

    return this.get<CountResponse>(
      `/api/v1/market-data/stocks/count?${searchParams.toString()}`
    );
  }

  // Permission Profile methods
  async listPermissionProfiles(
    options: {
      category?: string;
      package_tier?: string;
      active_only?: boolean;
      limit?: number;
      offset?: number;
    } = {}
  ): Promise<
    ApiResponse<{
      permission_profiles: PermissionProfile[];
      total: number;
      limit: number;
      offset: number;
    }>
  > {
    const params = new URLSearchParams();
    Object.entries(options).forEach(([key, value]) => {
      if (value !== undefined) {
        params.set(key, value.toString());
      }
    });

    return this.get(`/api/admin/permission-profiles?${params.toString()}`);
  }

  async getPermissionProfile(
    permissionProfileId: string
  ): Promise<ApiResponse<PermissionProfile>> {
    return this.get<PermissionProfile>(
      `/api/admin/permission-profiles/${permissionProfileId}`
    );
  }

  // Permission Management methods
  async getPermissions(): Promise<ApiResponse<Permission[]>> {
    return this.get<Permission[]>('/api/v1/permissions');
  }

  async checkUserPermission(
    request: PermissionCheckRequest
  ): Promise<ApiResponse<PermissionCheckResponse>> {
    return this.post<PermissionCheckResponse>(
      '/api/v1/auth/check-permission',
      request
    );
  }

  async getUserPermissionStatus(): Promise<ApiResponse<UserPermissionStatus>> {
    const response = await this.get<unknown>('/api/v1/auth/profile');
    if (response.error) {
      return response as ApiResponse<UserPermissionStatus>;
    }

    // Transform API response to UserPermissionStatus format
    const userData = response.data as Record<string, unknown>;
    const transformedData: UserPermissionStatus = {
      userId: userData.user_id as string,
      permissions: (userData.permissions as string[]) || [],
      profiles: (userData.permission_profiles as string[]) || [],
      role: (userData.role as string) || 'user',
      effectivePermissions:
        (userData.effective_permissions as string[]) || (userData.permissions as string[]) || [],
      hasWildcardAccess: ((userData.permissions as string[])?.includes('*')) || false,
    };

    return { data: transformedData };
  }

  async assignUserPermissionProfile(
    userId: string,
    profileId: string,
    expiresAt?: string
  ): Promise<ApiResponse<void>> {
    return this.post<void>(`/admin/users/${userId}/permission-profiles`, {
      permission_profile_id: profileId,
      expires_at: expiresAt,
      assigned_by: 'admin',
      reason: 'Manual assignment via admin interface',
    });
  }

  async revokeUserPermissionProfile(
    userId: string,
    profileId: string
  ): Promise<ApiResponse<void>> {
    return this.delete<void>(
      `/admin/users/${userId}/permission-profiles/${profileId}`
    );
  }

  async getRoles(): Promise<ApiResponse<Role[]>> {
    return this.get<Role[]>('/roles');
  }

  async updateRolePermissions(
    roleId: string,
    permissionIds: string[]
  ): Promise<ApiResponse<void>> {
    return this.put<void>(`/roles/${roleId}/permissions`, {
      permissions: permissionIds,
    });
  }

  async updateUserPermissions(
    userId: string,
    roleIds: string[],
    directPermissions: string[]
  ): Promise<ApiResponse<void>> {
    return this.put<void>(`/users/${userId}/permissions`, {
      roles: roleIds,
      directPermissions,
    });
  }

  // Trading methods
  async getWatchlist(): Promise<ApiResponse<StockItem[]>> {
    return this.get<StockItem[]>('/api/trading/watchlist');
  }

  async addToWatchlist(
    request: WatchlistAddRequest
  ): Promise<ApiResponse<StockItem>> {
    return this.post<StockItem>('/api/trading/watchlist', request);
  }

  async removeFromWatchlist(symbol: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/trading/watchlist/${symbol}`);
  }

  async getPortfolio(): Promise<ApiResponse<PortfolioItem[]>> {
    return this.get<PortfolioItem[]>('/api/trading/portfolio');
  }

  async getRankings(): Promise<ApiResponse<StockRanking[]>> {
    return this.get<StockRanking[]>('/api/trading/rankings');
  }

  async getPriceAlerts(): Promise<ApiResponse<PriceAlert[]>> {
    return this.get<PriceAlert[]>('/api/trading/alerts');
  }

  async addPriceAlert(
    request: PriceAlertCreateRequest
  ): Promise<ApiResponse<PriceAlert>> {
    return this.post<PriceAlert>('/api/trading/alerts', request);
  }

  async removePriceAlert(alertId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/trading/alerts/${alertId}`);
  }

  // Notification methods
  async getNotifications(): Promise<ApiResponse<NotificationListResponse>> {
    return this.get<NotificationListResponse>('/api/notifications');
  }

  async markNotificationRead(
    notificationId: string
  ): Promise<ApiResponse<void>> {
    return this.post<void>(`/api/notifications/${notificationId}/read`);
  }

  async markAllNotificationsRead(): Promise<ApiResponse<void>> {
    return this.post<void>('/api/notifications/read-all');
  }

  async deleteNotification(notificationId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/notifications/${notificationId}`);
  }

  async getNotificationPreferences(): Promise<
    ApiResponse<NotificationPreferences>
  > {
    return this.get<NotificationPreferences>('/api/notifications/preferences');
  }

  async updateNotificationPreferences(
    preferences: Partial<NotificationPreferences>
  ): Promise<ApiResponse<NotificationPreferences>> {
    return this.put<NotificationPreferences>(
      '/api/notifications/preferences',
      preferences
    );
  }

  async subscribeToPushNotifications(
    subscription: PushSubscriptionRequest
  ): Promise<ApiResponse<void>> {
    return this.post<void>(
      '/api/notifications/push-subscription',
      subscription
    );
  }

  async unsubscribeFromPushNotifications(): Promise<ApiResponse<void>> {
    return this.delete<void>('/api/notifications/push-subscription');
  }

  // IAM methods
  async getIamUsers(_filters?: {
    packageTier?: string;
    subscriptionStatus?: string;
    hasCustomPermissions?: boolean;
  }): Promise<ApiResponse<unknown[]>> {
    // Note: This endpoint doesn't exist in backend yet, returning empty array
    console.warn('getIamUsers: Backend endpoint not implemented yet');
    return { data: [] };
  }

  async getIamUser(_uid: string): Promise<ApiResponse<unknown>> {
    // Note: This endpoint doesn't exist in backend yet
    console.warn('getIamUser: Backend endpoint not implemented yet');
    return { data: null };
  }

  async getIamRoles(): Promise<ApiResponse<unknown[]>> {
    const response = await this.get<{ success: boolean; data: unknown[] }>('/api/v1/iam/roles');
    if (response.error) {
      // Handle database/backend errors gracefully for now
      console.warn('getIamRoles: Backend error, returning empty data:', response.error);
      return { data: [] };
    }
    return { data: response.data?.data || [] };
  }

  async getIamPolicies(): Promise<ApiResponse<unknown[]>> {
    const response = await this.get<{ success: boolean; data: unknown[] }>('/api/v1/iam/policies');
    if (response.error) {
      // Handle database/backend errors gracefully for now
      console.warn('getIamPolicies: Backend error, returning empty data:', response.error);
      return { data: [] };
    }
    return { data: response.data?.data || [] };
  }

  async updateUserPackageTier(_uid: string, _data: {
    packageTier: string;
    updatedBy: string;
  }): Promise<ApiResponse<void>> {
    console.warn('updateUserPackageTier: Backend endpoint not implemented yet');
    return { data: undefined };
  }

  async applyPackagePermissions(_data: {
    userId: string;
    packageTier: string;
  }): Promise<ApiResponse<void>> {
    console.warn('applyPackagePermissions: Backend endpoint not implemented yet');
    return { data: undefined };
  }

  async grantCustomPermission(_data: {
    userId: string;
    featureId: string;
    permission: string;
    grantedBy: string;
    expiresAt?: Date;
    reason?: string;
  }): Promise<ApiResponse<unknown>> {
    console.warn('grantCustomPermission: Backend endpoint not implemented yet');
    return { data: null };
  }

  async revokeCustomPermission(_data: {
    permissionId: string;
    revokedBy: string;
    reason?: string;
  }): Promise<ApiResponse<void>> {
    console.warn('revokeCustomPermission: Backend endpoint not implemented yet');
    return { data: undefined };
  }

  async evaluatePermission(data: {
    userId: string;
    action: string;
    resource: string;
  }): Promise<ApiResponse<{ allowed: boolean }>> {
    const response = await this.post<{ success: boolean; data: { allowed: boolean; reasons: string[]; matching_policies: string[]; package_tier_access: boolean; explicit_permissions: string[] } }>('/api/v1/iam/evaluate', {
      user_id: data.userId,
      action: data.action,
      resource: data.resource
    });
    if (response.error) {
      // Handle database/backend errors gracefully for now
      console.warn('evaluatePermission: Backend error, returning denied:', response.error);
      return { data: { allowed: false } };
    }
    return { data: { allowed: response.data?.data?.allowed || false } };
  }

  async getUserEffectivePermissions(_uid: string): Promise<ApiResponse<unknown[]>> {
    console.warn('getUserEffectivePermissions: Backend endpoint not implemented yet');
    return { data: [] };
  }

  async bulkApplyPermissionProfile(_data: {
    userIds: string[];
    profileId: string;
    appliedBy: string;
  }): Promise<ApiResponse<void>> {
    console.warn('bulkApplyPermissionProfile: Backend endpoint not implemented yet');
    return { data: undefined };
  }

  async previewPackageUpgrade(_data: {
    userId: string;
    targetTier: string;
  }): Promise<ApiResponse<unknown>> {
    console.warn('previewPackageUpgrade: Backend endpoint not implemented yet');
    return { data: { currentPermissions: [], newPermissions: [], addedPermissions: [], removedPermissions: [] } };
  }

  async getUserAuditLogs(_uid: string, _limit: number): Promise<ApiResponse<unknown[]>> {
    console.warn('getUserAuditLogs: Backend endpoint not implemented yet');
    return { data: [] };
  }

  async getAllAuditLogs(_limit: number): Promise<ApiResponse<unknown[]>> {
    console.warn('getAllAuditLogs: Backend endpoint not implemented yet');
    return { data: [] };
  }

  async getPermissionProfiles(): Promise<ApiResponse<unknown>> {
    console.warn('getPermissionProfiles: Backend endpoint not implemented yet');
    return { data: {} };
  }

  async createIamRole(data: {
    name: string;
    description?: string;
    packageTier: string;
    policies: string[];
    inlinePermissions: unknown[];
    assignable: boolean;
  }): Promise<ApiResponse<unknown>> {
    const response = await this.post<{ success: boolean; data: unknown }>('/api/v1/iam/roles', {
      name: data.name,
      description: data.description,
      package_tier: data.packageTier,
      policies: data.policies,
      inline_permissions: data.inlinePermissions,
      assignable: data.assignable
    });
    if (response.error) {
      return response;
    }
    return { data: response.data?.data };
  }

  async cleanupExpiredPermissions(): Promise<ApiResponse<void>> {
    console.warn('cleanupExpiredPermissions: Backend endpoint not implemented yet');
    return { data: undefined };
  }
}
