'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { fcmClient, FCMTestNotificationRequest } from '@/lib/fcm-client';
import { Bell, Smartphone, CheckCircle, AlertCircle, Send, Settings } from 'lucide-react';

export default function UserFCMTestPage() {
  const [status, setStatus] = useState({
    isSupported: false,
    permission: 'unsupported' as any,
    hasToken: false,
    isSubscribed: false
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadStatus();
  }, []);

  const loadStatus = async () => {
    try {
      setLoading(true);
      const subscriptionStatus = await fcmClient.getSubscriptionStatus();
      setStatus(subscriptionStatus);
    } catch (error) {
      console.error('Error loading FCM status:', error);
      setError(error instanceof Error ? error.message : 'Failed to load FCM status');
    } finally {
      setLoading(false);
    }
  };

  const handleSubscribe = async () => {
    try {
      setLoading(true);
      setError(null);
      await fcmClient.subscribe();
      setSuccess('Successfully subscribed to push notifications!');
      await loadStatus();
    } catch (error) {
      console.error('Error subscribing:', error);
      setError(error instanceof Error ? error.message : 'Failed to subscribe');
    } finally {
      setLoading(false);
    }
  };

  const handleUnsubscribe = async () => {
    try {
      setLoading(true);
      setError(null);
      await fcmClient.unsubscribe();
      setSuccess('Successfully unsubscribed from push notifications!');
      await loadStatus();
    } catch (error) {
      console.error('Error unsubscribing:', error);
      setError(error instanceof Error ? error.message : 'Failed to unsubscribe');
    } finally {
      setLoading(false);
    }
  };

  const sendTestNotifications = async () => {
    const testNotifications: FCMTestNotificationRequest[] = [
      {
        title: '📈 Trading Alert',
        body: 'AAPL stock has moved +5.2% in the last hour',
        data: { category: 'trading', symbol: 'AAPL' }
      },
      {
        title: '🔐 Security Alert',
        body: 'New login detected from Chrome on macOS',
        data: { category: 'security', type: 'login' }
      },
      {
        title: '👤 Account Update',
        body: 'Your subscription has been renewed successfully',
        data: { category: 'account', type: 'subscription' }
      },
      {
        title: '⚙️ System Notification',
        body: 'Scheduled maintenance will begin in 30 minutes',
        data: { category: 'system', type: 'maintenance' }
      },
      {
        title: '📢 New Feature',
        body: 'Try our new portfolio analytics dashboard!',
        data: { category: 'marketing', type: 'feature' }
      }
    ];

    for (let i = 0; i < testNotifications.length; i++) {
      try {
        setLoading(true);
        setError(null);
        
        await fcmClient.sendTestNotification(testNotifications[i]);
        setSuccess(`Sent test notification ${i + 1}/${testNotifications.length}: ${testNotifications[i].title}`);
        
        // Wait 2 seconds between notifications
        if (i < testNotifications.length - 1) {
          await new Promise(resolve => setTimeout(resolve, 2000));
        }
      } catch (error) {
        console.error('Error sending test notification:', error);
        setError(`Failed to send notification: ${testNotifications[i].title}`);
        break;
      }
    }
    
    setLoading(false);
    if (!error) {
      setSuccess('All test notifications sent successfully!');
    }
  };

  const getStatusBadge = (isGood: boolean, text: string) => (
    <Badge variant={isGood ? "default" : "destructive"} className="flex items-center gap-1">
      {isGood ? <CheckCircle className="w-3 h-3" /> : <AlertCircle className="w-3 h-3" />}
      {text}
    </Badge>
  );

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="text-center">
        <h1 className="text-3xl font-bold mb-2">🔔 User FCM Test Dashboard</h1>
        <p className="text-muted-foreground">
          Test Firebase Cloud Messaging functionality for regular users
        </p>
      </div>

      {/* Status Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Smartphone className="h-5 w-5" />
            FCM Status Overview
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center space-y-2">
              <div className="text-sm font-medium">Browser Support</div>
              {getStatusBadge(status.isSupported, status.isSupported ? 'Supported' : 'Not Supported')}
            </div>
            <div className="text-center space-y-2">
              <div className="text-sm font-medium">Permission</div>
              {getStatusBadge(status.permission === 'granted', status.permission)}
            </div>
            <div className="text-center space-y-2">
              <div className="text-sm font-medium">FCM Token</div>
              {getStatusBadge(status.hasToken, status.hasToken ? 'Available' : 'Missing')}
            </div>
            <div className="text-center space-y-2">
              <div className="text-sm font-medium">Subscription</div>
              {getStatusBadge(status.isSubscribed, status.isSubscribed ? 'Active' : 'Inactive')}
            </div>
          </div>

          <div className="flex justify-center">
            <Button variant="outline" onClick={loadStatus} disabled={loading}>
              {loading ? 'Refreshing...' : 'Refresh Status'}
            </Button>
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

      {/* Subscription Management */}
      <Card>
        <CardHeader>
          <CardTitle>Subscription Management</CardTitle>
          <CardDescription>
            Subscribe or unsubscribe from push notifications
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-4 justify-center">
            <Button
              onClick={handleSubscribe}
              disabled={!status.isSupported || status.isSubscribed || loading}
              className="flex items-center gap-2"
            >
              <Bell className="h-4 w-4" />
              Subscribe to FCM
            </Button>
            
            <Button
              onClick={handleUnsubscribe}
              disabled={!status.isSupported || !status.isSubscribed || loading}
              variant="outline"
              className="flex items-center gap-2"
            >
              <Bell className="h-4 w-4" />
              Unsubscribe from FCM
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Test Notifications */}
      {status.isSubscribed && (
        <Card>
          <CardHeader>
            <CardTitle>Test Notification Categories</CardTitle>
            <CardDescription>
              Send test notifications for different categories to verify your preferences
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-2">
              <div className="p-3 border rounded-lg">
                <div className="font-medium">📈 Trading Alerts</div>
                <div className="text-sm text-muted-foreground">Price movements, portfolio updates</div>
              </div>
              <div className="p-3 border rounded-lg">
                <div className="font-medium">🔐 Security Alerts</div>
                <div className="text-sm text-muted-foreground">Login attempts, security warnings</div>
              </div>
              <div className="p-3 border rounded-lg">
                <div className="font-medium">👤 Account Updates</div>
                <div className="text-sm text-muted-foreground">Profile changes, subscription updates</div>
              </div>
              <div className="p-3 border rounded-lg">
                <div className="font-medium">⚙️ System Notifications</div>
                <div className="text-sm text-muted-foreground">Maintenance, platform updates</div>
              </div>
              <div className="p-3 border rounded-lg">
                <div className="font-medium">📢 Marketing & Promotions</div>
                <div className="text-sm text-muted-foreground">New features, special offers</div>
              </div>
            </div>
            
            <div className="flex justify-center">
              <Button
                onClick={sendTestNotifications}
                disabled={loading}
                className="flex items-center gap-2"
              >
                <Send className="h-4 w-4" />
                {loading ? 'Sending Test Notifications...' : 'Send All Category Tests'}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Settings Link */}
      <Card>
        <CardContent className="pt-6 text-center">
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">
              Configure your notification preferences in settings
            </p>
            <Button variant="outline" asChild>
              <a href="/settings" className="flex items-center gap-2">
                <Settings className="h-4 w-4" />
                Go to Settings
              </a>
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Unsupported Browser Message */}
      {!status.isSupported && (
        <Card>
          <CardContent className="pt-6 text-center">
            <div className="space-y-2">
              <AlertCircle className="w-12 h-12 mx-auto text-muted-foreground" />
              <h3 className="text-lg font-semibold">Push Notifications Not Supported</h3>
              <p className="text-muted-foreground">
                Your browser or device does not support push notifications. 
                Please use a modern browser like Chrome, Firefox, or Safari.
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}