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
  validateNotificationFilters,
  validateSendNotificationRequest,
  validateSSENotification
} from '../components/notifications/schemas';
import { API_ROUTES } from '../config/route-constants';
import { isApiResponse, type ApiResponse } from '../types/api';
import type { UnifiedApiClient } from '../utils/api-client';
import { logger } from '../utils/logger';

// ============================================================================
// ERROR HANDLING
// ============================================================================

export interface NotificationAPIErrorOptions {
  status?: number;
  details?: Record<string, unknown>;
}

export class NotificationAPIError extends Error {
  public readonly status?: number;
  public readonly details?: Record<string, unknown>;

  constructor(
    message: string,
    public readonly code: string,
    options: NotificationAPIErrorOptions = {}
  ) {
    super(message);
    this.status = options.status;
    this.details = options.details;
    this.name = 'NotificationAPIError';
    Object.setPrototypeOf(this, NotificationAPIError.prototype);
  }
}

export class NotificationNotFoundError extends NotificationAPIError {
  constructor(notificationId: string, details?: Record<string, unknown>) {
    super(
      `Notification not found: ${notificationId}`,
      'NOTIFICATION_NOT_FOUND',
      { status: 404, details }
    );
    this.name = 'NotificationNotFoundError';
  }
}

export class NotificationPermissionError extends NotificationAPIError {
  constructor(operation: string, details?: Record<string, unknown>) {
    super(
      `Permission denied for operation: ${operation}`,
      'NOTIFICATION_PERMISSION_DENIED',
      { status: 403, details }
    );
    this.name = 'Notificationpermission-error';
  }
}

export class NotificationValidationError extends NotificationAPIError {
  constructor(message: string, details?: Record<string, unknown>) {
    super(message, 'NOTIFICATION_VALIDATION_ERROR', { status: 400, details });
    this.name = 'NotificationValidationError';
  }
}

const API_VERSION_HEADER = 'X-API-Version';
const ACCESS_LEVEL_HEADER = 'X-Access-Level';
const ADMIN_CONTEXT_HEADER = 'X-Admin-Context';
const V1 = 'v1';
const AUTH_LEVEL = 'auth';
const ADMIN_LEVEL = 'admin';
const TRUE_STR = 'true';
const UNKNOWN_ERROR_MSG = 'Unknown error';

function getErrorMessage(error: unknown): string {
  if (isApiResponse(error)) {
    return error.error?.message ?? 'API error occurred';
  }

  if (error !== null && typeof error === 'object') {
    const err = error as Record<string, unknown>;
    const errProperty = err.error;
    if (errProperty !== null && typeof errProperty === 'object') {
      const apiError = errProperty as Record<string, unknown>;
      return (apiError.message as string | undefined) ?? (apiError.error as string | undefined) ?? JSON.stringify(errProperty);
    }
    return (err.error as string | undefined) ?? (err.message as string | undefined) ?? 'Unknown error occurred';
  }

  return String(error);
}

function extractErrorStatus(error: unknown): number | undefined {
  if (isApiResponse(error)) {
    const apiResp = error as unknown as Record<string, unknown>;
    return typeof apiResp.status === 'number' ? apiResp.status : 400;
  }

  if (error !== null && typeof error === 'object') {
    const anyError = error as Record<string, unknown>;
    return (anyError.status as number | undefined) ?? (anyError.response as Record<string, unknown> | undefined)?.status as number | undefined;
  }

  return undefined;
}

function handleNotificationError(error: unknown, operation: string, details?: Record<string, unknown>): never {
  if (error instanceof NotificationAPIError) {
    throw error;
  }

  const errorMessage = getErrorMessage(error);
  const status = extractErrorStatus(error);

  const notificationId = (typeof details?.notificationId === 'string' && details.notificationId !== '') ? details.notificationId : undefined;

  if (status === 404) {
    throw new NotificationNotFoundError(notificationId ?? 'unknown', { operation, originalError: errorMessage });
  }

  if (status === 403) {
    throw new NotificationPermissionError(operation, { originalError: errorMessage, ...details });
  }

  if (status === 400) {
    throw new NotificationValidationError(`Failed to ${operation}: ${errorMessage}`, { originalError: errorMessage, ...details });
  }

  throw new NotificationAPIError(
    `Failed to ${operation}: ${errorMessage}`,
    'NOTIFICATION_API_ERROR',
    { status, details: { operation, originalError: errorMessage, ...details } }
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
  data?: Record<string, unknown>;
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
  | 'general'
  | 'announcement'
  | 'advertisement'
  | 'chat';

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

export interface NotificationsData {
  notifications: Notification[];
  total_count: number;
  unread_count: number;
  page: number;
  limit: number;
  total_pages: number;
}

export type NotificationsResponse = ApiResponse<NotificationsData>;

export interface NotificationPreferences {
  email_enabled: boolean;
  push_enabled: boolean;
  sms_enabled?: boolean;
  types: {
    system: boolean;
    security: boolean;
    permission: boolean;
    wallet_management: boolean;
    wallet: boolean;
    payment: boolean;
    general: boolean;
    announcement: boolean;
    advertisement: boolean;
    chat: boolean;
  };
  priority_filter: NotificationPriority;
  quiet_hours?: {
    enabled: boolean;
    start_time: string;
    end_time: string;
    timezone: string;
  };
}

export type NotificationPreferencesResponse = ApiResponse<NotificationPreferences>;

export interface SendNotificationRequest {
  recipient_wallet_address?: string;
  recipient_group?: string;
  broadcast?: boolean;
  notification_type: NotificationType;
  priority: NotificationPriority;
  title: string;
  message: string;
  data?: Record<string, unknown>;
  action_url?: string;
  image_url?: string;
  expires_at?: string;
  schedule_at?: string;
}

export interface SendNotificationData {
  notification_id: string;
  recipients_count: number;
  scheduled: boolean;
  delivery_status: 'sent' | 'scheduled' | 'failed';
}

export type SendNotificationResponse = ApiResponse<SendNotificationData> & { message?: string };

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

export type NotificationStatsResponse = ApiResponse<NotificationStats>;

export interface PushSubscription {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
  user_agent?: string;
  device_type?: 'desktop' | 'mobile' | 'tablet';
}

export interface PushSubscriptionData {
  subscription_id: string;
  active: boolean;
  created_at: string;
}

export type PushSubscriptionResponse = ApiResponse<PushSubscriptionData> & { message?: string };

// ============================================================================
// SSE TYPES
// ============================================================================

export interface SSENotification {
  id: string;
  wallet_address: string;
  notification_type: NotificationType;
  title: string;
  message: string;
  data?: Record<string, unknown>;
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

      const response = await this.client.get<NotificationsData>(
        API_ROUTES.USERS.NOTIFICATIONS,
        filterValidation.data,
        {
          headers: {
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'fetch notifications', { filters });
      }

      // The API client normalizes responses
      // Validate the full response against the schema
      return response;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch unread count: ${response.error?.message ?? UNKNOWN_ERROR_MSG}`);
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
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'mark notification as read', { notificationId });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'mark all notifications as read');
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      // Don't throw error for acknowledgement failures - just log
      logger.warn(`Failed to acknowledge notification ${notificationId}`, { error: response.error?.message });
      return { success: false, message: response.error?.message ?? 'Failed to acknowledge notification' };
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
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'delete notification', { notificationId });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'clear all notifications');
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
  async getPreferences(): Promise<NotificationPreferences> {
    const response = await this.client.get<NotificationPreferences>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/preferences`,
      undefined,
      {
        headers: {
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch notification preferences: ${response.error?.message ?? UNKNOWN_ERROR_MSG}`);
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
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to update notification preferences: ${response.error?.message ?? UNKNOWN_ERROR_MSG}`);
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
  async subscribeToPush(subscription: PushSubscription): Promise<PushSubscriptionData> {
    const response = await this.client.post<PushSubscriptionData>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/push/subscribe`,
      subscription,
      {
        headers: {
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to subscribe to push notifications: ${response.error?.message ?? UNKNOWN_ERROR_MSG}`);
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
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to unsubscribe from push notifications: ${response.error?.message ?? UNKNOWN_ERROR_MSG}`);
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
    const response = await this.client.get<{ data: { subscribed: boolean; subscription_id?: string; created_at?: string } }>(
      `${API_ROUTES.USERS.NOTIFICATIONS}/push/status`,
      undefined,
      {
        headers: {
          [API_VERSION_HEADER]: V1,
          [ACCESS_LEVEL_HEADER]: AUTH_LEVEL,
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get push notification status: ${response.error?.message ?? UNKNOWN_ERROR_MSG}`);
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
  async sendNotification(request: SendNotificationRequest): Promise<SendNotificationData> {
    try {
      // Validate request payload
      const requestValidation = validateSendNotificationRequest(request);
      if (!requestValidation.success) {
        throw new NotificationValidationError(
          'Invalid send notification request',
          requestValidation.errors.format()
        );
      }

      const response = await this.client.post<SendNotificationData>(
        `${API_ROUTES.ADMIN.NOTIFICATIONS}/send`,
        requestValidation.data,
        {
          headers: {
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: ADMIN_LEVEL,
            [ADMIN_CONTEXT_HEADER]: TRUE_STR,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'send notification', { request });
      }

      return response.data;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
      handleNotificationError(error, 'send notification', { request });
    }
  }

  /**
   * Get notification statistics (admin only)
   * Route: GET /api/admin/notifications/stats
   */
  async getNotificationStats(): Promise<NotificationStatsResponse> {
    try {
      const response = await this.client.get<NotificationStats>(
        `${API_ROUTES.ADMIN.NOTIFICATIONS}/stats`,
        undefined,
        {
          headers: {
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: ADMIN_LEVEL,
            [ADMIN_CONTEXT_HEADER]: TRUE_STR,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'fetch notification stats');
      }

      return response;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
      handleNotificationError(error, 'fetch notification stats');
    }
  }

  /**
   * Get all notifications (admin only)
   * Route: GET /api/admin/notifications
   */
  async getAllNotifications(filters: NotificationFilters = {}): Promise<NotificationsResponse> {
    try {
      const response = await this.client.get<NotificationsData>(
        API_ROUTES.ADMIN.NOTIFICATIONS,
        filters as Record<string, unknown>,
        {
          headers: {
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: ADMIN_LEVEL,
            [ADMIN_CONTEXT_HEADER]: TRUE_STR,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'fetch all notifications (admin)', { filters });
      }

      return response;
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
            [API_VERSION_HEADER]: V1,
            [ACCESS_LEVEL_HEADER]: ADMIN_LEVEL,
            [ADMIN_CONTEXT_HEADER]: TRUE_STR,
          },
        }
      );

      if (!this.client.isApiSuccess(response)) {
        handleNotificationError(response, 'delete admin notification', { notificationId });
      }

      return response.data as { success: boolean; message: string };
    } catch (error) {
      if (error instanceof NotificationAPIError) { throw error; }
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
    callbacks: {
      onNotification: (notification: SSENotification) => void;
      onError?: (error: Event) => void;
      onOpen?: (event: Event) => void;
    }
  ): () => void {
    const { onNotification, onError, onOpen } = callbacks;
    if (this.sseConnection && this.sseConnection.readyState === EventSource.CONNECTING) {
      logger.info('⏭️ SSE: Connection already in progress, skipping duplicate request');
      return () => { };
    }

    this.disconnectFromSSE();
    this.isReconnecting = false;

    // Reset reconnection flag
    this.isReconnecting = false;

    // Check EventSource support
    if (typeof EventSource === 'undefined') {
      const error = 'EventSource not supported in this browser';
      logger.error(error);
      throw new Error(error);
    }

    const { sseUrl, token, platform } = this.buildSSEUrl(options);

    // Create SSE connection with timestamp for error tracking
    const connectionStartTime = Date.now();
    logger.info('🔌 Creating EventSource connection', { url: sseUrl });
    this.sseConnection = new EventSource(sseUrl);

    // Store handlers for cleanup
    this.setupSSEHandlers({
      options,
      onNotification,
      onError,
      onOpen,
      connectionStartTime,
      token,
      platform,
      sseUrl,
    });

    const listenerId = Math.random().toString(36).slice(7);
    this.sseListeners.set(listenerId, onNotification);

    return () => {
      this.isReconnecting = false;
      this.sseListeners.delete(listenerId);
      if (this.sseListeners.size === 0 && this.sseConnection) {
        this.disconnectFromSSE();
      }
    };
  }

  private setupSSEHandlers(context: {
    options: SSEConnectionOptions;
    onNotification: (notification: SSENotification) => void;
    onError?: (error: Event) => void;
    onOpen?: (event: Event) => void;
    connectionStartTime: number;
    token: string | null;
    platform: string | undefined;
    sseUrl: string;
  }): void {
    if (!this.sseConnection) { return; }

    this.sseEventHandlers.onopen = (event: Event) => {
      logger.info('✅ SSE connection opened');
      this.isReconnecting = false;
      if (context.onOpen) { context.onOpen(event); }
    };
    this.sseConnection.onopen = this.sseEventHandlers.onopen;

    this.sseEventHandlers.onmessage = (event: MessageEvent) => {
      this.handleSSEMessage(event, context.onNotification);
    };
    this.sseConnection.onmessage = this.sseEventHandlers.onmessage;

    this.sseEventHandlers.ping = (event: MessageEvent) => {
      logger.info('🏓 SSE ping received', { data: event.data as unknown });
    };
    this.sseConnection.addEventListener('ping', this.sseEventHandlers.ping);

    this.sseEventHandlers.onerror = (event: Event) => {
      this.handleSSEError(event, context);
    };
    this.sseConnection.onerror = this.sseEventHandlers.onerror;

    this.sseEventHandlers.notification = (event: MessageEvent) => {
      this.handleSSEMessage(event, context.onNotification);
    };
    this.sseConnection.addEventListener('notification', this.sseEventHandlers.notification);
  }

  /**
   * Internal handler for SSE messages
   */
  private handleSSEMessage(event: MessageEvent, onNotification: (notification: SSENotification) => void): void {
    try {
      const data = event.data as string;
      const payloadSize = data.length;
      const MAX_PAYLOAD_SIZE = 64 * 1024; // 64KB limit
      if (payloadSize > MAX_PAYLOAD_SIZE) {
        logger.error(`SSE payload too large: ${payloadSize} bytes (max: ${MAX_PAYLOAD_SIZE})`);
        return;
      }

      const rawData = JSON.parse(data) as Record<string, unknown>;

      // Validate SSE notification with Zod schema
      const validatedNotification = validateSSENotification(rawData);
      if (!validatedNotification) {
        logger.warn('Invalid SSE notification received, skipping', { data: rawData });
        return;
      }

      const notification = validatedNotification as unknown as SSENotification;
      onNotification(notification);

      // Auto-acknowledge notification in background
      this.backgroundAcknowledge(notification.id);
    } catch (error) {
      logger.error('Failed to parse SSE notification', { error });
    }
  }

  private backgroundAcknowledge(notificationId: string): void {
    const acknowledge = async () => {
      try {
        await this.acknowledgeNotification(notificationId);
      } catch (err) {
        logger.info(`Background acknowledgement failed for notification ${notificationId}`, { error: err });
      }
    };
    void acknowledge();
  }

  /**
   * Internal handler for SSE errors
   */
  private handleSSEError(event: Event, context: {
    options: SSEConnectionOptions;
    onNotification: (notification: SSENotification) => void;
    onError?: (error: Event) => void;
    onOpen?: (event: Event) => void;
    connectionStartTime: number;
    token: string | null;
    platform: string | undefined;
    sseUrl: string;
  }): void {
    const { currentState, timeSinceConnection } = this.getSSEErrorState(context.connectionStartTime);

    if (currentState === EventSource.CLOSED) {
      this.logSSEClosedError(context, timeSinceConnection);
    } else if (currentState === EventSource.CONNECTING) {
      logger.info('⏳ SSE: Transient error during connection (still CONNECTING)');
    } else if (currentState === EventSource.OPEN) {
      logger.info('⚠️ SSE: Error event fired while connection is OPEN (transient)');
    }

    if (currentState === EventSource.CLOSED && !this.isReconnecting) {
      this.triggerSSECleanupAndReconnect({ ...context, event });
    } else if (this.isReconnecting) {
      logger.info('⏭️ Already reconnecting, skipping additional reconnect attempts');
    }
  }

  private getSSEErrorState(connectionStartTime: number): { currentState: number | undefined, timeSinceConnection: number } {
    return {
      currentState: this.sseConnection?.readyState,
      timeSinceConnection: Date.now() - connectionStartTime
    };
  }

  private logSSEClosedError(context: { token: string | null, platform: string | undefined, sseUrl: string }, timeSinceConnection: number): void {
    const { token, platform, sseUrl } = context;
    const isImmediateFailure = timeSinceConnection < 1000;
    const isExpectedFailure = isImmediateFailure || token === null || token === '';

    logger.info(`${isExpectedFailure ? 'ℹ️' : '⚠️'} SSE connection closed${isExpectedFailure ? ' (expected)' : ''}`, {
      url: sseUrl,
      platform,
      hasToken: Boolean(token),
      timeSinceConnection,
      reason: (token === null || token === '') ? 'No authentication token' : isImmediateFailure ? 'Immediate closure' : 'Connection dropped'
    });
  }

  /**
   * Internal trigger for SSE cleanup and reconnect
   */
  private triggerSSECleanupAndReconnect(context: {
    event: Event,
    options: SSEConnectionOptions,
    onNotification: (notification: SSENotification) => void,
    onError?: (error: Event) => void,
    onOpen?: (event: Event) => void
  }): void {
    const { event, options, onNotification, onError, onOpen } = context;
    logger.info('🔴 Connection is CLOSED, triggering cleanup and reconnect logic');

    if (onError) { onError(event); }

    if (options.auto_reconnect !== false) {
      this.isReconnecting = true;
      const interval = options.reconnect_interval ?? 5000;
      logger.info(`🔄 Scheduling reconnection in ${interval}ms...`);

      if (typeof window !== 'undefined') {
        window.setTimeout(() => {
          if (!this.sseConnection || this.sseConnection.readyState === EventSource.CLOSED) {
            logger.info('🔁 Attempting reconnection...');
            this.connectToSSE(options, { onNotification, onError, onOpen });
          } else {
            logger.info('⏭️ Skipping reconnection, connection already exists', { status: this.sseConnection.readyState });
            this.isReconnecting = false;
          }
        }, interval);
      }
    }
  }

  /**
   * Clean up all SSE event listeners to prevent memory leaks
   */
  private cleanupSSEListeners(): void {
    if (!this.sseConnection) { return; }

    logger.info('🧹 Cleaning up SSE event listeners');

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
      return (preferences.types as Record<string, boolean>)[type] ?? false;
    } catch (error) {
      logger.warn(`Failed to check notification type: ${error}`);
      return false;
    }
  }

  /**
   * Get notification priority color for UI
   */
  static getPriorityColor(priority: NotificationPriority): string {
    const colors: Record<string, string> = {
      low: '#10b981',    // green-500
      normal: '#3b82f6', // blue-500
      high: '#f59e0b',   // amber-500
      critical: '#ef4444', // red-500
    };
    return colors[priority] ?? '#10b981';
  }

  /**
   * Get notification type icon for UI
   */
  static getTypeIcon(type: NotificationType): string {
    const icons: Record<string, string> = {
      system: '⚙️',
      security: '🔒',
      permission: '🔑',
      wallet_management: '👥',
      wallet: '💼',
      payment: '💳',
      general: '📬',
      announcement: '📢',
      advertisement: '📣',
      chat: '💬',
    };
    return icons[type] ?? '📬';
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

    if (minutes < 1) { return 'Just now'; }
    if (minutes < 60) { return `${minutes}m ago`; }
    if (hours < 24) { return `${hours}h ago`; }
    if (days < 7) { return `${days}d ago`; }

    return date.toLocaleDateString();
  }

  /**
   * Get authentication token information from accessible cookies
   * Since access tokens are HttpOnly, we get user info from client-side cookies
   */
  private getTokenFromCookies(): string | null {
    if (typeof document === 'undefined') {
      logger.warn('🔐 Document not available, cannot extract authentication token');
      return null;
    }

    const platform = this.client.getPlatform();

    try {
      const cookies = this.parseBrowserCookies();
      const userCookie = cookies[COOKIES.user] as string | undefined;
      if (userCookie === undefined || userCookie === '') {
        logger.warn(`⚠️ No user cookie found in cookies for platform: ${platform}`);
        return null;
      }

      return this.extractTokenFromUserCookie(userCookie, cookies, platform);
    } catch (error) {
      logger.error('❌ Error extracting authentication token from cookies', { error });
      return null;
    }
  }

  /**
   * Parse browser cookies into a record
   */
  private parseBrowserCookies(): Record<string, string> {
    return document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
      const [key, value] = cookie.trim().split('=');
      if (key && value) {
        acc[key] = value;
      }
      return acc;
    }, {});
  }

  /**
   * Extract token from user cookie with fallback to other JWT tokens
   */
  private extractTokenFromUserCookie(userCookie: string, cookies: Record<string, string>, platform: string | undefined): string | null {
    let token: string | null = null;
    let tokenSource = 'user-cookie';

    try {
      const user = JSON.parse(decodeURIComponent(userCookie)) as { access?: string };
      token = user.access ?? null;
    } catch (error) {
      logger.warn('Failed to parse user cookie', { error });
    }

    // Fallback: check for any JWT-like tokens
    if (token === null || token === '') {
      for (const [cookieName, value] of Object.entries(cookies)) {
        if (value !== '' && value.length > 50 && value.startsWith('eyJ')) {
          token = value;
          tokenSource = `${cookieName} (JWT-like)`;
          break;
        }
      }
    }

    if (token === null || token === '') {
      logger.warn(`⚠️ No authentication token found in cookies for platform: ${platform}`);
      return null;
    }

    return this.validateAndLogToken(token, tokenSource);
  }

  /**
   * Validate token format and log its extraction
   */
  private validateAndLogToken(token: string, source: string): string | null {
    if (token.startsWith('eyJ')) {
      logger.info(`🔑 JWT token extracted from ${source}`);
      return token;
    }

    // DEPRECATED: Legacy format support
    if (token.startsWith('web3_token_')) {
      const wallet = token.slice(12);
      if (wallet.length >= 20 && wallet.startsWith('0x')) {
        logger.warn('⚠️ DEPRECATED: Legacy web3_token_ format detected.');
        return token;
      }
    }

    if (token.length > 50) {
      logger.info(`🔑 Token extracted from ${source}`);
      return token;
    }

    logger.warn('⚠️ Token invalid format or too short');
    return null;
  }

  /**
   * Internal helper to build SSE URL and validate it
   */
  private buildSSEUrl(options: SSEConnectionOptions): { sseUrl: string; token: string | null; platform: string | undefined } {
    const params = new URLSearchParams();
    if (typeof options.wallet_address === 'string' && options.wallet_address !== '') { params.append('wallet_address', options.wallet_address); }
    if (options.types !== undefined && options.types.length > 0) { params.append('types', options.types.join(',')); }
    if (options.priority !== undefined) { params.append('priority', options.priority); }

    const token = this.getTokenFromCookies();
    const platform = this.client.getPlatform();

    if (token !== null && token !== '') {
      params.append('token', token);
      logger.info(`🔑 SSE [${platform}]: Client-side token found and added to URL`);
    } else {
      logger.info(`ℹ️ SSE [${platform}]: No client-side token found. Relying on Proxy Middleware.`);
    }

    const queryString = params.toString();

    let baseURL: string;
    if (typeof window !== 'undefined') {
      const { getBackendUrl } = require('../utils/url-resolver') as { getBackendUrl: (p: string) => string };
      baseURL = getBackendUrl('client');
    } else {
      baseURL = this.client.getBaseURL();
    }

    const sseUrl = `${baseURL}${API_ROUTES.NOTIFICATIONS.STREAM}${queryString ? `?${queryString}` : ''}`;

    if (!baseURL || typeof baseURL !== 'string') {
      const error = `Invalid baseURL: "${baseURL}"`;
      logger.error('❌ SSE Connection Error', { error });
      throw new Error(error);
    }

    this.validateUrlFormat(sseUrl);

    return { sseUrl, token, platform };
  }

  /**
   * Validate URL format for SSE
   */
  private validateUrlFormat(sseUrl: string): void {
    try {
      const base = typeof window !== 'undefined' ? window.location.origin : 'http://localhost';
      const urlObj = new URL(sseUrl, base);

      logger.info('✅ SSE URL validated', {
        protocol: urlObj.protocol,
        host: urlObj.host,
        pathname: urlObj.pathname,
      });
    } catch (e) {
      const error = `Invalid SSE URL format: "${sseUrl}"`;
      logger.error('❌ SSE URL validation failed', { error, exception: e });
      throw new Error(error);
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
    const { createAdminApiClient } = require('../utils/api-client') as { createAdminApiClient: () => UnifiedApiClient };
    return new NotificationsAPIClient(createAdminApiClient());
  } else {
    const { createFrontendApiClient } = require('../utils/api-client') as { createFrontendApiClient: () => UnifiedApiClient };
    return new NotificationsAPIClient(createFrontendApiClient());
  }
}

// Type alias for backward compatibility with useApiClient
export type NotificationsApi = NotificationsAPIClient;