/**
 * UNIFIED NOTIFICATIONS API CLIENT
 *
 * Consolidates all notification-related API calls across EPSX applications.
 * Eliminates proxy routes by providing direct backend communication.
 *
 * Features:
 * - User notification management (get, mark read, delete)
 * - Admin notification sending and management
 * - Real-time SSE notifications
 * - Push notification subscriptions
 * - Notification preferences management
 * - Type-safe responses with proper error handling
 */

import { UnifiedApiClient, ApiResponse, PaginatedResponse } from '../utils/api-client';
import { COOKIES } from '../auth/cookies';
import { API_ROUTES } from '../config/route-constants';
import {
  NotificationSchema,
  NotificationsResponseSchema,
  SSENotificationSchema,
  NotificationFiltersSchema,
  SendNotificationRequestSchema,
  validateNotification,
  validateSSENotification,
  validateNotificationFilters,
  validateSendNotificationRequest,
} from '../components/notifications/schemas';

// ============================================================================
// ERROR HANDLING
// ============================================================================

export class NotificationAPIError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly status?: number,
    public readonly details?: any
  ) {
    super(message);
    this.name = 'NotificationAPIError';
    Object.setPrototypeOf(this, NotificationAPIError.prototype);
  }
}

export class NotificationNotFoundError extends NotificationAPIError {
  constructor(notificationId: string, details?: any) {
    super(
      `Notification not found: ${notificationId}`,
      'NOTIFICATION_NOT_FOUND',
      404,
      details
    );
    this.name = 'NotificationNotFoundError';
  }
}

export class NotificationPermissionError extends NotificationAPIError {
  constructor(operation: string, details?: any) {
    super(
      `Permission denied for operation: ${operation}`,
      'NOTIFICATION_PERMISSION_DENIED',
      403,
      details
    );
    this.name = 'NotificationPermissionError';
  }
}

export class NotificationValidationError extends NotificationAPIError {
  constructor(message: string, details?: any) {
    super(message, 'NOTIFICATION_VALIDATION_ERROR', 400, details);
    this.name = 'NotificationValidationError';
  }
}

function handleNotificationError(error: any, operation: string, details?: any): never {
  const status = error?.status || error?.response?.status;
  const errorMessage = error?.error || error?.message || 'Unknown error occurred';

  if (status === 404) {
    throw new NotificationNotFoundError(
      details?.notificationId || 'unknown',
      { operation, originalError: errorMessage }
    );
  }

  if (status === 403) {
    throw new NotificationPermissionError(operation, {
      originalError: errorMessage,
      ...details
    });
  }

  if (status === 400) {
    throw new NotificationValidationError(
      `Failed to ${operation}: ${errorMessage}`,
      { originalError: errorMessage, ...details }
    );
  }

  throw new NotificationAPIError(
    `Failed to ${operation}: ${errorMessage}`,
    'NOTIFICATION_API_ERROR',
    status,
    { operation, originalError: errorMessage, ...details }
  );
}

// ============================================================================
// NOTIFICATION TYPES
// ============================================================================

export interface Notification {
  id: string;
  wallet_address: string;
  notification_type: NotificationType;
  title: string;
  message: string;
  data?: Record<string, any>;
  priority: NotificationPriority;
  timestamp: string;
  expires_at?: string;
  read_at?: string;
  clicked_at?: string;
  delivered_at?: string;
  action_url?: string;
  image_url?: string;
}

export type NotificationType =
  | 'system'
  | 'security'
  | 'permission'
  | 'wallet_management'
  | 'wallet'
  | 'payment'
  | 'general';

export type NotificationPriority = 'low' | 'normal' | 'high' | 'critical';

export interface NotificationFilters {
  page?: number;
  limit?: number;
  type?: NotificationType;
  priority?: NotificationPriority;
  status?: 'read' | 'unread' | 'all';
  start_date?: string;
  end_date?: string;
  wallet_address?: string;
}

export interface NotificationsResponse {
  success: boolean;
  data: {
    notifications: Notification[];
    total_count: number;
    unread_count: number;
    page: number;
    limit: number;
    total_pages: number;
  };
  api_version?: string;
  access_level?: string;
}

export interface NotificationPreferences {
  email_enabled: boolean;
  push_enabled: boolean;
  sms_enabled?: boolean;
  types: {
    system: boolean;
    security: boolean;
    permission: boolean;
    user_management: boolean;
    wallet: boolean;
    payment: boolean;
    general: boolean;
  };
  priority_filter: NotificationPriority;
  quiet_hours?: {
    enabled: boolean;
    start_time: string;
    end_time: string;
    timezone: string;
  };
}

export interface NotificationPreferencesResponse {
  success: boolean;
  data: NotificationPreferences;
  api_version?: string;
  access_level?: string;
}

export interface SendNotificationRequest {
  recipient_wallet_address?: string;
  recipient_group?: string;
  broadcast?: boolean;
  notification_type: NotificationType;
  priority: NotificationPriority;
  title: string;
  message: string;
  data?: Record<string, any>;
  action_url?: string;
  image_url?: string;
  expires_at?: string;
  schedule_at?: string;
}

export interface SendNotificationResponse {
  success: boolean;
  data: {
    notification_id: string;
    recipients_count: number;
    scheduled: boolean;
    delivery_status: 'sent' | 'scheduled' | 'failed';
  };
  message: string;
  api_version?: string;
}

export interface NotificationStats {
  total_notifications: number;
  sent_today: number;
  sent_this_week: number;
  sent_this_month: number;
  delivery_rate: number;
  read_rate: number;
  click_rate: number;
  by_type: Record<NotificationType, number>;
  by_priority: Record<NotificationPriority, number>;
  recent_activity: Array<{
    timestamp: string;
    action: string;
    count: number;
  }>;
}

export interface NotificationStatsResponse {
  success: boolean;
  data: NotificationStats;
  api_version?: string;
  access_level?: string;
}

export interface PushSubscription {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
  user_agent?: string;
  device_type?: 'desktop' | 'mobile' | 'tablet';
}

export interface PushSubscriptionResponse {
  success: boolean;
  data: {
    subscription_id: string;
    active: boolean;
    created_at: string;
  };
  message: string;
  api_version?: string;
}

// ============================================================================
// SSE TYPES
// ============================================================================

export interface SSENotification {
  id: string;
  wallet_address: string;
  notification_type: NotificationType;
  title: string;
  message: string;
  data?: Record<string, any>;
  priority: NotificationPriority;
  timestamp: string;
  expires_at?: string;
}

export interface SSEConnectionOptions {
  wallet_address?: string;
  types?: NotificationType[];
  priority?: NotificationPriority;
  auto_reconnect?: boolean;
  reconnect_interval?: number;
}

// ============================================================================
// NOTIFICATIONS API CLIENT CLASS
// ============================================================================

export class NotificationsAPIClient {
  private sseConnection?: EventSource;
  private sseListeners: Map<string, (notification: SSENotification) => void> = new Map();
  // Track event listeners for proper cleanup
  private sseEventHandlers: {
    onmessage?: (event: MessageEvent) => void;
    onerror?: (event: Event) => void;
    onopen?: (event: Event) => void;
    notification?: (event: MessageEvent) => void;
    ping?: (event: MessageEvent) => void;
  } = {};

  constructor(private client: UnifiedApiClient) {}

  // ============================================================================
  // USER NOTIFICATIONS
  // ============================================================================

  /**
   * Get user notifications
   * Route: GET /api/notifications
   */
  async getNotifications(filters: NotificationFilters = {}): Promise<NotificationsResponse> {
    try {
      // Validate input filters
      const filterValidation = validateNotificationFilters(filters);
      if (!filterValidation.success) {
        throw new NotificationValidationError(
          'Invalid notification filters',
          filterValidation.errors.format()
        );
      }

      const response = await this.client.get<NotificationsResponse>(
        '/api/notifications',
        filterValidation.data,
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'auth',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'fetch notifications', { filters });
      }

      // Validate response schema
      const validatedResponse = NotificationsResponseSchema.safeParse(response.data);
      if (!validatedResponse.success) {
        console.warn('API response validation failed:', validatedResponse.error);
        // Continue with unvalidated data but log the issue
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'fetch notifications', { filters });
    }
  }

  /**
   * Get unread notification count
   * Route: GET /api/notifications/unread-count
   */
  async getUnreadCount(): Promise<{ unread_count: number }> {
    const response = await this.client.get<{ unread_count: number }>(
      '/api/notifications/unread-count',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch unread count: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Mark notification as read
   * Route: PUT /api/notifications/{id}/read
   */
  async markAsRead(notificationId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.client.put<{ success: boolean; message: string }>(
        `/api/notifications/${notificationId}/read`,
        {},
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'auth',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'mark notification as read', { notificationId });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'mark notification as read', { notificationId });
    }
  }

  /**
   * Mark all notifications as read
   * Route: PUT /api/v1/auth/notifications/mark-all-read
   */
  async markAllAsRead(): Promise<{ success: boolean; updated_count: number }> {
    try {
      const response = await this.client.put<{ success: boolean; updated_count: number }>(
        '/api/notifications/mark-all-read',
        {},
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'auth',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'mark all notifications as read');
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'mark all notifications as read');
    }
  }

  /**
   * Acknowledge notification (mark as delivered/received)
   * Route: PUT /api/notifications/{id}/acknowledge
   */
  async acknowledgeNotification(notificationId: string): Promise<{ success: boolean; message: string }> {
    const response = await this.client.put<{ success: boolean; message: string }>(
      `/api/notifications/${notificationId}/acknowledge`,
      {},
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      // Don't throw error for acknowledgement failures - just log
      console.warn(`Failed to acknowledge notification ${notificationId}: ${response.error}`);
      return { success: false, message: response.error || 'Failed to acknowledge notification' };
    }

    return response.data;
  }

  /**
   * Delete notification
   * Route: DELETE /api/v1/auth/notifications/{id}
   */
  async deleteNotification(notificationId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.client.delete<{ success: boolean; message: string }>(
        `/api/notifications/${notificationId}`,
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'auth',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'delete notification', { notificationId });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'delete notification', { notificationId });
    }
  }

  /**
   * Clear all notifications
   * Route: DELETE /api/v1/auth/notifications/clear-all
   */
  async clearAllNotifications(): Promise<{ success: boolean; deleted_count: number }> {
    try {
      const response = await this.client.delete<{ success: boolean; deleted_count: number }>(
        '/api/notifications/clear-all',
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'auth',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'clear all notifications');
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'clear all notifications');
    }
  }

  // ============================================================================
  // NOTIFICATION PREFERENCES
  // ============================================================================

  /**
   * Get notification preferences
   * Route: GET /api/v1/auth/notifications/preferences
   */
  async getPreferences(): Promise<NotificationPreferencesResponse> {
    const response = await this.client.get<NotificationPreferencesResponse>(
      '/api/notifications/preferences',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch notification preferences: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Update notification preferences
   * Route: PUT /api/v1/auth/notifications/preferences
   */
  async updatePreferences(preferences: Partial<NotificationPreferences>): Promise<{
    success: boolean;
    message: string;
  }> {
    const response = await this.client.put<{ success: boolean; message: string }>(
      '/api/notifications/preferences',
      preferences,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to update notification preferences: ${response.error}`);
    }

    return response.data;
  }

  // ============================================================================
  // PUSH NOTIFICATIONS
  // ============================================================================

  /**
   * Subscribe to push notifications
   * Route: POST /api/v1/auth/notifications/push/subscribe
   */
  async subscribeToPush(subscription: PushSubscription): Promise<PushSubscriptionResponse> {
    const response = await this.client.post<PushSubscriptionResponse>(
      '/api/notifications/push/subscribe',
      subscription,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to subscribe to push notifications: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Unsubscribe from push notifications
   * Route: DELETE /api/v1/auth/notifications/push/unsubscribe
   */
  async unsubscribeFromPush(): Promise<{ success: boolean; message: string }> {
    const response = await this.client.delete<{ success: boolean; message: string }>(
      '/api/notifications/push/unsubscribe',
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to unsubscribe from push notifications: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get push subscription status
   * Route: GET /api/v1/auth/notifications/push/status
   */
  async getPushStatus(): Promise<{
    subscribed: boolean;
    subscription_id?: string;
    created_at?: string;
  }> {
    const response = await this.client.get(
      '/api/notifications/push/status',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get push notification status: ${response.error}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // ADMIN NOTIFICATIONS
  // ============================================================================

  /**
   * Send notification (admin only)
   * Route: POST /api/v1/admin/notifications/send
   */
  async sendNotification(request: SendNotificationRequest): Promise<SendNotificationResponse> {
    try {
      // Validate request payload
      const requestValidation = validateSendNotificationRequest(request);
      if (!requestValidation.success) {
        throw new NotificationValidationError(
          'Invalid send notification request',
          requestValidation.errors.format()
        );
      }

      const response = await this.client.post<SendNotificationResponse>(
        API_ROUTES.ADMIN.NOTIFICATIONS + '/send',
        requestValidation.data,
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'admin',
            'X-Admin-Context': 'true',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'send notification', { request });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'send notification', { request });
    }
  }

  /**
   * Get notification statistics (admin only)
   * Route: GET /api/v1/admin/notifications/stats
   */
  async getNotificationStats(): Promise<NotificationStatsResponse> {
    try {
      const response = await this.client.get<NotificationStatsResponse>(
        API_ROUTES.ADMIN.NOTIFICATIONS + '/stats',
        undefined,
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'admin',
            'X-Admin-Context': 'true',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'fetch notification stats');
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'fetch notification stats');
    }
  }

  /**
   * Get all notifications (admin only)
   * Route: GET /api/v1/admin/notifications
   */
  async getAllNotifications(filters: NotificationFilters = {}): Promise<NotificationsResponse> {
    try {
      const response = await this.client.get<NotificationsResponse>(
        API_ROUTES.ADMIN.NOTIFICATIONS,
        filters,
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'admin',
            'X-Admin-Context': 'true',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'fetch all notifications (admin)', { filters });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'fetch all notifications (admin)', { filters });
    }
  }

  /**
   * Delete notification (admin only - hard delete)
   * Route: DELETE /api/v1/admin/notifications/{id}
   */
  async deleteAdminNotification(notificationId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.client.delete<{ success: boolean; message: string }>(
        `${API_ROUTES.ADMIN.NOTIFICATIONS}/${notificationId}`,
        {
          headers: {
            'X-API-Version': 'v1',
            'X-Access-Level': 'admin',
            'X-Admin-Context': 'true',
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'delete notification (admin)', { notificationId });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'delete notification (admin)', { notificationId });
    }
  }

  /**
   * Validate SSE connection before establishing EventSource
   * @param baseURL - Backend base URL
   * @param platform - Platform identifier (admin/frontend)
   * @param token - Authentication token
   */
  private async validateSSEConnection(
    baseURL: string,
    platform: string | undefined,
    token: string | null
  ): Promise<void> {
    console.log('🔍 SSE Connection Validation Started');

    // Check EventSource support
    if (typeof EventSource === 'undefined') {
      const error = 'EventSource not supported in this browser';
      console.error('❌ SSE Validation Failed:', error);
      throw new Error(error);
    }

    // Check online status
    if (!navigator.onLine) {
      console.warn('⚠️ Browser is offline, SSE connection may fail');
    }

    // Check cookie support
    if (!navigator.cookieEnabled) {
      console.warn('⚠️ Cookies are disabled, authentication may fail');
    }

    // Validate authentication context
    if (!token && platform === 'admin') {
      console.warn('⚠️ Admin platform requires authentication, but no token found');
    }

    // Test SSE endpoint availability (without streaming)
    try {
      console.log('📡 Testing SSE endpoint availability...');
      const sseTestUrl = `${baseURL}${API_ROUTES.NOTIFICATIONS.STREAM}`;
      const testResponse = await fetch(sseTestUrl, {
        method: 'HEAD', // Just check headers, don't stream
        headers: {
          'Accept': 'text/event-stream',
          'Cache-Control': 'no-cache'
        }
      });

      if (testResponse.status === 401) {
        console.warn('⚠️ SSE endpoint requires authentication');
      } else if (testResponse.status === 405 || testResponse.status === 400) {
        // HEAD method not supported, try GET with timeout
        console.log('🔄 Testing SSE endpoint with GET request...');
        const abortController = new AbortController();
        const timeoutId = setTimeout(() => abortController.abort(), 3000);

        try {
          const getResponse = await fetch(sseTestUrl, {
            method: 'GET',
            headers: {
              'Accept': 'text/event-stream',
              'Cache-Control': 'no-cache'
            },
            signal: abortController.signal
          });

          clearTimeout(timeoutId);
          console.log('✅ SSE endpoint is reachable');
        } catch (fetchError) {
          clearTimeout(timeoutId);
          if (fetchError.name === 'AbortError') {
            console.log('✅ SSE endpoint responded (timeout expected for streaming)');
          } else {
            console.warn('⚠️ SSE endpoint test failed:', fetchError);
          }
        }
      } else {
        console.log('✅ SSE endpoint is reachable');
      }
    } catch (error) {
      console.warn('⚠️ SSE endpoint availability test failed:', error);
      // Don't throw here - allow connection attempt to proceed
    }

    // Log connection details for debugging
    console.log('📋 SSE Connection Summary:', {
      platform,
      hasToken: !!token,
      baseURL,
      userAgent: navigator.userAgent.substring(0, 50),
      timestamp: new Date().toISOString()
    });

    console.log('✅ SSE Connection Validation Completed');
  }

  // ============================================================================
  // SERVER-SENT EVENTS (SSE)
  // ============================================================================

  /**
   * Connect to SSE for real-time notifications
   * Route: GET /api/v1/auth/notifications/stream
   */
  connectToSSE(
    options: SSEConnectionOptions = {},
    onNotification: (notification: SSENotification) => void,
    onError?: (error: Event) => void,
    onOpen?: (event: Event) => void
  ): () => void {
    // Close existing connection if any with proper cleanup
    if (this.sseConnection) {
      console.log('🔄 Closing existing SSE connection before creating new one');
      this.cleanupSSEListeners();
      this.sseConnection.close();
      this.sseConnection = undefined;
    }

    // Check EventSource support
    if (typeof EventSource === 'undefined') {
      const error = 'EventSource not supported in this browser';
      console.error(error);
      throw new Error(error);
    }

    // Build SSE URL with parameters
    const params = new URLSearchParams();
    if (options.wallet_address) params.append('wallet_address', options.wallet_address);
    if (options.types) params.append('types', options.types.join(','));
    if (options.priority) params.append('priority', options.priority);

    // Get token from cookies (EventSource doesn't support Authorization header)
    const token = this.getTokenFromCookies();
    const platform = this.client['platform'] as 'admin' | 'frontend' | undefined;

    if (token) {
      params.append('token', token);
      console.log(`🔑 SSE [${platform}]: User wallet found and added to connection URL: ${token}`);
    } else {
      console.warn(`⚠️ SSE [${platform}]: No authenticated user found in cookies, only broadcast notifications will be received`);
      console.warn(`Available cookies: ${document.cookie ? document.cookie.split(';').map(c => c.trim().split('=')[0]).join(', ') : 'none'}`);
    }

    // Only append query string if there are parameters
    const queryString = params.toString();
    const baseURL = this.client['baseURL'];
    const sseUrl = `${baseURL}${API_ROUTES.NOTIFICATIONS.STREAM}${queryString ? '?' + queryString : ''}`;

    // Comprehensive validation
    if (!baseURL || typeof baseURL !== 'string') {
      const error = `Invalid baseURL: "${baseURL}" (type: ${typeof baseURL})`;
      console.error('❌ SSE Connection Error:', error);
      throw new Error(error);
    }

    // Validate URL format
    try {
      const urlObj = new URL(sseUrl);
      console.log('✅ SSE URL validated:', {
        protocol: urlObj.protocol,
        host: urlObj.host,
        pathname: urlObj.pathname,
        search: urlObj.search,
        fullURL: sseUrl
      });
    } catch (e) {
      const error = `Invalid SSE URL format: "${sseUrl}"`;
      console.error('❌ SSE URL validation failed:', error, e);
      throw new Error(error);
    }

    // Pre-connection validation and health check (fire-and-forget)
    this.validateSSEConnection(baseURL, platform, token).catch(error => {
      console.warn('⚠️ SSE validation failed:', error);
    });

    // Create SSE connection with timestamp for error tracking
    const connectionStartTime = Date.now();
    console.log('🔌 Creating EventSource connection to:', sseUrl);
    this.sseConnection = new EventSource(sseUrl);

    // Generate unique listener ID
    const listenerId = Math.random().toString(36).substring(7);
    this.sseListeners.set(listenerId, onNotification);

    // Handle SSE events - store handlers for proper cleanup
    const openHandler = (event: Event) => {
      console.log('✅ SSE connection opened');
      if (onOpen) onOpen(event);
    };
    this.sseEventHandlers.onopen = openHandler;
    this.sseConnection.onopen = openHandler;

    const messageHandler = (event: MessageEvent) => {
      try {
        const rawData = JSON.parse(event.data);

        // Validate SSE notification with Zod schema
        const validatedNotification = validateSSENotification(rawData);
        if (!validatedNotification) {
          console.warn('Invalid SSE notification received, skipping:', rawData);
          return;
        }

        onNotification(validatedNotification as SSENotification);

        // Auto-acknowledge notification in background
        this.acknowledgeNotification(validatedNotification.id).catch(err => {
          console.debug(`Background acknowledgement failed for notification ${validatedNotification.id}:`, err);
        });
      } catch (error) {
        console.error('Failed to parse SSE notification:', error);
      }
    };
    this.sseEventHandlers.onmessage = messageHandler;
    this.sseConnection.onmessage = messageHandler;

    // Handle ping events (connection establishment)
    const pingHandler = (event: MessageEvent) => {
      console.log('🏓 SSE ping received:', event.data);
    };
    this.sseEventHandlers.ping = pingHandler;
    this.sseConnection.addEventListener('ping', pingHandler);

    const errorHandler = (event: Event) => {
      const currentState = this.sseConnection?.readyState;
      const stateNames = {
        0: 'CONNECTING',
        1: 'OPEN',
        2: 'CLOSED'
      };

      // Enhanced error logging with detailed diagnostic information
      console.error('❌ SSE connection error:', {
        // Connection state information
        readyState: currentState,
        stateName: stateNames[currentState as 0 | 1 | 2] || 'UNKNOWN',
        url: sseUrl,
        timestamp: new Date().toISOString(),

        // EventSource details
        lastEventId: this.sseConnection?.lastEventId,
        withCredentials: this.sseConnection?.withCredentials,

        // Browser environment info
        userAgent: navigator.userAgent,
        onlineStatus: navigator.onLine,
        cookieEnabled: navigator.cookieEnabled,

        // Current context
        platform: platform,
        hasToken: !!token,
        tokenLength: token?.length || 0,

        // URL breakdown for debugging
        urlParts: {
          baseURL: baseURL,
          routePath: API_ROUTES.NOTIFICATIONS.STREAM,
          queryString: queryString,
          paramsCount: params.toString().split('&').filter(p => p).length
        },

        // Available cookies (names only, for security)
        availableCookies: document.cookie ? document.cookie.split(';').map(c => c.trim().split('=')[0]) : [],

        // EventSource constants for reference
        constants: {
          CONNECTING: EventSource.CONNECTING,
          OPEN: EventSource.OPEN,
          CLOSED: EventSource.CLOSED
        },

        // Network information if available
        connection: (navigator as any).connection ? {
          effectiveType: (navigator as any).connection.effectiveType,
          downlink: (navigator as any).connection.downlink,
          rtt: (navigator as any).connection.rtt
        } : 'not available',

        // Event details (often empty but worth logging)
        eventDetails: {
          type: event.type,
          bubbles: event.bubbles,
          cancelable: event.cancelable,
          timeSinceConnection: Date.now() - connectionStartTime
        }
      });

      // Only reconnect if connection is actually closed
      if (currentState === EventSource.CLOSED) {
        console.log('🔴 Connection is CLOSED, triggering cleanup and reconnect logic');

        if (onError) onError(event);

        // Auto-reconnect if enabled (but not immediately to avoid tight loops)
        if (options.auto_reconnect !== false) {
          const interval = options.reconnect_interval || 5000;
          console.log(`🔄 Scheduling reconnection in ${interval}ms...`);

          setTimeout(() => {
            // Double-check connection is still closed and not recreated
            if (!this.sseConnection || this.sseConnection.readyState === EventSource.CLOSED) {
              console.log('🔁 Attempting reconnection...');
              this.connectToSSE(options, onNotification, onError, onOpen);
            } else {
              console.log('⏭️ Skipping reconnection, connection already exists:', this.sseConnection.readyState);
            }
          }, interval);
        }
      } else if (currentState === EventSource.CONNECTING) {
        console.log('⏳ Connection is still CONNECTING, waiting...');
      } else if (currentState === EventSource.OPEN) {
        console.log('⚠️ Connection is OPEN but error event fired (transient error)');
      }
    };
    this.sseEventHandlers.onerror = errorHandler;
    this.sseConnection.onerror = errorHandler;

    // Handle specific notification types (with event name as notification type)
    const notificationHandler = (event: MessageEvent) => {
      try {
        const notification: SSENotification = JSON.parse(event.data);
        onNotification(notification);

        // Auto-acknowledge notification in background
        this.acknowledgeNotification(notification.id).catch(err => {
          console.debug(`Background acknowledgement failed for notification ${notification.id}:`, err);
        });
      } catch (error) {
        console.error('Failed to parse notification event:', error);
      }
    };
    this.sseEventHandlers.notification = notificationHandler;
    this.sseConnection.addEventListener('notification', notificationHandler);

    // Return disconnect function with proper cleanup
    return () => {
      this.sseListeners.delete(listenerId);
      if (this.sseListeners.size === 0 && this.sseConnection) {
        this.cleanupSSEListeners();
        this.sseConnection.close();
        this.sseConnection = undefined;
      }
    };
  }

  /**
   * Clean up all SSE event listeners to prevent memory leaks
   */
  private cleanupSSEListeners(): void {
    if (!this.sseConnection) return;

    console.log('🧹 Cleaning up SSE event listeners');

    // Remove addEventListener handlers
    if (this.sseEventHandlers.ping) {
      this.sseConnection.removeEventListener('ping', this.sseEventHandlers.ping);
    }
    if (this.sseEventHandlers.notification) {
      this.sseConnection.removeEventListener('notification', this.sseEventHandlers.notification);
    }

    // Clear direct assignment handlers by setting to null
    if (this.sseEventHandlers.onopen) {
      this.sseConnection.onopen = null;
    }
    if (this.sseEventHandlers.onmessage) {
      this.sseConnection.onmessage = null;
    }
    if (this.sseEventHandlers.onerror) {
      this.sseConnection.onerror = null;
    }

    // Clear all tracked handlers
    this.sseEventHandlers = {};
  }

  /**
   * Disconnect from SSE with proper cleanup
   */
  disconnectFromSSE(): void {
    if (this.sseConnection) {
      this.cleanupSSEListeners();
      this.sseConnection.close();
      this.sseConnection = undefined;
    }
    this.sseListeners.clear();
  }

  /**
   * Check if SSE is connected
   */
  isSSEConnected(): boolean {
    return this.sseConnection?.readyState === EventSource.OPEN;
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  /**
   * Check if notification type is enabled in preferences
   */
  async isNotificationTypeEnabled(type: NotificationType): Promise<boolean> {
    try {
      const preferences = await this.getPreferences();
      return preferences.data.types[type] || false;
    } catch (error) {
      console.warn(`Failed to check notification type: ${error}`);
      return false;
    }
  }

  /**
   * Get notification priority color for UI
   */
  static getPriorityColor(priority: NotificationPriority): string {
    const colors = {
      low: '#10b981',    // green-500
      normal: '#3b82f6', // blue-500
      high: '#f59e0b',   // amber-500
      urgent: '#ef4444', // red-500
    };
    return colors[priority];
  }

  /**
   * Get notification type icon for UI
   */
  static getTypeIcon(type: NotificationType): string {
    const icons = {
      system: '⚙️',
      security: '🔒',
      permission: '🔑',
      user_management: '👥',
      wallet: '💼',
      payment: '💳',
      general: '📬',
    };
    return icons[type];
  }

  /**
   * Format notification timestamp
   */
  static formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    const minutes = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;

    return date.toLocaleDateString();
  }

  /**
   * Get authentication token information from accessible cookies
   * Since access tokens are HttpOnly, we get user info from client-side cookies
   */
  private getTokenFromCookies(): string | null {
    if (typeof document === 'undefined') {
      console.warn('🔐 Document not available, cannot extract authentication token');
      return null;
    }

    // Get platform from client (accessing private property like we do for baseURL)
    const platform = this.client['platform'] as 'admin' | 'frontend' | undefined;

    try {
      const cookies = document.cookie.split(';').reduce((acc, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) {
          acc[key] = value;
        }
        return acc;
      }, {} as Record<string, string>);

      let token: string | null = null;
      let tokenSource = '';

      // Try multiple authentication cookie sources
      const userCookie = cookies[COOKIES.user];
      if (userCookie) {
        try {
          const user = JSON.parse(decodeURIComponent(userCookie));
          if (user.access && typeof user.access === 'string' && user.access.length > 10) {
            token = user.access;
            tokenSource = 'user.access (JWT)';
          } else if (user.wallet_address && typeof user.wallet_address === 'string') {
            // Fallback for Web3 sessions - create legacy format token
            token = `web3_token_${user.wallet_address}`;
            tokenSource = 'user.wallet_address (Web3 legacy)';
          }
        } catch (error) {
          console.warn('⚠️ Failed to parse user cookie:', error);
        }
      }

      // Fallback: try direct session cookies (admin platform)
      if (!token && platform === 'admin') {
        const sessionCookie = cookies[COOKIES.session];
        if (sessionCookie && sessionCookie.length > 10) {
          token = sessionCookie;
          tokenSource = 'admin.session';
        }
      }

      // Additional fallback: check for any JWT-like tokens
      if (!token) {
        const cookieNames = Object.keys(cookies);
        for (const cookieName of cookieNames) {
          const value = cookies[cookieName];
          if (value && value.length > 50 && value.startsWith('eyJ')) { // JWT header
            token = value;
            tokenSource = `${cookieName} (JWT-like)`;
            break;
          }
        }
      }

      // Validate token format
      if (token) {
        if (token.startsWith('web3_token_')) {
          const wallet = token.substring(12); // Remove 'web3_token_' prefix
          if (wallet.length >= 20 && wallet.startsWith('0x')) {
            console.log(`🔑 Authentication token extracted from ${tokenSource}: ${wallet.substring(0, 8)}...${wallet.substring(wallet.length - 6)}`);
            return token;
          } else {
            console.warn('⚠️ Invalid Web3 token format in cookies');
            return null;
          }
        } else if (token.length > 20) {
          console.log(`🔑 JWT token extracted from ${tokenSource}: ${token.substring(0, 20)}...`);
          return token;
        } else {
          console.warn('⚠️ Token too short or invalid format');
          return null;
        }
      } else {
        console.warn(`⚠️ No authentication token found in cookies for platform: ${platform}`);
        console.warn(`🔍 Available cookie names: ${Object.keys(cookies).join(', ')}`);
        return null;
      }

    } catch (error) {
      console.error('❌ Error extracting authentication token from cookies:', error);
      return null;
    }
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create notifications API client for applications
 */
export function createNotificationsClient(client: UnifiedApiClient): NotificationsAPIClient {
  return new NotificationsAPIClient(client);
}

/**
 * Create notifications client with automatic platform detection
 */
export function createPlatformNotificationsClient(platform: 'frontend' | 'admin' = 'frontend'): NotificationsAPIClient {
  if (platform === 'admin') {
    const { createAdminApiClient } = require('../utils/api-client');
    return new NotificationsAPIClient(createAdminApiClient());
  } else {
    const { createFrontendApiClient } = require('../utils/api-client');
    return new NotificationsAPIClient(createFrontendApiClient());
  }
}