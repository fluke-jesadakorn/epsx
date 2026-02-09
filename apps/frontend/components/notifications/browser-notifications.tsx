'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertTriangle, Bell, BellOff } from 'lucide-react';
import { useCallback } from 'react';
import { toast } from 'sonner';
import { useBrowserNotificationsState } from './hooks/use-browser-notifications-state';
import { useNotificationApi } from './hooks/use-notification-api';
import { useNotificationMethods } from './hooks/use-notification-methods';
import { NotificationPermissionBadge } from './ui/notification-permission-badge';
import { NotificationSettingsPanel } from './ui/notification-settings-panel';

interface EpsxNotificationsAPI {
  analytics: (title: string, body: string, url?: string) => void;
  security: (title: string, body: string, url?: string) => void;
  system: (title: string, body: string, url?: string) => void;
  permissions: (title: string, body: string, url?: string) => void;
  show: (title: string, body: string, options?: NotificationOptions) => void;
}

declare global {
  interface Window {
    epsxNotifications?: EpsxNotificationsAPI;
  }
}

interface BrowserNotificationsProps {
  className?: string;
  autoRequestPermission?: boolean;
  enabledByDefault?: boolean;
}

export function BrowserNotifications({
  className = '',
  autoRequestPermission = false,
  enabledByDefault = false
}: BrowserNotificationsProps) {
  const {
    permission,
    isSupported,
    settings,
    setSettings,
    requestPermission
  } = useBrowserNotificationsState({ autoRequestPermission, enabledByDefault });

  const { showNotification } = useNotificationMethods({
    isSupported,
    permission,
    enabled: settings.enabled
  });

  useNotificationApi({ settings, showNotification });

  const testNotification = useCallback(() => {
    showNotification(
      'Test Notification',
      'This is a test browser notification from EPSX',
      { tag: 'test', requireInteraction: false }
    );
    toast.success('Test notification sent');
  }, [showNotification]);

  if (!isSupported) {
    return (
      <Card className={className}>
        <CardContent className="p-4">
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Browser notifications are not supported in this browser.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Bell className="h-5 w-5 text-blue-500" />
            Browser Notifications
          </div>
          <NotificationPermissionBadge permission={permission} />
        </CardTitle>
      </CardHeader>

      <CardContent className="space-y-4">
        {permission === 'default' && (
          <>
            <Alert>
              <Bell className="h-4 w-4" />
              <AlertDescription>
                Enable browser notifications to receive important alerts about your analytics activity,
                security events, and permission changes.
              </AlertDescription>
            </Alert>
            <Button onClick={() => void requestPermission()} className="w-full">
              <Bell className="h-4 w-4 mr-2" />
              Enable Browser Notifications
            </Button>
          </>
        )}

        {permission === 'denied' && (
          <Alert variant="destructive">
            <BellOff className="h-4 w-4" />
            <AlertDescription>
              Browser notifications are blocked. To enable them, click the 🔒 icon in your browser's
              address bar and allow notifications for this site.
            </AlertDescription>
          </Alert>
        )}

        {permission === 'granted' && (
          <NotificationSettingsPanel
            settings={settings}
            onSettingsChange={setSettings}
            onTest={testNotification}
          />
        )}

        <div className="text-xs text-gray-600 dark:text-gray-400 space-y-1">
          <p><strong>📱 Usage:</strong> Browser notifications appear outside the website as native OS notifications.</p>
          <p><strong>🔕 Privacy:</strong> Notifications are processed locally in your browser only.</p>
          <p><strong>⚙️ Control:</strong> You can disable or customize notification types at any time.</p>
        </div>
      </CardContent>
    </Card>
  );
}

interface NotificationParams {
  type: 'analytics' | 'security' | 'system' | 'permissions';
  title: string;
  body: string;
  url?: string;
}

export function useBrowserNotifications() {
  const showNotification = useCallback((params: NotificationParams) => {
    if (typeof window !== 'undefined' && window.epsxNotifications !== undefined) {
      window.epsxNotifications[params.type](params.title, params.body, params.url);
    }
  }, []);

  return { showNotification };
}
