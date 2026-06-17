'use client';

import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

interface NotificationSettings {
  enabled: boolean;
  analytics: boolean;
  security: boolean;
  system: boolean;
  permissions: boolean;
}

interface UseBrowserNotificationsStateProps {
  autoRequestPermission?: boolean;
  enabledByDefault?: boolean;
}

export function useBrowserNotificationsState({
  autoRequestPermission = false,
  enabledByDefault = false
}: UseBrowserNotificationsStateProps) {
  const [permission, setPermission] = useState<NotificationPermission>('default');
  const [isSupported, setIsSupported] = useState(false);
  const [settings, setSettings] = useState<NotificationSettings>({
    enabled: enabledByDefault,
    analytics: true,
    security: true,
    system: false,
    permissions: true,
  });

  // Check browser support
  useEffect(() => {
    if (typeof window !== 'undefined' && 'Notification' in window) {
      setIsSupported(true);
      setPermission(Notification.permission);

      const cookies = document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) {acc[key] = value;}
        return acc;
      }, {});

      const savedSettings = cookies.browser_notifications ?? localStorage.getItem('browser-notifications');
      if (savedSettings && savedSettings.length > 0) {
        try {
          const parsed = JSON.parse(savedSettings) as NotificationSettings;
          setSettings(prev => ({ ...prev, ...parsed }));
        } catch (_error) {
          // Parsing failed
        }
      }
    }
  }, []);

  // Save settings
  useEffect(() => {
    if (typeof window !== 'undefined') {
      document.cookie = `browser_notifications=${JSON.stringify(settings)}; path=/; max-age=31536000; SameSite=lax`;
    }
  }, [settings]);

  const requestPermission = useCallback(async () => {
    if (!isSupported) {return;}

    try {
      const result = await Notification.requestPermission();
      setPermission(result);

      if (result === 'granted') {
        toast.success('Browser notifications enabled');
        setSettings(prev => ({ ...prev, enabled: true }));
      } else {
        toast.error('Browser notifications denied');
        setSettings(prev => ({ ...prev, enabled: false }));
      }
    } catch (_error) {
      toast.error('Failed to enable notifications');
    }
  }, [isSupported]);

  // Auto-request
  useEffect(() => {
    if (autoRequestPermission && permission === 'default' && isSupported) {
      void requestPermission();
    }
  }, [autoRequestPermission, permission, isSupported, requestPermission]);

  return {
    permission,
    isSupported,
    settings,
    setSettings,
    requestPermission,
  };
}
