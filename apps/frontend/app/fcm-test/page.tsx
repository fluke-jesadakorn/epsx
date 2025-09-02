import { Metadata } from 'next';
import { FCMNotificationManager } from '@/components/notifications/FCMNotificationManager';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { InfoIcon } from 'lucide-react';

export const metadata: Metadata = {
  title: 'FCM Test - EPSX',
  description: 'Test Firebase Cloud Messaging functionality',
};

export default function FCMTestPage() {
  return (
    <div className="container mx-auto py-8 space-y-8">
      {/* Header */}
      <div className="space-y-2">
        <h1 className="text-3xl font-bold">FCM Notification Testing</h1>
        <p className="text-muted-foreground">
          Test Firebase Cloud Messaging functionality for EPSX platform
        </p>
      </div>

      {/* Development Notice */}
      <Alert>
        <InfoIcon className="h-4 w-4" />
        <AlertDescription>
          This page is for testing FCM push notifications. In production, notification management 
          would be integrated into the main settings page.
        </AlertDescription>
      </Alert>

      {/* FCM Status Overview */}
      <Card>
        <CardHeader>
          <CardTitle>FCM Integration Status</CardTitle>
          <CardDescription>
            Current Firebase Cloud Messaging setup for EPSX platform
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-green-600">✓</div>
              <div className="text-sm font-medium">Service Worker</div>
              <Badge variant="default">Active</Badge>
            </div>
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-green-600">✓</div>
              <div className="text-sm font-medium">Firebase Config</div>
              <Badge variant="default">Connected</Badge>
            </div>
            <div className="text-center space-y-2">
              <div className="text-2xl font-bold text-blue-600">⚡</div>
              <div className="text-sm font-medium">Backend API</div>
              <Badge variant="secondary">Ready</Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* FCM Manager Component */}
      <FCMNotificationManager />

      {/* Implementation Details */}
      <Card>
        <CardHeader>
          <CardTitle>Implementation Details</CardTitle>
          <CardDescription>
            Technical information about FCM implementation
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-semibold mb-2">Frontend Features</h4>
              <ul className="text-sm space-y-1 text-muted-foreground">
                <li>• Firebase v10+ modular SDK</li>
                <li>• Service worker registration</li>
                <li>• Token management & refresh</li>
                <li>• Foreground message handling</li>
                <li>• Permission management</li>
                <li>• Background notification support</li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold mb-2">Backend Integration</h4>
              <ul className="text-sm space-y-1 text-muted-foreground">
                <li>• Token registration via API</li>
                <li>• Notification analytics tracking</li>
                <li>• User token management</li>
                <li>• Test notification sending</li>
                <li>• Subscription management</li>
                <li>• Error handling & recovery</li>
              </ul>
            </div>
          </div>

          <div className="pt-4 border-t">
            <h4 className="font-semibold mb-2">API Endpoints</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">
                  POST /api/v1/notifications/fcm/subscribe
                </code>
                <p className="text-muted-foreground mt-1">Subscribe to push notifications</p>
              </div>
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">
                  POST /api/v1/notifications/fcm/unsubscribe
                </code>
                <p className="text-muted-foreground mt-1">Unsubscribe from notifications</p>
              </div>
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">
                  POST /api/v1/notifications/fcm/test
                </code>
                <p className="text-muted-foreground mt-1">Send test notification</p>
              </div>
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">
                  GET /api/v1/notifications/fcm/tokens/my
                </code>
                <p className="text-muted-foreground mt-1">Get user's FCM tokens</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}