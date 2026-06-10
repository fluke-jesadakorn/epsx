'use client';

import { useCallback } from 'react';

interface NotificationData {
  url?: string;
  type?: string;
}

interface ShowNotificationContext {
  isSupported: boolean;
  permission: NotificationPermission;
  enabled: boolean;
}

export function useNotificationMethods(ctx: ShowNotificationContext) {
  const showNotification = useCallback((
    title: string,
    body: string,
    options: NotificationOptions & { tag?: string; requireInteraction?: boolean } = {}
  ) => {
    if (!ctx.isSupported || ctx.permission !== 'granted' || !ctx.enabled) {
      return;
    }

    try {
      const notification = new Notification(title, {
        body,
        icon: options.icon ?? '/favicon.ico',
        tag: options.tag ?? 'epsx-notification',
        requireInteraction: options.requireInteraction ?? false,
        ...options
      });

      if (options.requireInteraction !== true) {
        setTimeout(() => notification.close(), 5000);
      }

      notification.onclick = () => {
        window.focus();
        notification.close();

        const data = options.data as NotificationData | undefined;
        if (data?.url !== undefined && data.url.length > 0) {
          window.location.href = data.url;
        }
      };

      return notification;
    } catch (_error) {
      // Creation failed
    }
  }, [ctx.isSupported, ctx.permission, ctx.enabled]);

  return { showNotification };
}
