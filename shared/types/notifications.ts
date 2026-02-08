// Consolidated Notification Domain Types

export type NotificationType =
    | 'system'
    | 'admin'
    | 'data'
    | 'feature'
    | 'security'
    | 'analytics'
    | 'account'
    | 'price_alert'
    | 'info'
    | 'success'
    | 'warning'
    | 'error'
    | 'marketing';

export type NotificationPriority = 'low' | 'normal' | 'high' | 'urgent' | 'critical' | 'medium';

export type NotificationSender = 'system' | 'admin' | 'automated';

export interface Notification {
    id: string;
    user_id?: string;
    title: string;
    message: string;
    body?: string; // Compatibility with older types
    type: NotificationType;
    priority: NotificationPriority;
    sender?: NotificationSender;
    imageUrl?: string;
    actionUrl?: string;
    customData?: Record<string, unknown>;
    metadata?: Record<string, unknown>;
    data?: Record<string, unknown>;
    createdAt: string;
    created_at?: string;
    updatedAt?: string;
    updated_at?: string;
    readAt?: string;
    read_at?: string;
    clickedAt?: string;
    deliveredAt?: string;
    expiresAt?: string;
    expires_at?: string;
    read?: boolean;
}

export interface NotificationStats {
    totalSent?: number;
    delivered?: number;
    failed?: number;
    pending?: number;
    successRate?: number;
    unreadCount: number;
    total?: number;
    unread?: number;
}

export interface NotificationListParams {
    page?: number;
    limit?: number;
    type?: string;
    priority?: string;
    read?: boolean;
    userId?: string;
    startDate?: string;
    endDate?: string;
}

export interface NotificationWSMessage {
    type: 'notification';
    timestamp: string;
    data: Notification;
    channel?: string;
}

export interface NotificationCreateRequest {
    title: string;
    message: string;
    type: NotificationType;
    priority: NotificationPriority;
    userId?: string;
    userIds?: string[];
    actionUrl?: string;
    metadata?: Record<string, unknown>;
}

export interface NotificationUpdateRequest {
    title?: string;
    message?: string;
    type?: NotificationType;
    priority?: NotificationPriority;
    read?: boolean;
    actionUrl?: string;
    metadata?: Record<string, unknown>;
}

export interface BroadcastNotificationRequest {
    title: string;
    message: string;
    type: string;
    priority: string;
    userIds?: string[];
    allUsers?: boolean;
    metadata?: Record<string, unknown>;
}

export interface BroadcastNotificationResponse {
    notificationIds: string[];
    userCount: number;
    message: string;
}
