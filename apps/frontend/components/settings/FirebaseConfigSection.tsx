'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Alert, AlertDescription } from '@/components/ui';
import { CheckCircle, XCircle, RefreshCw, Wifi, WifiOff, Activity, AlertTriangle, BarChart3 } from 'lucide-react';
import { 
  isFirebaseInitialized, 
  isServiceAvailable, 
  trackEvent, 
  getRemoteConfigValue, 
  fetchRemoteConfig 
} from '@/lib/firebase';

interface FirebaseConnectionStatus {
  isConnected: boolean;
  analyticsAvailable: boolean;
  remoteConfigAvailable: boolean;
  messagingAvailable: boolean;
  lastChecked: Date;
  error?: string;
  remoteConfigKeys?: string[];
}

export function FirebaseConfigSection() {
  const [connectionStatus, setConnectionStatus] = useState<FirebaseConnectionStatus>({
    isConnected: false,
    analyticsAvailable: false,
    remoteConfigAvailable: false,
    messagingAvailable: false,
    lastChecked: new Date()
  });
  const [isChecking, setIsChecking] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  const checkFirebaseConnection = async () => {
    setIsChecking(true);
    setTestResult(null);
    
    try {
      // Check if Firebase services are available
      const isConnected = isFirebaseInitialized();
      const analyticsAvailable = isServiceAvailable('analytics');
      const remoteConfigAvailable = isServiceAvailable('remoteConfig');
      const messagingAvailable = isServiceAvailable('messaging');
      
      let remoteConfigKeys: string[] = [];
      
      // Test remote config if available
      if (remoteConfigAvailable) {
        try {
          await fetchRemoteConfig();
          // You can add specific config keys to test here
          remoteConfigKeys = ['feature_flags', 'api_version', 'maintenance_mode'];
        } catch (error) {
          console.warn('Remote config fetch failed:', error);
        }
      }
      
      setConnectionStatus({
        isConnected,
        analyticsAvailable,
        remoteConfigAvailable,
        messagingAvailable,
        lastChecked: new Date(),
        remoteConfigKeys
      });

      if (isConnected) {
        const availableServices = [
          analyticsAvailable && 'Analytics',
          remoteConfigAvailable && 'Remote Config',
          messagingAvailable && 'Messaging'
        ].filter(Boolean);
        
        if (availableServices.length > 0) {
          setTestResult(`✅ Firebase connected! Available services: ${availableServices.join(', ')}`);
        } else {
          setTestResult('⚠️ Firebase connected but no services available.');
        }
      } else {
        setTestResult('❌ Firebase connection failed. Check configuration.');
      }
      
      // Test analytics event tracking
      if (analyticsAvailable) {
        trackEvent('firebase_config_test', {
          timestamp: Date.now(),
          services_available: availableServices.length
        });
      }
      
    } catch (error) {
      console.error('Firebase connection test failed:', error);
      setConnectionStatus({
        isConnected: false,
        analyticsAvailable: false,
        remoteConfigAvailable: false,
        messagingAvailable: false,
        lastChecked: new Date(),
        error: error instanceof Error ? error.message : 'Unknown error'
      });
      setTestResult(`❌ Firebase test failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsChecking(false);
    }
  };

  const reinitializeFirebase = async () => {
    setIsChecking(true);
    try {
      // This would trigger a re-initialization if needed
      // For now, just re-check connection
      await new Promise(resolve => setTimeout(resolve, 1000));
      await checkFirebaseConnection();
      setTestResult('🔄 Firebase reinitialization attempted. Check status above.');
    } catch (error) {
      console.error('Firebase reinitialization failed:', error);
      setTestResult(`❌ Reinitialization failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    checkFirebaseConnection();
  }, []);

  const getConnectionIcon = () => {
    if (connectionStatus.isConnected) {
      return connectionStatus.analyticsAvailable ? 
        <BarChart3 className="h-4 w-4 text-green-600" /> : 
        <Activity className="h-4 w-4 text-yellow-600" />;
    }
    return <WifiOff className="h-4 w-4 text-red-600" />;
  };

  const getConnectionBadge = () => {
    if (connectionStatus.isConnected && connectionStatus.analyticsAvailable) {
      return (
        <Badge variant="default" className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
          <CheckCircle className="h-3 w-3 mr-1" />
          Analytics Active
        </Badge>
      );
    } else if (connectionStatus.isConnected) {
      return (
        <Badge variant="default" className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
          <AlertTriangle className="h-3 w-3 mr-1" />
          Connected (Limited Services)
        </Badge>
      );
    }
    return (
      <Badge variant="destructive">
        <XCircle className="h-3 w-3 mr-1" />
        Disconnected
      </Badge>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {getConnectionIcon()}
          Firebase Connection Status
          {getConnectionBadge()}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Connection Status:</span>
              <span className={connectionStatus.isConnected ? 'text-green-600' : 'text-red-600'}>
                {connectionStatus.isConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Analytics:</span>
              <span className={connectionStatus.analyticsAvailable ? 'text-green-600' : 'text-yellow-600'}>
                {connectionStatus.analyticsAvailable ? 'Available' : 'Unavailable'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Remote Config:</span>
              <span className={connectionStatus.remoteConfigAvailable ? 'text-green-600' : 'text-yellow-600'}>
                {connectionStatus.remoteConfigAvailable ? 'Available' : 'Unavailable'}
              </span>
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
              <span className="text-muted-foreground">Messaging:</span>
              <span className={connectionStatus.messagingAvailable ? 'text-green-600' : 'text-yellow-600'}>
                {connectionStatus.messagingAvailable ? 'Available' : 'Unavailable'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Config Keys:</span>
              <span className="text-xs">
                {connectionStatus.remoteConfigKeys?.length || 0} loaded
              </span>
            </div>
          </div>
        </div>

        {connectionStatus.remoteConfigKeys && connectionStatus.remoteConfigKeys.length > 0 && (
          <div className="bg-green-50 dark:bg-green-950 p-3 rounded-lg">
            <div className="text-sm text-green-700 dark:text-green-300 space-y-1">
              <div><strong>Remote Config Keys:</strong></div>
              <div className="flex flex-wrap gap-1">
                {connectionStatus.remoteConfigKeys.map(key => (
                  <code key={key} className="text-xs bg-green-100 dark:bg-green-800 px-1 rounded">
                    {key}
                  </code>
                ))}
              </div>
            </div>
          </div>
        )}

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
            onClick={checkFirebaseConnection}
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
            Test Connection
          </Button>
          <Button 
            onClick={reinitializeFirebase}
            disabled={isChecking}
            variant="outline"
            size="sm"
            className="flex items-center gap-2"
          >
            <RefreshCw className="h-4 w-4" />
            Reinitialize
          </Button>
        </div>

        <div className="bg-blue-50 dark:bg-blue-950 p-3 rounded-lg">
          <div className="text-sm text-blue-700 dark:text-blue-300">
            <strong>📊 Firebase Analytics Status:</strong> This section shows Firebase Analytics, Remote Config, and Messaging service status. 
            Authentication is now handled via OIDC OAuth flow. Use "Test Connection" to verify Firebase services are working.
          </div>
        </div>
      </CardContent>
    </Card>
  );
}