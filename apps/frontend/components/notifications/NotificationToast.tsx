'use client';

import React, { useEffect } from 'react';
import { X, Bell, TrendingUp, AlertTriangle, CreditCard, Settings } from 'lucide-react';
import { toast } from 'sonner';
import type { Notification } from '@/lib/state/types';
import { Button } from '@/components/ui/button';
import { useNotifications } from '@/context/notification-context';

interface NotificationToastProps {
  notification: Notification;
  onDismiss?: () => void;
  onAction?: () => void;
}

const notificationIcons = {
  trading: TrendingUp,
  system: Settings,
  account: CreditCard,
  price_alert: AlertTriangle,
};

export function NotificationToast({ 
  notification, 
  onDismiss, 
  onAction 
}: NotificationToastProps) {
  const { markRead } = useNotifications();
  
  const Icon = notificationIcons[notification.type] || Bell;

  const handleAction = () => {
    if (onAction) {
      onAction();
    } else if (notification.actionUrl) {
      window.open(notification.actionUrl, '_blank');
    }
    
    // Mark as read when action is taken
    if (!notification.read) {
      markRead(notification.id);
    }
  };

  const handleDismiss = () => {
    if (onDismiss) {
      onDismiss();
    }
    
    // Mark as read when dismissed
    if (!notification.read) {
      markRead(notification.id);
    }
  };

  return (
    <div className="flex items-start gap-3 p-4 bg-background border rounded-lg shadow-lg max-w-sm">
      {/* Icon */}
      <div className="flex-shrink-0">
        <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
          <Icon className="h-4 w-4 text-primary" />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <h3 className="font-medium text-sm text-foreground line-clamp-1">
          {notification.title}
        </h3>
        <p className="text-sm text-muted-foreground line-clamp-2 mt-1">
          {notification.message}
        </p>
        
        {/* Actions */}
        {notification.actionUrl && (
          <div className="mt-3">
            <Button
              variant="outline"
              size="sm"
              onClick={handleAction}
              className="h-8"
            >
              View Details
            </Button>
          </div>
        )}
      </div>

      {/* Close Button */}
      <Button
        variant="ghost"
        size="sm"
        onClick={handleDismiss}
        className="h-8 w-8 p-0 flex-shrink-0"
      >
        <X className="h-4 w-4" />
      </Button>
    </div>
  );
}

// Hook to show notification toasts
export function useNotificationToasts() {
  const { notifications } = useNotifications();
  const [lastNotificationId, setLastNotificationId] = React.useState<string | null>(null);

  useEffect(() => {
    if (notifications.length === 0) return;

    // Get the most recent notification
    const latestNotification = notifications
      .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())[0];

    // Only show toast for new notifications
    if (latestNotification.id !== lastNotificationId && !latestNotification.read) {
      setLastNotificationId(latestNotification.id);

      // Show toast with different styles based on priority/type
      const toastId = toast.custom((t) => (
        <NotificationToast 
          notification={latestNotification}
          onDismiss={() => toast.dismiss(t)}
        />
      ), {
        duration: latestNotification.type === 'trading' ? 8000 : 5000, // Trading notifications stay longer
        position: 'top-right',
      });
    }
  }, [notifications, lastNotificationId]);
}

// Component to automatically show notification toasts
export function NotificationToastProvider({ children }: { children: React.ReactNode }) {
  useNotificationToasts();
  return <>{children}</>;
}

export default NotificationToast;