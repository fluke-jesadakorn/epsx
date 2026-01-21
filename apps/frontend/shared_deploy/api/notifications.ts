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

import { COOKIES } from '../auth/cookies';
import {
  NotificationsResponseSchema,
  validateNotificationFilters,
  validateSendNotificationRequest,
  validateSSENotification
} from '../components/notifications/schemas';
import { API_ROUTES } from '../config/route-constants';
import { UnifiedApiClient } from '../utils/api-client';

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
  private isReconnecting = false; // Prevent reconnection race conditions

  constructor(private client: UnifiedApiClient) { }

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
        API_ROUTES.USERS.NOTIFICATIONS,
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

      // The API client normalizes responses - response IS the full backend response
      // with { success, data, api_version, access_level }
      // Validate the full response against the schema
      const validatedResponse = NotificationsResponseSchema.safeParse(response);
      if (!validatedResponse.success) {
        console.warn('API response validation failed:', validatedResponse.error);
        // Continue with unvalidated data but log the issue
      }

      // Return the full response structure (with success, data, etc.)
      return response as unknown as NotificationsResponse;
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
      `${API_ROUTES.USERS.NOTIFICATIONS}/unread-count`,
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
        `${API_ROUTES.USERS.NOTIFICATIONS}/${notificationId}/read`,
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
   * Route: PUT /api/auth/notifications/mark-all-read
   */
  async markAllAsRead(): Promise<{ success: boolean; updated_count: number }> {
    try {
      const response = await this.client.put<{ success: boolean; updated_count: number }>(
        `${API_ROUTES.USERS.NOTIFICATIONS}/mark-all-read`,
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
      `${API_ROUTES.USERS.NOTIFICATIONS}/${notificationId}/acknowledge`,
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
   * Route: DELETE /api/auth/notifications/{id}
   */
  async deleteNotification(notificationId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.client.delete<{ success: boolean; message: string }>(
        `${API_ROUTES.USERS.NOTIFICATIONS}/${notificationId}`,
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
   * Route: DELETE /api/auth/notifications/clear-all
   */
  async clearAllNotifications(): Promise<{ success: boolean; deleted_count: number }> {
    try {
      const response = await this.client.delete<{ success: boolean; deleted_count: number }>(
        `${API_ROUTES.USERS.NOTIFICATIONS}/clear-all`,
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
   * Route: GET /api/auth/notifications/preferences
   */
  async getPreferences(): Promise<NotificationPreferencesResponse> {
    const response = await this.client.get<NotificationPreferencesResponse>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/preferences`,
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
   * Route: PUT /api/auth/notifications/preferences
   */
  async updatePreferences(preferences: Partial<NotificationPreferences>): Promise<{
    success: boolean;
    message: string;
  }> {
    const response = await this.client.put<{ success: boolean; message: string }>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/preferences`,
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
   * Route: POST /api/auth/notifications/push/subscribe
   */
  async subscribeToPush(subscription: PushSubscription): Promise<PushSubscriptionResponse> {
    const response = await this.client.post<PushSubscriptionResponse>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/push/subscribe`,
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
   * Route: DELETE /api/auth/notifications/push/unsubscribe
   */
  async unsubscribeFromPush(): Promise<{ success: boolean; message: string }> {
    const response = await this.client.delete<{ success: boolean; message: string }>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/push/unsubscribe`,
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
   * Route: GET /api/auth/notifications/push/status
   */
  async getPushStatus(): Promise<{
    subscribed: boolean;
    subscription_id?: string;
    created_at?: string;
  }> {
    const response = await this.client.get(
      `${API_ROUTES.USERS.NOTIFICATIONS}/push/status`,
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
   * Route: POST /api/admin/notifications/send
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
        `${API_ROUTES.ADMIN.NOTIFICATIONS}/send`,
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
   * Route: GET /api/admin/notifications/stats
   */
  async getNotificationStats(): Promise<NotificationStatsResponse> {
    try {
      const response = await this.client.get<NotificationStatsResponse>(
        `${API_ROUTES.ADMIN.NOTIFICATIONS}/stats`,
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
   * Route: GET /api/admin/notifications
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

      // Return the full response structure (with success, data, etc.)
      return response as unknown as NotificationsResponse;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'fetch all notifications (admin)', { filters });
    }
  }

  /**
   * Delete notification (admin only)
   * Route: DELETE /api/admin/notifications/{id}
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
        handleNotificationError(response, 'delete admin notification', { notificationId });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) throw error;
      handleNotificationError(error, 'delete admin notification', { notificationId });
    }
  }

  // ============================================================================
  // SERVER-SENT EVENTS (SSE)
  // ============================================================================

  /**
   * Connect to SSE for real-time notifications
   * Route: GET /api/auth/notifications/stream
   */
  connectToSSE(
    options: SSEConnectionOptions = {},
    onNotification: (notification: SSENotification) => void,
    onError?: (error: Event) => void,
    onOpen?: (event: Event) => void
  ): () => void {
    // Prevent multiple connection attempts
    if (this.sseConnection && this.sseConnection.readyState === EventSource.CONNECTING) {
      console.log('⏭️ SSE: Connection already in progress, skipping duplicate request');
      return () => { };
    }

    // Close existing connection if any with proper cleanup
    if (this.sseConnection) {
      console.log('🔄 Closing existing SSE connection before creating new one');
      this.cleanupSSEListeners();
      this.sseConnection.close();
      this.sseConnection = undefined;
    }

    // Reset reconnection flag
    this.isReconnecting = false;

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
    // NOTE: access_token is HttpOnly, so we can't read it here.
    // The middleware proxy at /api/proxy will inject the Authorization header for us.
    const token = this.getTokenFromCookies();
    const platform = this.client['platform'] as 'admin' | 'frontend' | undefined;

    if (token) {
      params.append('token', token);
      console.log(`🔑 SSE [${platform}]: Client-side token found and added to URL`);
    } else {
      // access_token is HttpOnly, so this is expected.
      // The proxy middleware will handle injecting the bearer token.
      console.log(`ℹ️ SSE [${platform}]: No client-side token found (expected for HttpOnly cookies). Relying on Proxy Middleware injection.`);
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
      // Use window.location.origin as base for relative URLs to ensure validation passes
      const base = typeof window !== 'undefined' ? window.location.origin : 'http://localhost';
      const urlObj = new URL(sseUrl, base);

      console.log('✅ SSE URL validated:', {
        protocol: urlObj.protocol,
        host: urlObj.host,
        pathname: urlObj.pathname,
        search: urlObj.search,
        fullURL: urlObj.toString(),
        isRelative: !sseUrl.startsWith('http')
      });
    } catch (e) {
      const error = `Invalid SSE URL format: "${sseUrl}"`;
      console.error('❌ SSE URL validation failed:', error, e);
      throw new Error(error);
    }

    // Pre-connection health check - simple connectivity test
    if (baseURL && token) {
      console.log(`🔍 SSE [${platform}]: Pre-connection check - baseURL: ${baseURL}, hasToken: true`);
    } else {
      console.warn(`⚠️ SSE [${platform}]: Pre-connection check - baseURL: ${baseURL ? 'valid' : 'missing'}, hasToken: ${!!token}`);
    }

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
      this.isReconnecting = false; // Reset reconnection flag on successful connection
      if (onOpen) onOpen(event);
    };
    this.sseEventHandlers.onopen = openHandler;
    this.sseConnection.onopen = openHandler;

    const messageHandler = (event: MessageEvent) => {
      try {
        // Check payload size to prevent WebSocket errors
        const payloadSize = event.data.length;
        const MAX_PAYLOAD_SIZE = 64 * 1024; // 64KB limit
        if (payloadSize > MAX_PAYLOAD_SIZE) {
          console.error(`SSE payload too large: ${payloadSize} bytes (max: ${MAX_PAYLOAD_SIZE})`);
          return;
        }

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

      // Only log as error when connection is actually CLOSED
      // For CONNECTING (0) and OPEN (1) states, these are transient or spurious events
      if (currentState === EventSource.CLOSED) {
        const timeSinceConnection = Date.now() - connectionStartTime;
        const isImmediateFailure = timeSinceConnection < 1000;
        const isExpectedFailure = isImmediateFailure || !token;

        // Use debug level for expected failures (no auth, immediate close) to reduce console noise
        // Use warn level for unexpected connection drops during active session
        const logLevel = isExpectedFailure ? 'debug' : 'warn';
        const logFn = console[logLevel];

        logFn(`${isExpectedFailure ? 'ℹ️' : '⚠️'} SSE connection closed${isExpectedFailure ? ' (expected)' : ''}:`, {
          readyState: currentState,
          stateName: 'CLOSED',
          url: sseUrl,
          timestamp: new Date().toISOString(),
          platform: platform,
          hasToken: !!token,
          timeSinceConnection,
          reason: !token ? 'No authentication token' : isImmediateFailure ? 'Immediate closure (endpoint may not exist)' : 'Connection dropped'
        });
      } else if (currentState === EventSource.CONNECTING) {
        // Connection is still connecting - this is likely a transient state event
        // Only log as debug to reduce console noise
        console.debug('⏳ SSE: Transient error during connection (still CONNECTING)');
      } else if (currentState === EventSource.OPEN) {
        // Connection is open but error fired - likely a transient network glitch
        console.warn('⚠️ SSE: Error event fired while connection is OPEN (transient)');
      }

      // Only reconnect if connection is actually closed and not already reconnecting
      if (currentState === EventSource.CLOSED && !this.isReconnecting) {
        console.log('🔴 Connection is CLOSED, triggering cleanup and reconnect logic');

        if (onError) onError(event);

        // Auto-reconnect if enabled (but not immediately to avoid tight loops)
        if (options.auto_reconnect !== false) {
          this.isReconnecting = true; // Prevent multiple reconnect attempts
          const interval = options.reconnect_interval || 5000;
          console.log(`🔄 Scheduling reconnection in ${interval}ms...`);

          setTimeout(() => {
            // Double-check connection is still closed and not recreated
            if (!this.sseConnection || this.sseConnection.readyState === EventSource.CLOSED) {
              console.log('🔁 Attempting reconnection...');
              this.connectToSSE(options, onNotification, onError, onOpen);
            } else {
              console.log('⏭️ Skipping reconnection, connection already exists:', this.sseConnection.readyState);
              this.isReconnecting = false;
            }
          }, interval);
        }
      } else if (currentState === EventSource.CONNECTING) {
        console.log('⏳ Connection is still CONNECTING, waiting...');
      } else if (currentState === EventSource.OPEN) {
        console.log('⚠️ Connection is OPEN but error event fired (transient error)');
      } else if (this.isReconnecting) {
        console.log('⏭️ Already reconnecting, skipping additional reconnect attempts');
      }
    };
    this.sseEventHandlers.onerror = errorHandler;
    this.sseConnection.onerror = errorHandler;

    // Handle specific notification types (with event name as notification type)
    const notificationHandler = (event: MessageEvent) => {
      try {
        // Check payload size to prevent WebSocket errors
        const payloadSize = event.data.length;
        const MAX_PAYLOAD_SIZE = 64 * 1024; // 64KB limit
        if (payloadSize > MAX_PAYLOAD_SIZE) {
          console.error(`SSE notification payload too large: ${payloadSize} bytes (max: ${MAX_PAYLOAD_SIZE})`);
          return;
        }

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
      this.isReconnecting = false; // Reset reconnection flag
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
    this.isReconnecting = false; // Reset reconnection flag
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
      return (preferences.data.types as any)[type] || false;
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
    return (colors as any)[priority];
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
    return (icons as any)[type];
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

      // Declare token variables
      let token: string | null = null;
      let tokenSource = 'unknown';

      // Get user data from accessible client-side cookies
      const userCookie = cookies[COOKIES.user];
      if (userCookie) {
        try {
          const user = JSON.parse(decodeURIComponent(userCookie));
          // Return JWT access token for backend verification
          token = user.access;
        } catch (error) {
          console.warn('Failed to parse user cookie:', error);
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

        // Validate token format - prefer JWT over legacy
        if (token) {
          // Preferred: JWT format (starts with eyJ - base64 encoded JSON header)
          if (token.startsWith('eyJ')) {
            console.log(`🔑 JWT token extracted from ${tokenSource}: ${token.substring(0, 20)}...`);
            return token;
          }

          // DEPRECATED: Legacy web3_token_ format for backward compatibility
          if (token.startsWith('web3_token_')) {
            const wallet = token.substring(12); // Remove 'web3_token_' prefix
            if (wallet.length >= 20 && wallet.startsWith('0x')) {
              console.warn('⚠️ DEPRECATED: Legacy web3_token_ format detected. Please re-authenticate to get a JWT.');
              console.log(`🔑 Legacy token extracted from ${tokenSource}: ${wallet.substring(0, 8)}...${wallet.substring(wallet.length - 6)}`);
              return token;
            } else {
              console.warn('⚠️ Invalid Web3 token format in cookies');
              return null;
            }
          }

          // Fallback for other valid-looking tokens
          if (token.length > 50) {
            console.log(`🔑 Token extracted from ${tokenSource}: ${token.substring(0, 20)}...`);
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
      }

    } catch (error) {
      console.error('❌ Error extracting authentication token from cookies:', error);
      return null;
    }

    // Fallback return (should never reach here)
    return null;
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

// Type alias for backward compatibility with useApiClient
export type NotificationsApi = NotificationsAPIClient;