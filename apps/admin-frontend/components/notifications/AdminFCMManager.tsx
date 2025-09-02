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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  adminFCMClient, 
  AdminFCMTokenInfo, 
  AdminFCMTestNotificationRequest,
  AdminFCMPermissionStatus 
} from '@/lib/admin-fcm-client';
import { 
  Shield, 
  ShieldOff, 
  Send, 
  Smartphone, 
  Monitor, 
  Tablet, 
  AlertCircle, 
  CheckCircle, 
  XCircle,
  Crown,
  Users,
  Settings,
  BarChart3,
  Lock
} from 'lucide-react';

interface AdminFCMManagerProps {
  className?: string;
}

interface AdminSubscriptionStatus {
  isSupported: boolean;
  permission: AdminFCMPermissionStatus;
  hasToken: boolean;
  isSubscribed: boolean;
}

export function AdminFCMManager({ className }: AdminFCMManagerProps) {
  const [status, setStatus] = useState<AdminSubscriptionStatus>({
    isSupported: false,
    permission: 'unsupported',
    hasToken: false,
    isSubscribed: false
  });
  const [adminTokens, setAdminTokens] = useState<AdminFCMTokenInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  
  // Test notification form
  const [testNotification, setTestNotification] = useState<AdminFCMTestNotificationRequest>({
    title: 'Admin Test Notification',
    body: 'This is a test notification from EPSX admin interface',
    icon: '/logo.png',
    url: '/notifications',
    adminType: 'system_alert',
    priority: 'normal'
  });

  // Load initial admin status
  const loadStatus = useCallback(async () => {
    try {
      setLoading(true);
      const subscriptionStatus = await adminFCMClient.getSubscriptionStatus();
      setStatus(subscriptionStatus);
      
      if (subscriptionStatus.hasToken) {
        const tokens = await adminFCMClient.getUserTokens();
        setAdminTokens(tokens);
      }
    } catch (error) {
      console.error('Error loading admin FCM status:', error);
      setError(error instanceof Error ? error.message : 'Failed to load admin FCM status');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  // Handle subscription toggle
  const handleAdminSubscriptionToggle = async (subscribe: boolean) => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      if (subscribe) {
        const result = await adminFCMClient.subscribe();
        setSuccess('Successfully subscribed to admin push notifications!');
        console.log('Admin FCM subscription result:', result);
      } else {
        const result = await adminFCMClient.unsubscribe();
        setSuccess(result.backendSuccess 
          ? 'Successfully unsubscribed from admin push notifications!' 
          : 'Admin unsubscribed locally, but backend sync may have failed'
        );
        console.log('Admin FCM unsubscription result:', result);
      }
      
      // Reload status
      await loadStatus();
    } catch (error) {
      console.error('Error handling admin subscription:', error);
      setError(error instanceof Error ? error.message : 'Admin subscription operation failed');
    } finally {
      setLoading(false);
    }
  };

  // Handle admin test notification
  const handleSendAdminTestNotification = async () => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      const result = await adminFCMClient.sendTestNotification(testNotification);
      setSuccess(result.message || 'Admin test notification sent successfully!');
    } catch (error) {
      console.error('Error sending admin test notification:', error);
      setError(error instanceof Error ? error.message : 'Failed to send admin test notification');
    } finally {
      setLoading(false);
    }
  };

  // Get permission status badge
  const getPermissionBadge = (permission: AdminFCMPermissionStatus) => {
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

  // Get admin type icon
  const getAdminTypeIcon = (adminType: string) => {
    switch (adminType) {
      case 'user_management':
        return <Users className="w-4 h-4" />;
      case 'system_alert':
        return <Settings className="w-4 h-4" />;
      case 'security_warning':
        return <Lock className="w-4 h-4" />;
      case 'analytics_report':
        return <BarChart3 className="w-4 h-4" />;
      case 'permission_change':
        return <Crown className="w-4 h-4" />;
      default:
        return <Shield className="w-4 h-4" />;
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Admin Status Overview Card */}
      <Card className="border-l-4 border-l-yellow-500">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="w-5 h-5 text-yellow-600" />
            Admin Push Notification Status
          </CardTitle>
          <CardDescription>
            Manage admin push notification preferences and test FCM functionality
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Support Status */}
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Admin FCM Support:</span>
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
            <span className="text-sm font-medium">Admin FCM Token:</span>
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

          {/* Admin Subscription Toggle */}
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-base font-medium">Admin Push Notifications</Label>
              <div className="text-sm text-muted-foreground">
                Receive admin notifications when the interface is not active
              </div>
            </div>
            <Switch
              checked={status.isSubscribed}
              onCheckedChange={handleAdminSubscriptionToggle}
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

      {/* Active Admin Tokens */}
      {adminTokens.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Active Admin Devices</CardTitle>
            <CardDescription>
              Devices currently subscribed to receive admin push notifications
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {adminTokens.map((token, index) => (
                <div key={token.token.substring(0, 20)} className="flex items-center justify-between p-3 border rounded-lg bg-yellow-50 dark:bg-yellow-950/20">
                  <div className="flex items-center gap-3">
                    {getDeviceIcon(token.userAgent)}
                    <div>
                      <div className="text-sm font-medium flex items-center gap-2">
                        <Crown className="w-3 h-3 text-yellow-600" />
                        {token.platform} (Admin)
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Registered: {token.createdAt.toLocaleDateString()}
                      </div>
                      {token.adminPermissions.length > 0 && (
                        <div className="text-xs text-muted-foreground">
                          Permissions: {token.adminPermissions.slice(0, 2).join(', ')}
                          {token.adminPermissions.length > 2 && ` +${token.adminPermissions.length - 2} more`}
                        </div>
                      )}
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

      {/* Admin Test Notification */}
      {status.isSubscribed && (
        <Card>
          <CardHeader>
            <CardTitle>Admin Test Notification</CardTitle>
            <CardDescription>
              Send a test admin push notification to verify functionality
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="admin-test-title">Title</Label>
                <Input
                  id="admin-test-title"
                  value={testNotification.title}
                  onChange={(e) => setTestNotification(prev => ({
                    ...prev,
                    title: e.target.value
                  }))}
                  placeholder="Admin notification title"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="admin-test-url">URL (optional)</Label>
                <Input
                  id="admin-test-url"
                  value={testNotification.url}
                  onChange={(e) => setTestNotification(prev => ({
                    ...prev,
                    url: e.target.value
                  }))}
                  placeholder="/admin/notifications"
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="admin-type">Admin Type</Label>
                <Select
                  value={testNotification.adminType}
                  onValueChange={(value: any) => setTestNotification(prev => ({
                    ...prev,
                    adminType: value
                  }))}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="user_management">
                      <div className="flex items-center gap-2">
                        <Users className="w-4 h-4" />
                        User Management
                      </div>
                    </SelectItem>
                    <SelectItem value="system_alert">
                      <div className="flex items-center gap-2">
                        <Settings className="w-4 h-4" />
                        System Alert
                      </div>
                    </SelectItem>
                    <SelectItem value="security_warning">
                      <div className="flex items-center gap-2">
                        <Lock className="w-4 h-4" />
                        Security Warning
                      </div>
                    </SelectItem>
                    <SelectItem value="analytics_report">
                      <div className="flex items-center gap-2">
                        <BarChart3 className="w-4 h-4" />
                        Analytics Report
                      </div>
                    </SelectItem>
                    <SelectItem value="permission_change">
                      <div className="flex items-center gap-2">
                        <Crown className="w-4 h-4" />
                        Permission Change
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="admin-priority">Priority</Label>
                <Select
                  value={testNotification.priority}
                  onValueChange={(value: any) => setTestNotification(prev => ({
                    ...prev,
                    priority: value
                  }))}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">Low</SelectItem>
                    <SelectItem value="normal">Normal</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="critical">Critical</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="admin-test-body">Message</Label>
              <Textarea
                id="admin-test-body"
                value={testNotification.body}
                onChange={(e) => setTestNotification(prev => ({
                  ...prev,
                  body: e.target.value
                }))}
                placeholder="Admin notification message"
                rows={3}
              />
            </div>

            <Button 
              onClick={handleSendAdminTestNotification}
              disabled={loading || !testNotification.title || !testNotification.body}
              className="w-full"
            >
              {getAdminTypeIcon(testNotification.adminType || 'system_alert')}
              <Send className="w-4 h-4 mr-2" />
              {loading ? 'Sending Admin Test...' : 'Send Admin Test Notification'}
            </Button>
          </CardContent>
        </Card>
      )}

      {/* Unsupported Browser Message */}
      {!status.isSupported && (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center space-y-2">
              <ShieldOff className="w-12 h-12 mx-auto text-muted-foreground" />
              <h3 className="text-lg font-semibold">Admin Push Notifications Not Supported</h3>
              <p className="text-muted-foreground">
                Your browser or device does not support admin push notifications. 
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
          {loading ? 'Refreshing...' : 'Refresh Admin Status'}
        </Button>
      </div>
    </div>
  );
}

export default AdminFCMManager;