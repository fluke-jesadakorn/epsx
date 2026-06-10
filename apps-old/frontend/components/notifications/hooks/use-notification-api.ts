'use client';

import { useCallback, useEffect } from 'react';

interface NotificationSettings {
  analytics: boolean;
  security: boolean;
  system: boolean;
  permissions: boolean;
}

interface UseNotificationApiContext {
  settings: NotificationSettings;
  showNotification: (title: string, body: string, options?: NotificationOptions) => void;
}

export function useNotificationApi(ctx: UseNotificationApiContext) {
  const showAnalyticsAlert = useCallback((title: string, body: string, url?: string) => {
    if (!ctx.settings.analytics) {return;}
    ctx.showNotification(title, body, {
      tag: 'analytics',
      requireInteraction: true,
      data: { url, type: 'analytics' }
    });
  }, [ctx]);

  const showSecurityAlert = useCallback((title: string, body: string, url?: string) => {
    if (!ctx.settings.security) {return;}
    ctx.showNotification(title, body, {
      tag: 'security',
      requireInteraction: true,
      data: { url, type: 'security' }
    });
  }, [ctx]);

  const showSystemAlert = useCallback((title: string, body: string, url?: string) => {
    if (!ctx.settings.system) {return;}
    ctx.showNotification(title, body, {
      tag: 'system',
      requireInteraction: false,
      data: { url, type: 'system' }
    });
  }, [ctx]);

  const showPermissionAlert = useCallback((title: string, body: string, url?: string) => {
    if (!ctx.settings.permissions) {return;}
    ctx.showNotification(title, body, {
      tag: 'permissions',
      requireInteraction: true,
      data: { url, type: 'permissions' }
    });
  }, [ctx]);

  // Expose globally
  useEffect(() => {
    if (typeof window !== 'undefined') {
      window.epsxNotifications = {
        analytics: showAnalyticsAlert,
        security: showSecurityAlert,
        system: showSystemAlert,
        permissions: showPermissionAlert,
        show: ctx.showNotification,
      };
    }
  }, [showAnalyticsAlert, showSecurityAlert, showSystemAlert, showPermissionAlert, ctx.showNotification]);

  return {
    showAnalyticsAlert,
    showSecurityAlert,
    showSystemAlert,
    showPermissionAlert,
  };
}
