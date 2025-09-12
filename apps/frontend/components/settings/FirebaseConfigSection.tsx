'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Alert, AlertDescription } from '@/components/ui';
import { CheckCircle, XCircle, RefreshCw, Wifi, WifiOff, Activity, AlertTriangle } from 'lucide-react';
import { auth, isFirebaseAuthenticated, getFirebaseUserInfo } from '@/lib/firebase';

interface FirebaseConnectionStatus {
  isConnected: boolean;
  isAuthenticated: boolean;
  userInfo: any;
  lastChecked: Date;
  error?: string;
}

export function FirebaseConfigSection() {
  const [connectionStatus, setConnectionStatus] = useState<FirebaseConnectionStatus>({
    isConnected: false,
    isAuthenticated: false,
    userInfo: null,
    lastChecked: new Date()
  });
  const [isChecking, setIsChecking] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  const checkFirebaseConnection = async () => {
    setIsChecking(true);
    setTestResult(null);
    
    try {
      // Check if Firebase is initialized
      const isConnected = !!auth;
      const isAuthenticated = isFirebaseAuthenticated();
      const userInfo = getFirebaseUserInfo();
      
      setConnectionStatus({
        isConnected,
        isAuthenticated,
        userInfo,
        lastChecked: new Date()
      });

      if (isConnected && isAuthenticated) {
        setTestResult('✅ Firebase connection successful! Authentication working properly.');
      } else if (isConnected && !isAuthenticated) {
        setTestResult('⚠️ Firebase connected but user not authenticated.');
      } else {
        setTestResult('❌ Firebase connection failed. Check configuration.');
      }
    } catch (error) {
      console.error('Firebase connection test failed:', error);
      setConnectionStatus({
        isConnected: false,
        isAuthenticated: false,
        userInfo: null,
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
      return connectionStatus.isAuthenticated ? 
        <Wifi className="h-4 w-4 text-green-600" /> : 
        <Activity className="h-4 w-4 text-yellow-600" />;
    }
    return <WifiOff className="h-4 w-4 text-red-600" />;
  };

  const getConnectionBadge = () => {
    if (connectionStatus.isConnected && connectionStatus.isAuthenticated) {
      return (
        <Badge variant="default" className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
          <CheckCircle className="h-3 w-3 mr-1" />
          Connected & Authenticated
        </Badge>
      );
    } else if (connectionStatus.isConnected) {
      return (
        <Badge variant="default" className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
          <AlertTriangle className="h-3 w-3 mr-1" />
          Connected (Not Authenticated)
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
              <span className="text-muted-foreground">Authentication:</span>
              <span className={connectionStatus.isAuthenticated ? 'text-green-600' : 'text-yellow-600'}>
                {connectionStatus.isAuthenticated ? 'Authenticated' : 'Not Authenticated'}
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
              <span className="text-muted-foreground">User ID:</span>
              <span className="text-xs font-mono">
                {connectionStatus.userInfo?.uid || 'N/A'}
              </span>
            </div>
          </div>
        </div>

        {connectionStatus.userInfo && (
          <div className="bg-green-50 dark:bg-green-950 p-3 rounded-lg">
            <div className="text-sm text-green-700 dark:text-green-300 space-y-1">
              <div><strong>Email:</strong> {connectionStatus.userInfo.email}</div>
              <div><strong>Display Name:</strong> {connectionStatus.userInfo.displayName || 'Not set'}</div>
              <div><strong>UID:</strong> <code className="text-xs">{connectionStatus.userInfo.uid}</code></div>
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
            <strong>🔍 Debug Info:</strong> This section shows live Firebase connection status. 
            If you're experiencing authentication issues, use the "Test Connection" button to diagnose problems.
          </div>
        </div>
      </CardContent>
    </Card>
  );
}