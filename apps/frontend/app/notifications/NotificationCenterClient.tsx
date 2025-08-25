'use client';

import React, { useState } from 'react';
import { Bell, Settings, Smartphone, Mail, Volume2, VolumeX } from 'lucide-react';
import { NotificationList } from '@/components/notifications';
import { useNotifications, useNotificationPreferences, usePushNotifications } from '@/context/notification-context';
import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { toast } from 'sonner';

export function NotificationCenterClient() {
  const { unreadCount, realtimeConnected, error } = useNotifications();
  const { preferences, updatePreferences } = useNotificationPreferences();
  const { pushEnabled, setPushEnabled, requestPermission, subscribeToPush } = usePushNotifications();
  const [isUpdatingPreferences, setIsUpdatingPreferences] = useState(false);
  const [activeTab, setActiveTab] = useState('notifications');

  const handlePreferenceChange = async (key: keyof typeof preferences, value: boolean) => {
    setIsUpdatingPreferences(true);
    try {
      await updatePreferences({ [key]: value });
      toast.success('Notification preferences updated');
    } catch (error) {
      console.error('Failed to update preferences:', error);
      toast.error('Failed to update preferences');
    } finally {
      setIsUpdatingPreferences(false);
    }
  };

  const handleEnablePushNotifications = async () => {
    try {
      const granted = await requestPermission();
      if (granted) {
        await subscribeToPush();
        setPushEnabled(true);
        toast.success('Push notifications enabled');
      } else {
        toast.error('Push notification permission denied');
      }
    } catch (error) {
      console.error('Failed to enable push notifications:', error);
      toast.error('Failed to enable push notifications');
    }
  };

  return (
    <div className="container max-w-4xl mx-auto py-8 px-4">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-3">
          <div className="relative">
            <Bell className="h-8 w-8 text-primary" />
            {!realtimeConnected && (
              <div className="absolute -top-1 -right-1 h-3 w-3 bg-yellow-500 rounded-full animate-pulse" />
            )}
          </div>
          <div>
            <h1 className="text-3xl font-bold">Notifications</h1>
            <p className="text-muted-foreground">
              Manage your notifications and preferences
            </p>
          </div>
        </div>

        {unreadCount > 0 && (
          <Badge variant="secondary" className="text-sm">
            {unreadCount} unread
          </Badge>
        )}
      </div>

      {/* Connection Status */}
      {!realtimeConnected && (
        <Card className="mb-6 border-yellow-200 bg-yellow-50">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-yellow-700">
              <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse" />
              <span className="text-sm font-medium">
                Reconnecting to notification service...
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Error State */}
      {error && (
        <Card className="mb-6 border-red-200 bg-red-50">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-red-700">
              <span className="text-sm font-medium">
                Failed to load notifications: {error}
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="notifications">
            Notifications {unreadCount > 0 && `(${unreadCount})`}
          </TabsTrigger>
          <TabsTrigger value="settings">
            <Settings className="h-4 w-4 mr-2" />
            Settings
          </TabsTrigger>
        </TabsList>

        <TabsContent value="notifications" className="mt-6">
          <NotificationList 
            showFilters={true}
            showTabs={true}
            showBulkActions={true}
            itemsPerPage={20}
          />
        </TabsContent>

        <TabsContent value="settings" className="mt-6 space-y-6">
          {/* Notification Channels */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Volume2 className="h-5 w-5" />
                Notification Channels
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Choose how you want to receive notifications
              </p>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* In-App Notifications */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Bell className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <Label className="text-base font-medium">In-App Notifications</Label>
                    <p className="text-sm text-muted-foreground">
                      Show notifications while using the app
                    </p>
                  </div>
                </div>
                <Switch
                  checked={preferences.inApp}
                  onCheckedChange={(value) => handlePreferenceChange('inApp', value)}
                  disabled={isUpdatingPreferences}
                />
              </div>

              <Separator />

              {/* Push Notifications */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Smartphone className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <Label className="text-base font-medium">Push Notifications</Label>
                    <p className="text-sm text-muted-foreground">
                      Receive notifications on your device even when the app is closed
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {!('Notification' in window) ? (
                    <Badge variant="secondary">Not supported</Badge>
                  ) : !pushEnabled ? (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleEnablePushNotifications}
                    >
                      Enable
                    </Button>
                  ) : (
                    <Switch
                      checked={pushEnabled}
                      onCheckedChange={setPushEnabled}
                    />
                  )}
                </div>
              </div>

              <Separator />

              {/* Email Notifications */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Mail className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <Label className="text-base font-medium">Email Notifications</Label>
                    <p className="text-sm text-muted-foreground">
                      Receive important notifications via email
                    </p>
                  </div>
                </div>
                <Switch
                  checked={preferences.email}
                  onCheckedChange={(value) => handlePreferenceChange('email', value)}
                  disabled={isUpdatingPreferences}
                />
              </div>
            </CardContent>
          </Card>

          {/* Notification Types */}
          <Card>
            <CardHeader>
              <CardTitle>Notification Types</CardTitle>
              <p className="text-sm text-muted-foreground">
                Choose which types of notifications you want to receive
              </p>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Trading Alerts */}
              <div className="flex items-center justify-between">
                <div>
                  <Label className="text-base font-medium">Trading Alerts</Label>
                  <p className="text-sm text-muted-foreground">
                    Price alerts, market movements, and trading opportunities
                  </p>
                </div>
                <Switch
                  checked={preferences.tradingAlerts}
                  onCheckedChange={(value) => handlePreferenceChange('tradingAlerts', value)}
                  disabled={isUpdatingPreferences}
                />
              </div>

              <Separator />

              {/* System Updates */}
              <div className="flex items-center justify-between">
                <div>
                  <Label className="text-base font-medium">System Updates</Label>
                  <p className="text-sm text-muted-foreground">
                    Platform updates, maintenance, and important announcements
                  </p>
                </div>
                <Switch
                  checked={preferences.systemUpdates}
                  onCheckedChange={(value) => handlePreferenceChange('systemUpdates', value)}
                  disabled={isUpdatingPreferences}
                />
              </div>
            </CardContent>
          </Card>

          {/* Advanced Settings */}
          <Card>
            <CardHeader>
              <CardTitle>Advanced Settings</CardTitle>
              <p className="text-sm text-muted-foreground">
                Configure advanced notification behavior
              </p>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="text-sm text-muted-foreground">
                <p>• Notifications are delivered in real-time when connected</p>
                <p>• Push notifications work even when the app is closed</p>
                <p>• Email notifications are sent for critical alerts only</p>
                <p>• You can disable notifications for specific categories</p>
              </div>
              
              <div className="pt-4 border-t">
                <div className="flex items-center justify-between text-sm">
                  <span>Connection Status:</span>
                  <div className="flex items-center gap-2">
                    {realtimeConnected ? (
                      <>
                        <div className="w-2 h-2 bg-green-500 rounded-full" />
                        <span className="text-green-600">Connected</span>
                      </>
                    ) : (
                      <>
                        <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse" />
                        <span className="text-yellow-600">Reconnecting...</span>
                      </>
                    )}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default NotificationCenterClient;