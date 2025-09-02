'use client';

import { useRouter } from 'next/navigation';
import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui';
import { FCMNotificationManager } from '@/components/notifications/FCMNotificationManager';
import { Bell, User, Settings, Shield, CheckCircle, AlertCircle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';

interface NotificationPreferences {
  trading: boolean;
  security: boolean;
  account: boolean;
  system: boolean;
  marketing: boolean;
}

export function SettingsClient() {
  const router = useRouter();
  const [notificationPrefs, setNotificationPrefs] = useState<NotificationPreferences>({
    trading: true,
    security: true,
    account: true,
    system: false,
    marketing: false
  });
  const [prefsLoading, setPrefsLoading] = useState(true);
  const [prefsError, setPrefsError] = useState<string | null>(null);
  const [prefsSuccess, setPrefsSuccess] = useState<string | null>(null);

  // Load notification preferences on component mount
  useEffect(() => {
    loadNotificationPreferences();
  }, []);

  const loadNotificationPreferences = async () => {
    try {
      setPrefsLoading(true);
      const response = await fetch('/api/v1/notifications/preferences');
      
      if (response.ok) {
        const data = await response.json();
        setNotificationPrefs(data.preferences);
      } else {
        throw new Error('Failed to load preferences');
      }
    } catch (error) {
      console.error('Error loading notification preferences:', error);
      setPrefsError('Failed to load notification preferences');
    } finally {
      setPrefsLoading(false);
    }
  };

  const handleSignOut = async () => {
    // Redirect to OIDC logout endpoint
    router.push('/auth/logout');
  };

  const updateNotificationPrefs = async (prefs: NotificationPreferences) => {
    try {
      setPrefsError(null);
      setPrefsSuccess(null);
      
      const response = await fetch('/api/v1/notifications/preferences', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(prefs)
      });

      if (response.ok) {
        setNotificationPrefs(prefs);
        setPrefsSuccess('Notification preferences updated successfully!');
        
        // Clear success message after 3 seconds
        setTimeout(() => setPrefsSuccess(null), 3000);
      } else {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to update preferences');
      }
    } catch (error) {
      console.error('Error updating notification preferences:', error);
      setPrefsError(error instanceof Error ? error.message : 'Failed to update preferences');
    }
  };

  return (
    <Tabs defaultValue="notifications" className="space-y-6">
      <TabsList className="grid w-full grid-cols-3">
        <TabsTrigger value="notifications" className="flex items-center gap-2">
          <Bell className="h-4 w-4" />
          Notifications
        </TabsTrigger>
        <TabsTrigger value="account" className="flex items-center gap-2">
          <User className="h-4 w-4" />
          Account
        </TabsTrigger>
        <TabsTrigger value="privacy" className="flex items-center gap-2">
          <Shield className="h-4 w-4" />
          Privacy
        </TabsTrigger>
      </TabsList>

      {/* FCM Notification Preferences */}
      <TabsContent value="notifications" className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              🔔 Push Notification Settings
            </CardTitle>
          </CardHeader>
          <CardContent>
            <FCMNotificationManager />
          </CardContent>
        </Card>

        {/* Error/Success Alerts */}
        {prefsError && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{prefsError}</AlertDescription>
          </Alert>
        )}

        {prefsSuccess && (
          <Alert>
            <CheckCircle className="h-4 w-4" />
            <AlertDescription>{prefsSuccess}</AlertDescription>
          </Alert>
        )}

        {/* Notification Category Preferences */}
        <Card>
          <CardHeader>
            <CardTitle>Notification Categories</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {prefsLoading ? (
              <div className="text-center py-4">
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900 mx-auto"></div>
                <p className="text-sm text-muted-foreground mt-2">Loading preferences...</p>
              </div>
            ) : (
            <div className="grid gap-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">📈 Trading Alerts</div>
                  <div className="text-sm text-muted-foreground">Price movements, portfolio updates</div>
                </div>
                <input
                  type="checkbox"
                  checked={notificationPrefs.trading}
                  onChange={(e) => updateNotificationPrefs({
                    ...notificationPrefs,
                    trading: e.target.checked
                  })}
                  className="w-4 h-4"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">🔐 Security Alerts</div>
                  <div className="text-sm text-muted-foreground">Login attempts, security warnings</div>
                </div>
                <input
                  type="checkbox"
                  checked={notificationPrefs.security}
                  onChange={(e) => updateNotificationPrefs({
                    ...notificationPrefs,
                    security: e.target.checked
                  })}
                  className="w-4 h-4"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">👤 Account Updates</div>
                  <div className="text-sm text-muted-foreground">Profile changes, subscription updates</div>
                </div>
                <input
                  type="checkbox"
                  checked={notificationPrefs.account}
                  onChange={(e) => updateNotificationPrefs({
                    ...notificationPrefs,
                    account: e.target.checked
                  })}
                  className="w-4 h-4"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">⚙️ System Notifications</div>
                  <div className="text-sm text-muted-foreground">Maintenance, platform updates</div>
                </div>
                <input
                  type="checkbox"
                  checked={notificationPrefs.system}
                  onChange={(e) => updateNotificationPrefs({
                    ...notificationPrefs,
                    system: e.target.checked
                  })}
                  className="w-4 h-4"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">📢 Marketing & Promotions</div>
                  <div className="text-sm text-muted-foreground">New features, special offers</div>
                </div>
                <input
                  type="checkbox"
                  checked={notificationPrefs.marketing}
                  onChange={(e) => updateNotificationPrefs({
                    ...notificationPrefs,
                    marketing: e.target.checked
                  })}
                  className="w-4 h-4"
                />
              </div>
            </div>
            )}
          </CardContent>
        </Card>
      </TabsContent>

      {/* Account Settings */}
      <TabsContent value="account">
        <Card>
          <CardHeader>
            <CardTitle>Account Actions</CardTitle>
          </CardHeader>
          <CardContent>
            <Button onClick={handleSignOut} variant="destructive">
              Sign Out
            </Button>
          </CardContent>
        </Card>
      </TabsContent>

      {/* Privacy Settings */}
      <TabsContent value="privacy">
        <Card>
          <CardHeader>
            <CardTitle>Privacy & Data</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <h4 className="font-medium">Data Collection</h4>
              <div className="text-sm text-muted-foreground">
                We collect minimal data required for platform functionality. 
                View our <a href="/privacy" className="text-blue-600 hover:underline">Privacy Policy</a> for details.
              </div>
            </div>
            <div className="space-y-2">
              <h4 className="font-medium">Cookie Settings</h4>
              <div className="text-sm text-muted-foreground">
                Essential cookies are required for authentication and security.
              </div>
            </div>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  );
}