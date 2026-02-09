'use client';

import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Bell } from 'lucide-react';

interface NotificationSettings {
  enabled: boolean;
  analytics: boolean;
  security: boolean;
  system: boolean;
  permissions: boolean;
}

interface NotificationSettingsPanelProps {
  settings: NotificationSettings;
  onSettingsChange: (settings: NotificationSettings) => void;
  onTest: () => void;
}

export function NotificationSettingsPanel({
  settings,
  onSettingsChange,
  onTest
}: NotificationSettingsPanelProps) {
  return (
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
            onSettingsChange({ ...settings, enabled: checked })
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
                onSettingsChange({ ...settings, analytics: checked })
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
                onSettingsChange({ ...settings, security: checked })
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
                onSettingsChange({ ...settings, permissions: checked })
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
                onSettingsChange({ ...settings, system: checked })
              }
            />
          </div>

          <Button
            onClick={onTest}
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
  );
}
