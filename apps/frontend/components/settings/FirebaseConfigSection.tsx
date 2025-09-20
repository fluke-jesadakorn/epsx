'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Alert, AlertDescription } from '@/components/ui';
import { CheckCircle, XCircle, RefreshCw, Activity, AlertTriangle, BarChart3 } from 'lucide-react';
import { 
  isFirebaseInitialized, 
  isServiceAvailable, 
  trackEvent, 
  fetchRemoteConfig 
} from '@/lib/utils/firebase';

interface FirebaseConnectionStatus {
  analyticsConfigured: boolean;
  lastChecked: Date;
  error?: string;
}

export function FirebaseConfigSection() {
  const [connectionStatus, setConnectionStatus] = useState<FirebaseConnectionStatus>({
    analyticsConfigured: false,
    lastChecked: new Date()
  });
  const [isChecking, setIsChecking] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  const checkFirebaseConfiguration = async () => {
    setIsChecking(true);
    setTestResult(null);
    
    try {
      // Check if Firebase analytics configuration is available
      const hasApiKey = !!process.env.NEXT_PUBLIC_FIREBASE_API_KEY;
      const hasProjectId = !!process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID;
      const hasAppId = !!process.env.NEXT_PUBLIC_FIREBASE_APP_ID;
      
      const analyticsConfigured = hasApiKey && hasProjectId && hasAppId;
      
      setConnectionStatus({
        analyticsConfigured,
        lastChecked: new Date()
      });

      if (analyticsConfigured) {
        setTestResult('✅ Firebase Analytics configuration is complete');
      } else {
        const missing = [];
        if (!hasApiKey) missing.push('API Key');
        if (!hasProjectId) missing.push('Project ID');
        if (!hasAppId) missing.push('App ID');
        setTestResult(`❌ Missing Firebase config: ${missing.join(', ')}`);
      }
      
    } catch (error) {
      console.error('Firebase configuration check failed:', error);
      setConnectionStatus({
        analyticsConfigured: false,
        lastChecked: new Date(),
        error: error instanceof Error ? error.message : 'Unknown error'
      });
      setTestResult(`❌ Configuration check failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    checkFirebaseConfiguration();
  }, []);

  const getConfigIcon = () => {
    return connectionStatus.analyticsConfigured ? 
      <BarChart3 className="h-4 w-4 text-green-600" /> : 
      <AlertTriangle className="h-4 w-4 text-yellow-600" />;
  };

  const getConfigBadge = () => {
    if (connectionStatus.analyticsConfigured) {
      return (
        <Badge variant="default" className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
          <CheckCircle className="h-3 w-3 mr-1" />
          Analytics Configured
        </Badge>
      );
    }
    return (
      <Badge variant="destructive">
        <XCircle className="h-3 w-3 mr-1" />
        Not Configured
      </Badge>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {getConfigIcon()}
          Firebase Analytics Configuration
          {getConfigBadge()}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Analytics Status:</span>
              <span className={connectionStatus.analyticsConfigured ? 'text-green-600' : 'text-red-600'}>
                {connectionStatus.analyticsConfigured ? 'Configured' : 'Not Configured'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">FCM Messaging:</span>
              <span className="text-yellow-600">Disabled</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Remote Config:</span>
              <span className="text-yellow-600">Disabled</span>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Last Checked:</span>
              <span className="text-xs">
                {connectionStatus.lastChecked.toLocaleTimeString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Auth Method:</span>
              <span className="text-blue-600">OIDC + Web3</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Notifications:</span>
              <span className="text-blue-600">REST API</span>
            </div>
          </div>
        </div>

        {connectionStatus.error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <strong>Connection Error:</strong> {connectionStatus.error}
            </AlertDescription>
          </Alert>
        )}

        {testResult && (
          <Alert>
            <Activity className="h-4 w-4" />
            <AlertDescription>
              {testResult}
            </AlertDescription>
          </Alert>
        )}

        <div className="flex gap-3">
          <Button 
            onClick={checkFirebaseConfiguration}
            disabled={isChecking}
            variant="outline"
            size="sm"
            className="flex items-center gap-2"
          >
            {isChecking ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <Activity className="h-4 w-4" />
            )}
            Check Configuration
          </Button>
        </div>

        <div className="bg-blue-50 dark:bg-blue-950 p-3 rounded-lg">
          <div className="text-sm text-blue-700 dark:text-blue-300">
            <strong>📊 Firebase Analytics Only:</strong> Firebase is now configured for analytics only. 
            FCM messaging and remote config have been disabled for Web3 migration. Authentication uses OIDC + Web3 wallets.
          </div>
        </div>
      </CardContent>
    </Card>
  );
}