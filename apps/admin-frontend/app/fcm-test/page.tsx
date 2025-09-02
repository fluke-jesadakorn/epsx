import { Metadata } from 'next';
import { AdminFCMManager } from '@/components/notifications/AdminFCMManager';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Shield, InfoIcon } from 'lucide-react';

export const metadata: Metadata = {
  title: 'Admin FCM Test - EPSX Admin',
  description: 'Test Firebase Cloud Messaging functionality for admin interface',
};

export default function AdminFCMTestPage() {
  return (
    <div className="container mx-auto py-8 space-y-8">
      {/* Header */}
      <div className="space-y-2">
        <h1 className="text-3xl font-bold flex items-center gap-2">
          <Shield className="h-8 w-8 text-yellow-600" />
          Admin FCM Notification Testing
        </h1>
        <p className="text-muted-foreground">
          Test Firebase Cloud Messaging functionality for EPSX admin interface
        </p>
      </div>

      {/* Development Notice */}
      <Alert className="border-yellow-200 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-950/20">
        <InfoIcon className="h-4 w-4 text-yellow-600" />
        <AlertDescription>
          This page is for testing admin FCM push notifications. In production, admin notification 
          management would be integrated into the admin settings page.
        </AlertDescription>
      </Alert>

      {/* Admin FCM Status Overview */}
      <Card className="border-l-4 border-l-yellow-500">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-6 w-6 text-yellow-600" />
            Admin FCM Integration Status
          </CardTitle>
          <CardDescription>
            Current Firebase Cloud Messaging setup for EPSX admin interface
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-green-600">✓</div>
              <div className="text-sm font-medium">Admin Service Worker</div>
              <Badge variant="default">Active</Badge>
            </div>
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-green-600">✓</div>
              <div className="text-sm font-medium">Admin Firebase Config</div>
              <Badge variant="default">Connected</Badge>
            </div>
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-blue-600">⚡</div>
              <div className="text-sm font-medium">Admin Backend API</div>
              <Badge variant="secondary">Ready</Badge>
            </div>
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-yellow-600">🛡️</div>
              <div className="text-sm font-medium">Admin Permissions</div>
              <Badge variant="outline" className="border-yellow-300">Verified</Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Admin FCM Manager Component */}
      <AdminFCMManager />

      {/* Admin Implementation Details */}
      <Card>
        <CardHeader>
          <CardTitle>Admin Implementation Details</CardTitle>
          <CardDescription>
            Technical information about admin FCM implementation
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-semibold mb-2 text-yellow-700 dark:text-yellow-400">Admin Frontend Features</h4>
              <ul className="text-sm space-y-1 text-muted-foreground">
                <li>• Admin-specific service worker</li>
                <li>• Enhanced permission management</li>
                <li>• Admin notification categorization</li>
                <li>• Priority-based notifications</li>
                <li>• Admin sound notifications</li>
                <li>• Enhanced security handling</li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold mb-2 text-yellow-700 dark:text-yellow-400">Admin Backend Integration</h4>
              <ul className="text-sm space-y-1 text-muted-foreground">
                <li>• Admin token registration</li>
                <li>• Permission-aware notifications</li>
                <li>• Admin analytics tracking</li>
                <li>• Role-based notification routing</li>
                <li>• Enhanced error handling</li>
                <li>• Admin audit logging</li>
              </ul>
            </div>
          </div>

          <div className="pt-4 border-t">
            <h4 className="font-semibold mb-2">Admin API Endpoints</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <code className="text-xs bg-yellow-100 dark:bg-yellow-950 px-2 py-1 rounded">
                  POST /api/v1/admin/notifications/fcm/subscribe
                </code>
                <p className="text-muted-foreground mt-1">Subscribe to admin push notifications</p>
              </div>
              <div>
                <code className="text-xs bg-yellow-100 dark:bg-yellow-950 px-2 py-1 rounded">
                  POST /api/v1/admin/notifications/fcm/unsubscribe
                </code>
                <p className="text-muted-foreground mt-1">Unsubscribe from admin notifications</p>
              </div>
              <div>
                <code className="text-xs bg-yellow-100 dark:bg-yellow-950 px-2 py-1 rounded">
                  POST /api/v1/admin/notifications/fcm/test
                </code>
                <p className="text-muted-foreground mt-1">Send admin test notification</p>
              </div>
              <div>
                <code className="text-xs bg-yellow-100 dark:bg-yellow-950 px-2 py-1 rounded">
                  GET /api/v1/admin/notifications/fcm/tokens/my
                </code>
                <p className="text-muted-foreground mt-1">Get admin user's FCM tokens</p>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t">
            <h4 className="font-semibold mb-2">Admin Notification Types</h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-2 text-sm">
              <Badge variant="outline" className="justify-center">User Management</Badge>
              <Badge variant="outline" className="justify-center">System Alert</Badge>
              <Badge variant="outline" className="justify-center">Security Warning</Badge>
              <Badge variant="outline" className="justify-center">Analytics Report</Badge>
              <Badge variant="outline" className="justify-center">Permission Change</Badge>
              <Badge variant="outline" className="justify-center">Admin Message</Badge>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}