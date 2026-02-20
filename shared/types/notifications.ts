// Notification Domain Types - aligned with backend (sse_handlers.rs) and shared/components/notifications/types.ts

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

export type NotificationPriority = 'low' | 'normal' | 'high' | 'critical' | 'urgent';

export type NotificationSender = 'system' | 'admin' | 'automated';

export interface Notification {
    id: string;
    title: string;
    message: string;
    type: NotificationType;
    priority: NotificationPriority;
    timestamp: string;
    expires_at?: string;
    read_at?: string;
    clicked_at?: string;
    delivered_at?: string;
    action_url?: string;
    image_url?: string;
    wallet_address?: string;
    data?: Record<string, unknown>;
    read: boolean;
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
