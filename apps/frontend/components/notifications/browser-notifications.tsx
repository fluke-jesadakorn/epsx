'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { AlertTriangle, Bell, BellOff, CheckCircle, X } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

interface BrowserNotificationsProps {
  className?: string;
  autoRequestPermission?: boolean;
  enabledByDefault?: boolean;
}

interface NotificationSettings {
  enabled: boolean;
  analytics: boolean;
  security: boolean;
  system: boolean;
  permissions: boolean;
}

export function BrowserNotifications({
  className = '',
  autoRequestPermission = false,
  enabledByDefault = false
}: BrowserNotificationsProps) {
  const [permission, setPermission] = useState<NotificationPermission>('default');
  const [isSupported, setIsSupported] = useState(false);
  const [settings, setSettings] = useState<NotificationSettings>({
    enabled: enabledByDefault,
    analytics: true,
    security: true,
    system: false,
    permissions: true,
  });

  // Check browser support and permission status
  useEffect(() => {
    if (typeof window !== 'undefined' && 'Notification' in window) {
      setIsSupported(true);
      setPermission(Notification.permission);

      // Load settings from cookies first, fallback to localStorage for migration
      const cookies = document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) {acc[key] = value;}
        return acc;
      }, {});

      const savedSettings = cookies.browser_notifications || localStorage.getItem('browser-notifications');
      if (savedSettings) {
        try {
          const parsed = JSON.parse(savedSettings);
          setSettings(prev => ({ ...prev, ...parsed }));
        } catch (error) {
          console.error('Failed to parse notification settings:', error);
        }
      }
    }
  }, []);

  // Save settings to cookies
  useEffect(() => {
    if (typeof window !== 'undefined') {
      // Store notification settings in cookie
      document.cookie = `browser_notifications=${JSON.stringify(settings)}; path=/; max-age=31536000; SameSite=lax`;
    }
  }, [settings]);

  // Auto-request permission if enabled
  useEffect(() => {
    if (autoRequestPermission && permission === 'default' && isSupported) {
      requestPermission();
    }
  }, [autoRequestPermission, permission, isSupported]);

  const requestPermission = async () => {
    if (!isSupported) {return;}

    try {
      const result = await Notification.requestPermission();
      setPermission(result);

      if (result === 'granted') {
        toast.success('Browser notifications enabled');
        setSettings(prev => ({ ...prev, enabled: true }));

        // Show welcome notification
        showNotification(
          'EPSX Notifications Enabled',
          'You\'ll receive important alerts about your account',
          {
            icon: '/favicon.ico',
            tag: 'welcome',
            requireInteraction: false
          }
        );
      } else {
        toast.error('Browser notifications denied');
        setSettings(prev => ({ ...prev, enabled: false }));
      }
    } catch (error) {
      console.error('Failed to request notification permission:', error);
      toast.error('Failed to enable notifications');
    }
  };

  const showNotification = useCallback((
    title: string,
    body: string,
    options: NotificationOptions & { tag?: string; requireInteraction?: boolean } = {}
  ) => {
    if (!isSupported || permission !== 'granted' || !settings.enabled) {
      return;
    }

    try {
      const notification = new Notification(title, {
        body,
        icon: options.icon || '/favicon.ico',
        tag: options.tag || 'epsx-notification',
        requireInteraction: options.requireInteraction || false,
        ...options
      });

      // Auto-close after 5 seconds unless requireInteraction is true
      if (!options.requireInteraction) {
        setTimeout(() => notification.close(), 5000);
      }

      // Handle click events
      notification.onclick = () => {
        window.focus();
        notification.close();

        // Navigate to relevant page if URL provided
        if (options.data?.url) {
          window.location.href = options.data.url;
        }
      };

      return notification;
    } catch (error) {
      console.error('Failed to show notification:', error);
    }
  }, [isSupported, permission, settings.enabled]);

  // Public method to show notifications from other components
  const showAnalyticsAlert = useCallback((title: string, body: string, url?: string) => {
    if (!settings.analytics) {return;}
    showNotification(title, body, {
      tag: 'analytics',
      requireInteraction: true,
      data: { url, type: 'analytics' }
    });
  }, [showNotification, settings.analytics]);

  const showSecurityAlert = useCallback((title: string, body: string, url?: string) => {
    if (!settings.security) {return;}
    showNotification(title, body, {
      tag: 'security',
      requireInteraction: true,
      data: { url, type: 'security' }
    });
  }, [showNotification, settings.security]);

  const showSystemAlert = useCallback((title: string, body: string, url?: string) => {
    if (!settings.system) {return;}
    showNotification(title, body, {
      tag: 'system',
      requireInteraction: false,
      data: { url, type: 'system' }
    });
  }, [showNotification, settings.system]);

  const showPermissionAlert = useCallback((title: string, body: string, url?: string) => {
    if (!settings.permissions) {return;}
    showNotification(title, body, {
      tag: 'permissions',
      requireInteraction: true,
      data: { url, type: 'permissions' }
    });
  }, [showNotification, settings.permissions]);

  // Expose notification methods globally for other components
  useEffect(() => {
    if (typeof window !== 'undefined') {
      (window as any).epsxNotifications = {
        analytics: showAnalyticsAlert,
        security: showSecurityAlert,
        system: showSystemAlert,
        permissions: showPermissionAlert,
        show: showNotification,
      };
    }
  }, [showAnalyticsAlert, showSecurityAlert, showSystemAlert, showPermissionAlert, showNotification]);

  const getPermissionBadge = () => {
    switch (permission) {
      case 'granted':
        return (
          <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            <CheckCircle className="h-3 w-3 mr-1" />
            Enabled
          </Badge>
        );
      case 'denied':
        return (
          <Badge variant="destructive">
            <X className="h-3 w-3 mr-1" />
            Blocked
          </Badge>
        );
      default:
        return (
          <Badge variant="secondary">
            <AlertTriangle className="h-3 w-3 mr-1" />
            Not Set
          </Badge>
        );
    }
  };

  const testNotification = () => {
    showNotification(
      'Test Notification',
      'This is a test browser notification from EPSX',
      { tag: 'test', requireInteraction: false }
    );
    toast.success('Test notification sent');
  };

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
          {getPermissionBadge()}
        </CardTitle>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Permission Status */}
        {permission === 'default' && (
          <Alert>
            <Bell className="h-4 w-4" />
            <AlertDescription>
              Enable browser notifications to receive important alerts about your analytics activity,
              security events, and permission changes.
            </AlertDescription>
          </Alert>
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

        {/* Main Enable/Disable Button */}
        {permission === 'default' && (
          <Button
            onClick={requestPermission}
            className="w-full"
          >
            <Bell className="h-4 w-4 mr-2" />
            Enable Browser Notifications
          </Button>
        )}

        {/* Notification Settings */}
        {permission === 'granted' && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label htmlFor="notifications-enabled" className="flex items-center gap-2">
                <Bell className="h-4 w-4" />
                Enable Notifications
              </Label>
              <Switch
                id="notifications-enabled"
                checked={settings.enabled}
                onCheckedChange={(checked) =>
                  setSettings(prev => ({ ...prev, enabled: checked }))
                }
              />
            </div>

            {settings.enabled && (
              <div className="space-y-3 pl-6 border-l-2 border-gray-200 dark:border-gray-700">
                <div className="flex items-center justify-between">
                  <Label htmlFor="analytics-notifications" className="text-sm">
                    💹 Analytics & Portfolio Alerts
                  </Label>
                  <Switch
                    id="analytics-notifications"
                    checked={settings.analytics}
                    onCheckedChange={(checked) =>
                      setSettings(prev => ({ ...prev, analytics: checked }))
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label htmlFor="security-notifications" className="text-sm">
                    🔒 Security & Login Alerts
                  </Label>
                  <Switch
                    id="security-notifications"
                    checked={settings.security}
                    onCheckedChange={(checked) =>
                      setSettings(prev => ({ ...prev, security: checked }))
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label htmlFor="permissions-notifications" className="text-sm">
                    🛡️ Permission Changes
                  </Label>
                  <Switch
                    id="permissions-notifications"
                    checked={settings.permissions}
                    onCheckedChange={(checked) =>
                      setSettings(prev => ({ ...prev, permissions: checked }))
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label htmlFor="system-notifications" className="text-sm">
                    🔧 System Updates
                  </Label>
                  <Switch
                    id="system-notifications"
                    checked={settings.system}
                    onCheckedChange={(checked) =>
                      setSettings(prev => ({ ...prev, system: checked }))
                    }
                  />
                </div>

                <Button
                  onClick={testNotification}
                  variant="outline"
                  size="sm"
                  className="w-full mt-4"
                >
                  <Bell className="h-4 w-4 mr-2" />
                  Test Notification
                </Button>
              </div>
            )}
          </div>
        )}

        {/* Usage Instructions */}
        <div className="text-xs text-gray-600 dark:text-gray-400 space-y-1">
          <p><strong>📱 Usage:</strong> Browser notifications appear outside the website as native OS notifications.</p>
          <p><strong>🔕 Privacy:</strong> Notifications are processed locally in your browser only.</p>
          <p><strong>⚙️ Control:</strong> You can disable or customize notification types at any time.</p>
        </div>
      </CardContent>
    </Card>
  );
}

// Export notification helper hook for other components
export function useBrowserNotifications() {
  const showNotification = useCallback((type: 'analytics' | 'security' | 'system' | 'permissions', title: string, body: string, url?: string) => {
    if (typeof window !== 'undefined' && (window as any).epsxNotifications) {
      (window as any).epsxNotifications[type](title, body, url);
    }
  }, []);

  return { showNotification };
}