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

  constructor(private client: UnifiedApiClient) {}

  // ============================================================================
  // USER NOTIFICATIONS
  // ============================================================================

  /**
   * Get user notifications
   * Route: GET /api/notifications
   */
  async getNotifications(filters: NotificationFilters = {}): Promise<NotificationsResponse> {
    const response = await this.client.get<NotificationsResponse>(
      '/api/notifications',
      filters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch notifications: ${response.error}`);
    }

    return response.data;
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
      throw new Error(`Failed to mark notification as read: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Mark all notifications as read
   * Route: PUT /api/v1/auth/notifications/mark-all-read
   */
  async markAllAsRead(): Promise<{ success: boolean; updated_count: number }> {
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
      throw new Error(`Failed to mark all notifications as read: ${response.error}`);
    }

    return response.data;
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
      throw new Error(`Failed to delete notification: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Clear all notifications
   * Route: DELETE /api/v1/auth/notifications/clear-all
   */
  async clearAllNotifications(): Promise<{ success: boolean; deleted_count: number }> {
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
      throw new Error(`Failed to clear all notifications: ${response.error}`);
    }

    return response.data;
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
   * Route: POST /api/admin/notifications/send
   */
  async sendNotification(request: SendNotificationRequest): Promise<SendNotificationResponse> {
    const response = await this.client.post<SendNotificationResponse>(
      '/api/admin/notifications/send',
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to send notification: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get notification statistics (admin only)
   * Route: GET /api/admin/notifications/stats
   */
  async getNotificationStats(): Promise<NotificationStatsResponse> {
    const response = await this.client.get<NotificationStatsResponse>(
      '/api/admin/notifications/stats',
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
      throw new Error(`Failed to fetch notification stats: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get all notifications (admin only)
   * Route: GET /api/admin/notifications
   */
  async getAllNotifications(filters: NotificationFilters = {}): Promise<NotificationsResponse> {
    const response = await this.client.get<NotificationsResponse>(
      '/api/admin/notifications',
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
      throw new Error(`Failed to fetch all notifications: ${response.error}`);
    }

    return response.data;
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
    // Close existing connection if any
    if (this.sseConnection) {
      console.log('🔄 Closing existing SSE connection before creating new one');
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
      params.append('wallet_address', token);
      console.log(`🔑 SSE [${platform}]: User wallet found and added to connection URL: ${token}`);
    } else {
      console.warn(`⚠️ SSE [${platform}]: No authenticated user found in cookies, only broadcast notifications will be received`);
      console.warn(`Available cookies: ${document.cookie ? document.cookie.split(';').map(c => c.trim().split('=')[0]).join(', ') : 'none'}`);
    }

    // Only append query string if there are parameters
    const queryString = params.toString();
    const baseURL = this.client['baseURL'];
    const sseUrl = `${baseURL}/api/notifications/stream${queryString ? '?' + queryString : ''}`;

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

    // Create SSE connection
    console.log('🔌 Creating EventSource connection to:', sseUrl);
    this.sseConnection = new EventSource(sseUrl);

    // Generate unique listener ID
    const listenerId = Math.random().toString(36).substring(7);
    this.sseListeners.set(listenerId, onNotification);

    // Handle SSE events
    this.sseConnection.onopen = (event) => {
      console.log('✅ SSE connection opened');
      if (onOpen) onOpen(event);
    };

    this.sseConnection.onmessage = (event) => {
      try {
        const notification: SSENotification = JSON.parse(event.data);
        onNotification(notification);

        // Auto-acknowledge notification in background
        this.acknowledgeNotification(notification.id).catch(err => {
          console.debug(`Background acknowledgement failed for notification ${notification.id}:`, err);
        });
      } catch (error) {
        console.error('Failed to parse SSE notification:', error);
      }
    };

    // Handle ping events (connection establishment)
    this.sseConnection.addEventListener('ping', (event) => {
      console.log('🏓 SSE ping received:', event.data);
    });

    this.sseConnection.onerror = (event) => {
      const currentState = this.sseConnection?.readyState;
      const stateNames = {
        0: 'CONNECTING',
        1: 'OPEN',
        2: 'CLOSED'
      };

      // EventSource errors don't provide details, log what we can
      console.error('❌ SSE connection error:', {
        readyState: currentState,
        stateName: stateNames[currentState as 0 | 1 | 2] || 'UNKNOWN',
        url: sseUrl,
        timestamp: new Date().toISOString(),
        // EventSource.CONNECTING = 0, OPEN = 1, CLOSED = 2
        constants: {
          CONNECTING: EventSource.CONNECTING,
          OPEN: EventSource.OPEN,
          CLOSED: EventSource.CLOSED
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

    // Handle specific notification types (with event name as notification type)
    this.sseConnection.addEventListener('notification', (event: any) => {
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
    });

    // Return disconnect function
    return () => {
      this.sseListeners.delete(listenerId);
      if (this.sseListeners.size === 0 && this.sseConnection) {
        this.sseConnection.close();
        this.sseConnection = undefined;
      }
    };
  }

  /**
   * Disconnect from SSE
   */
  disconnectFromSSE(): void {
    if (this.sseConnection) {
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
    if (typeof document === 'undefined') return null;

    const cookies = document.cookie.split(';').reduce((acc, cookie) => {
      const [key, value] = cookie.trim().split('=');
      acc[key] = value;
      return acc;
    }, {} as Record<string, string>);

    // Get platform from client (accessing private property like we do for baseURL)
    const platform = this.client['platform'] as 'admin' | 'frontend' | undefined;

    let token: string | null = null;

    // Get user data from accessible client-side cookies
    if (platform === 'admin') {
      const userCookie = cookies[COOKIES.admin.user];
      if (userCookie) {
        try {
          const user = JSON.parse(decodeURIComponent(userCookie));
          // Return wallet address as token identifier (backend will verify via server-side cookies)
          token = user.wallet_address || user.sub;
        } catch (error) {
          console.warn('Failed to parse admin user cookie:', error);
        }
      }
    } else {
      const userCookie = cookies[COOKIES.user.user];
      if (userCookie) {
        try {
          const user = JSON.parse(decodeURIComponent(userCookie));
          // Return wallet address as token identifier (backend will verify via server-side cookies)
          token = user.wallet_address || user.sub;
        } catch (error) {
          console.warn('Failed to parse user cookie:', error);
        }
      }
    }

    return token;
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