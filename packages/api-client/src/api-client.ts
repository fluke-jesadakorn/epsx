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
    // Client-side: use relative URLs (Next.js will proxy)
    return '';
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
      let data;

      if (contentType?.includes('application/json')) {
        data = await response.json();
        console.log(`📄 [ApiClient] JSON response data:`, data);
      } else {
        data = await response.text();
        console.log(`📄 [ApiClient] Text response data:`, data);
      }

      if (!response.ok) {
        const errorResponse = {
          error: data?.error || data || 'Request failed',
          details: data?.details || `HTTP ${response.status}`,
        };
        
        console.error(`❌ [ApiClient] Request failed:`, {
          url,
          status: response.status,
          statusText: response.statusText,
          errorResponse,
          responseData: data,
        });
        
        return errorResponse;
      }

      console.log(`✅ [ApiClient] Request successful:`, {
        url,
        status: response.status,
        dataType: typeof data,
        dataKeys: data && typeof data === 'object' ? Object.keys(data) : 'N/A',
      });

      return { data };
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
    data?: any,
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
    data?: any,
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
    data?: any,
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
    data?: any,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'DELETE',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  // Authentication methods
  async login(credentials: LoginRequest): Promise<ApiResponse<UserProfile>> {
    return this.post<UserProfile>('/api/v1/auth/login', credentials);
  }

  async register(userData: RegisterRequest): Promise<
    ApiResponse<{
      user_id: string;
      email: string;
      verification_sent: boolean;
      message: string;
    }>
  > {
    return this.post('/api/v1/auth/register', userData);
  }

  async enhancedRegister(
    userData: EnhancedRegisterRequest
  ): Promise<ApiResponse<RegistrationResponse>> {
    return this.post<RegistrationResponse>('/api/v1/auth/register', userData);
  }

  async logout(): Promise<ApiResponse<void>> {
    return this.post<void>('/api/v1/auth/logout');
  }

  async getCurrentUser(): Promise<ApiResponse<UserProfile>> {
    return this.get<UserProfile>('/api/v1/auth/profile');
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

  async preloadStocks(symbols: string[]): Promise<ApiResponse<any>> {
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
    const response = await this.get<any>('/api/v1/auth/profile');
    if (response.error) {
      return response;
    }

    // Transform API response to UserPermissionStatus format
    const userData = response.data;
    const transformedData: UserPermissionStatus = {
      userId: userData.user_id,
      permissions: userData.permissions || [],
      profiles: userData.permission_profiles || [],
      role: userData.role || 'user',
      effectivePermissions:
        userData.effective_permissions || userData.permissions || [],
      hasWildcardAccess: userData.permissions?.includes('*') || false,
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
}
