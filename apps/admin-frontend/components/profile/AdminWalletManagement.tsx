'use client';

import { 
  Wallet, 
  Power, 
  RefreshCcw, 
  AlertTriangle, 
  CheckCircle, 
  Clock,
  TrendingUp,
  Activity,
  Shield
} from 'lucide-react';
import { useState, useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { useAccount, useDisconnect, useBalance } from 'wagmi';

import { AdminWalletAuth } from '@/components/auth/AdminWalletAuth';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { apiFetch } from '@/lib/api-fetch';

interface WalletSession {
  wallet_address: string;
  connected_at: number;
  expires_at: number;
  last_activity: number;
  session_id: string;
}

interface AdminWalletManagementProps {
  initialSession?: WalletSession;
}

/**
 *
 * @param root0
 * @param root0.initialSession
 */
export function AdminWalletManagement({ initialSession }: AdminWalletManagementProps) {
  const { address, isConnected, connector } = useAccount();
  const { disconnect } = useDisconnect();
  const { data: balance } = useBalance({ address });
  
  const [session, setSession] = useState<WalletSession | null>(initialSession || null);
  const [isLoading, setIsLoading] = useState(false);
  const [sessionHealth, setSessionHealth] = useState(100);
  const [lastActivity, setLastActivity] = useState<Date>(new Date());

  useEffect(() => {
    if (address && isConnected) {
      fetchSessionInfo(address);
      startSessionMonitoring();
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (session) {
      updateSessionHealth();
    }
  }, [session]);

  const fetchSessionInfo = async (walletAddress: string) => {
    try {
      setIsLoading(true);
      const sessionData = await apiFetch('/api/auth/session');

      if (sessionData.wallet_address === walletAddress) {
        setSession({
          wallet_address: walletAddress,
          connected_at: Date.now() - (3600 * 1000), // 1 hour ago (mock)
          expires_at: Date.now() + (3600 * 1000), // 1 hour from now
          last_activity: Date.now(),
          session_id: sessionData.session_id || 'mock-session-id'
        });
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to fetch session info:', _error);
    } finally {
      setIsLoading(false);
    }
  };

  const startSessionMonitoring = () => {
    const interval = setInterval(() => {
      setLastActivity(new Date());
      if (session) {
        updateSessionHealth();
      }
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  };

  const updateSessionHealth = () => {
    if (!session) {return;}

    const now = Date.now();
    const sessionDuration = now - session.connected_at;
    const maxDuration = session.expires_at - session.connected_at;
    const remainingTime = session.expires_at - now;
    
    // Calculate health based on remaining time
    const healthPercent = Math.max(0, (remainingTime / maxDuration) * 100);
    setSessionHealth(Math.round(healthPercent));
  };

  const handleExtendSession = async () => {
    if (!address || !session) {return;}

    try {
      setIsLoading(true);

      // Mock extending session by refreshing authentication
      await apiFetch('/api/auth/web3/verify', {
        method: 'POST',
        body: JSON.stringify({
          wallet_address: address,
          extend_session: true
        }),
      });

      setSession(prev => prev ? {
        ...prev,
        expires_at: Date.now() + (3600 * 1000), // Extend by 1 hour
        last_activity: Date.now()
      } : null);
      toast.success('Session extended successfully');
    } catch (_error) {
      toast.error('Failed to extend session');
    } finally {
      setIsLoading(false);
    }
  };

  const handleForceDisconnect = async () => {
    try {
      setIsLoading(true);

      // Call logout API
      await apiFetch('/api/auth/logout', {
        method: 'POST',
      });

      // Disconnect wallet
      disconnect();
      setSession(null);

      toast.success('Wallet disconnected successfully');
    } catch (_error) {
      toast.error('Failed to disconnect wallet');
    } finally {
      setIsLoading(false);
    }
  };

  const formatDuration = (timestamp: number) => {
    const diff = Math.abs(Date.now() - timestamp);
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
    
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  const getSessionStatus = () => {
    if (!session) {return { status: 'disconnected', color: 'text-gray-500' };}
    
    const remainingTime = session.expires_at - Date.now();
    
    if (remainingTime <= 0) {
      return { status: 'expired', color: 'text-red-500' };
    } else if (remainingTime < 300000) { // Less than 5 minutes
      return { status: 'expiring soon', color: 'text-orange-500' };
    } else {
      return { status: 'active', color: 'text-green-500' };
    }
  };

  const getHealthColor = () => {
    if (sessionHealth >= 70) {return 'bg-green-500';}
    if (sessionHealth >= 30) {return 'bg-orange-500';}
    return 'bg-red-500';
  };

  const sessionStatus = getSessionStatus();

  return (
    <div className="space-y-6">
      {/* Connection Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Wallet Connection Status
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {!isConnected ? (
            <div className="text-center py-6">
              <div className="mb-4">
                <Power className="h-12 w-12 mx-auto text-gray-400 mb-2" />
                <p className="text-gray-600 mb-4">No wallet connected</p>
              </div>
              <AdminWalletAuth 
                className="w-full"
                onAuthSuccess={(address) => {
                  toast.success('Wallet connected successfully');
                }}
                onAuthError={(error) => {
                  toast.error(error);
                }}
              />
            </div>
          ) : (
            <div className="space-y-4">
              {/* Connection Info */}
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <p className="text-sm font-medium">Status</p>
                  <div className="flex items-center gap-2">
                    <CheckCircle className={`h-4 w-4 ${sessionStatus.color}`} />
                    <span className={`text-sm capitalize ${sessionStatus.color}`}>
                      {sessionStatus.status}
                    </span>
                  </div>
                </div>
                <div className="space-y-1">
                  <p className="text-sm font-medium">Connector</p>
                  <p className="text-sm text-gray-600">
                    {connector?.name || 'Unknown'}
                  </p>
                </div>
              </div>

              {/* Session Health */}
              {session && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium">Session Health</p>
                    <span className="text-sm text-gray-600">{sessionHealth}%</span>
                  </div>
                  <Progress value={sessionHealth} className="h-2" />
                  <div className={`w-full bg-gray-200 rounded-full h-2`}>
                    <div 
                      className={`h-2 rounded-full transition-all duration-300 ${getHealthColor()}`}
                      style={{ width: `${sessionHealth}%` }}
                    />
                  </div>
                </div>
              )}

              {/* Session Details */}
              {session && (
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div className="space-y-1">
                    <p className="font-medium">Connected</p>
                    <p className="text-gray-600">{formatDuration(session.connected_at)} ago</p>
                  </div>
                  <div className="space-y-1">
                    <p className="font-medium">Expires</p>
                    <p className="text-gray-600">in {formatDuration(session.expires_at)}</p>
                  </div>
                  <div className="space-y-1">
                    <p className="font-medium">Last Activity</p>
                    <p className="text-gray-600">{formatDuration(session.last_activity)} ago</p>
                  </div>
                  <div className="space-y-1">
                    <p className="font-medium">Session ID</p>
                    <code className="text-xs bg-gray-100 px-1 rounded">
                      {session.session_id.substring(0, 8)}...
                    </code>
                  </div>
                </div>
              )}

              {/* Wallet Balance */}
              {balance && (
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <TrendingUp className="h-4 w-4 text-green-500" />
                      <span className="text-sm font-medium">Balance</span>
                    </div>
                    <span className="text-sm font-mono">
                      {parseFloat(balance.formatted).toFixed(4)} {balance.symbol}
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Session Management */}
      {isConnected && session && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Session Management
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Session Expiry Warning */}
            {sessionHealth < 30 && (
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Your session is expiring soon. Extend your session or you'll need to reconnect.
                </AlertDescription>
              </Alert>
            )}

            {/* Session Actions */}
            <div className="flex gap-2">
              <Button
                onClick={handleExtendSession}
                disabled={isLoading}
                className="flex items-center gap-2"
              >
                <RefreshCcw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
                Extend Session
              </Button>
              
              <Button
                variant="outline"
                onClick={handleForceDisconnect}
                disabled={isLoading}
                className="flex items-center gap-2 text-red-600 hover:text-red-700"
              >
                <Power className="h-4 w-4" />
                Disconnect
              </Button>
            </div>

            {/* Security Notice */}
            <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
              <div className="flex items-start gap-3">
                <Shield className="h-4 w-4 text-blue-600 mt-0.5" />
                <div className="text-sm">
                  <p className="font-medium text-blue-800">Security Notice</p>
                  <p className="text-blue-700 mt-1">
                    Always disconnect your wallet when finished using the admin panel, 
                    especially on shared computers.
                  </p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Connection History */}
      {isConnected && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Recent Activity
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-green-50 border border-green-200 rounded-lg">
                <div className="flex items-center gap-3">
                  <CheckCircle className="h-4 w-4 text-green-600" />
                  <div>
                    <p className="text-sm font-medium">Wallet Connected</p>
                    <p className="text-xs text-gray-600">Admin session established</p>
                  </div>
                </div>
                <span className="text-xs text-gray-500">
                  {session ? formatDuration(session.connected_at) + ' ago' : 'Now'}
                </span>
              </div>

              <div className="flex items-center justify-between p-3 bg-blue-50 border border-blue-200 rounded-lg">
                <div className="flex items-center gap-3">
                  <Shield className="h-4 w-4 text-blue-600" />
                  <div>
                    <p className="text-sm font-medium">Admin Permissions Verified</p>
                    <p className="text-xs text-gray-600">Full admin access granted</p>
                  </div>
                </div>
                <span className="text-xs text-gray-500">
                  {session ? formatDuration(session.connected_at) + ' ago' : 'Now'}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}