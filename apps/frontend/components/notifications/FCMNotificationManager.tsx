'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import { Separator } from '@/components/ui/separator';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { 
  fcmClient, 
  FCMTokenInfo, 
  FCMTestNotificationRequest,
  FCMPermissionStatus 
} from '@/lib/fcm-client';
import { Bell, BellOff, Send, Smartphone, Monitor, Tablet, AlertCircle, CheckCircle, XCircle } from 'lucide-react';

interface FCMNotificationManagerProps {
  className?: string;
}

interface SubscriptionStatus {
  isSupported: boolean;
  permission: FCMPermissionStatus;
  hasToken: boolean;
  isSubscribed: boolean;
}

export function FCMNotificationManager({ className }: FCMNotificationManagerProps) {
  const [status, setStatus] = useState<SubscriptionStatus>({
    isSupported: false,
    permission: 'unsupported',
    hasToken: false,
    isSubscribed: false
  });
  const [userTokens, setUserTokens] = useState<FCMTokenInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  
  // Test notification form
  const [testNotification, setTestNotification] = useState<FCMTestNotificationRequest>({
    title: 'EPSX Test Notification',
    body: 'This is a test notification from EPSX platform',
    icon: '/logo.png',
    url: '/notifications'
  });

  // Load initial status
  const loadStatus = useCallback(async () => {
    try {
      setLoading(true);
      const subscriptionStatus = await fcmClient.getSubscriptionStatus();
      setStatus(subscriptionStatus);
      
      if (subscriptionStatus.hasToken) {
        const tokens = await fcmClient.getUserTokens();
        setUserTokens(tokens);
      }
    } catch (error) {
      console.error('Error loading FCM status:', error);
      setError(error instanceof Error ? error.message : 'Failed to load FCM status');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  // Handle subscription toggle
  const handleSubscriptionToggle = async (subscribe: boolean) => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      if (subscribe) {
        const result = await fcmClient.subscribe();
        setSuccess('Successfully subscribed to push notifications!');
        console.log('FCM subscription result:', result);
      } else {
        const result = await fcmClient.unsubscribe();
        setSuccess(result.backendSuccess 
          ? 'Successfully unsubscribed from push notifications!' 
          : 'Unsubscribed locally, but backend sync may have failed'
        );
        console.log('FCM unsubscription result:', result);
      }
      
      // Reload status
      await loadStatus();
    } catch (error) {
      console.error('Error handling subscription:', error);
      setError(error instanceof Error ? error.message : 'Subscription operation failed');
    } finally {
      setLoading(false);
    }
  };

  // Handle test notification
  const handleSendTestNotification = async () => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      const result = await fcmClient.sendTestNotification(testNotification);
      setSuccess(result.message || 'Test notification sent successfully!');
    } catch (error) {
      console.error('Error sending test notification:', error);
      setError(error instanceof Error ? error.message : 'Failed to send test notification');
    } finally {
      setLoading(false);
    }
  };

  // Get permission status badge
  const getPermissionBadge = (permission: FCMPermissionStatus) => {
    switch (permission) {
      case 'granted':
        return <Badge variant="default" className="bg-green-100 text-green-800"><CheckCircle className="w-3 h-3 mr-1" />Granted</Badge>;
      case 'denied':
        return <Badge variant="destructive"><XCircle className="w-3 h-3 mr-1" />Denied</Badge>;
      case 'default':
        return <Badge variant="secondary"><AlertCircle className="w-3 h-3 mr-1" />Not Requested</Badge>;
      case 'unsupported':
        return <Badge variant="outline"><XCircle className="w-3 h-3 mr-1" />Unsupported</Badge>;
      default:
        return <Badge variant="outline">Unknown</Badge>;
    }
  };

  // Get device icon based on user agent
  const getDeviceIcon = (userAgent: string) => {
    const ua = userAgent.toLowerCase();
    if (ua.includes('mobile') || ua.includes('android') || ua.includes('iphone')) {
      return <Smartphone className="w-4 h-4" />;
    } else if (ua.includes('tablet') || ua.includes('ipad')) {
      return <Tablet className="w-4 h-4" />;
    } else {
      return <Monitor className="w-4 h-4" />;
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Status Overview Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bell className="w-5 h-5" />
            Push Notification Status
          </CardTitle>
          <CardDescription>
            Manage your push notification preferences and test FCM functionality
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Support Status */}
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Browser Support:</span>
            <Badge variant={status.isSupported ? "default" : "destructive"}>
              {status.isSupported ? 'Supported' : 'Not Supported'}
            </Badge>
          </div>

          {/* Permission Status */}
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Permission Status:</span>
            {getPermissionBadge(status.permission)}
          </div>

          {/* Token Status */}
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">FCM Token:</span>
            <Badge variant={status.hasToken ? "default" : "secondary"}>
              {status.hasToken ? 'Available' : 'Not Available'}
            </Badge>
          </div>

          {/* Subscription Status */}
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Subscription Status:</span>
            <Badge variant={status.isSubscribed ? "default" : "outline"}>
              {status.isSubscribed ? 'Subscribed' : 'Not Subscribed'}
            </Badge>
          </div>

          <Separator />

          {/* Subscription Toggle */}
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-base">Push Notifications</Label>
              <div className="text-sm text-muted-foreground">
                Receive notifications when the app is not active
              </div>
            </div>
            <Switch
              checked={status.isSubscribed}
              onCheckedChange={handleSubscriptionToggle}
              disabled={!status.isSupported || loading}
            />
          </div>
        </CardContent>
      </Card>

      {/* Error/Success Alerts */}
      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {success && (
        <Alert>
          <CheckCircle className="h-4 w-4" />
          <AlertDescription>{success}</AlertDescription>
        </Alert>
      )}

      {/* Active Tokens */}
      {userTokens.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Active Devices</CardTitle>
            <CardDescription>
              Devices currently subscribed to receive push notifications
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {userTokens.map((token, index) => (
                <div key={token.token.substring(0, 20)} className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center gap-3">
                    {getDeviceIcon(token.userAgent)}
                    <div>
                      <div className="text-sm font-medium">
                        {token.platform}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Registered: {token.createdAt.toLocaleDateString()}
                      </div>
                    </div>
                  </div>
                  <Badge variant={token.isActive ? "default" : "secondary"}>
                    {token.isActive ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Test Notification */}
      {status.isSubscribed && (
        <Card>
          <CardHeader>
            <CardTitle>Test Notification</CardTitle>
            <CardDescription>
              Send a test push notification to verify functionality
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="test-title">Title</Label>
                <Input
                  id="test-title"
                  value={testNotification.title}
                  onChange={(e) => setTestNotification(prev => ({
                    ...prev,
                    title: e.target.value
                  }))}
                  placeholder="Notification title"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="test-url">URL (optional)</Label>
                <Input
                  id="test-url"
                  value={testNotification.url}
                  onChange={(e) => setTestNotification(prev => ({
                    ...prev,
                    url: e.target.value
                  }))}
                  placeholder="/notifications"
                />
              </div>
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="test-body">Message</Label>
              <Textarea
                id="test-body"
                value={testNotification.body}
                onChange={(e) => setTestNotification(prev => ({
                  ...prev,
                  body: e.target.value
                }))}
                placeholder="Notification message"
                rows={3}
              />
            </div>

            <Button 
              onClick={handleSendTestNotification}
              disabled={loading || !testNotification.title || !testNotification.body}
              className="w-full"
            >
              <Send className="w-4 h-4 mr-2" />
              {loading ? 'Sending...' : 'Send Test Notification'}
            </Button>
          </CardContent>
        </Card>
      )}

      {/* Unsupported Browser Message */}
      {!status.isSupported && (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center space-y-2">
              <BellOff className="w-12 h-12 mx-auto text-muted-foreground" />
              <h3 className="text-lg font-semibold">Push Notifications Not Supported</h3>
              <p className="text-muted-foreground">
                Your browser or device does not support push notifications. 
                Please use a modern browser like Chrome, Firefox, or Safari.
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Refresh Button */}
      <div className="flex justify-center">
        <Button 
          variant="outline" 
          onClick={loadStatus}
          disabled={loading}
        >
          {loading ? 'Refreshing...' : 'Refresh Status'}
        </Button>
      </div>
    </div>
  );
}

export default FCMNotificationManager;