/**
 * Progressive Authentication Gate for Admin Frontend
 * Controls access to admin features based on authentication level and permissions
 */
'use client';

import { Wallet, Shield, Lock, AlertTriangle, Crown } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useState } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useAdminProgressiveAuth } from '@/hooks/useAdminProgressiveAuth';
import { AuthLevel, type AuthGateProps } from '@/types/progressive-auth';

/**
 *
 * @param root0
 * @param root0.requiredLevel
 * @param root0.requiredPermissions
 * @param root0.children
 * @param root0.fallback
 * @param root0.authMessage
 * @param root0.showUpgradePrompts
 * @param root0.actionName
 * @param root0.loading
 */
export function AdminProgressiveAuthGate({
  requiredLevel,
  requiredPermissions = [],
  children,
  fallback,
  authMessage,
  showUpgradePrompts = true,
  actionName,
  loading
}: AuthGateProps) {
  const router = useRouter();
  const [isRedirecting, setIsRedirecting] = useState(false);
  
  const {
    level: currentLevel,
    walletAddress,
    isAuthenticated,
    isWalletConnected,
    adminPermissions,
    adminLevel,
    canAccess,
    getAuthMessage,
    getUpgradeAction,
    hasAnyPermission
  } = useAdminProgressiveAuth();

  const hasAccess = canAccess(requiredLevel, requiredPermissions);

  // Show loading state
  if (loading && (!isWalletConnected || !isAuthenticated)) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500"></div>
        <span className="ml-3 text-sm text-gray-600 dark:text-gray-400">
          Checking admin authentication...
        </span>
      </div>
    );
  }

  // User has access
  if (hasAccess) {
    return <>{children}</>;
  }

  // Show custom fallback if provided
  if (fallback) {
    return <>{fallback}</>;
  }

  // Show upgrade prompts
  if (!showUpgradePrompts) {
    return null;
  }

  const message = authMessage || getAuthMessage(requiredLevel, actionName);
  const upgradeAction = getUpgradeAction(requiredLevel);

  const handleUpgrade = async () => {
    setIsRedirecting(true);
    
    try {
      if (upgradeAction === 'connect' || upgradeAction === 'connect_and_authenticate') {
        // Redirect to Web3 login with current URL as return path
        const currentPath = window.location.pathname + window.location.search;
        const loginUrl = `/login?return_url=${encodeURIComponent(currentPath)}&reason=no-permission`;
        router.push(loginUrl);
      } else if (upgradeAction === 'authenticate') {
        // Try to authenticate if wallet is connected
        if (isWalletConnected) {
          // Redirect to permissions page to complete authentication
          router.push('/permissions/web3');
        } else {
          router.push('/login');
        }
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to upgrade auth:', _error);
      setIsRedirecting(false);
    }
  };

  const getIcon = () => {
    switch (requiredLevel) {
      case AuthLevel.CONNECTED:
        return <Wallet className="h-5 w-5 text-blue-500" />;
      case AuthLevel.AUTHENTICATED:
        return <Shield className="h-5 w-5 text-orange-500" />;
      default:
        return <Lock className="h-5 w-5 text-gray-500" />;
    }
  };

  const getStatusIcon = () => {
    switch (currentLevel) {
      case AuthLevel.AUTHENTICATED:
        return <div className="w-3 h-3 bg-green-500 rounded-full" />;
      case AuthLevel.CONNECTED:
        return <div className="w-3 h-3 bg-blue-500 rounded-full" />;
      default:
        return <div className="w-3 h-3 bg-gray-400 rounded-full" />;
    }
  };

  const getUpgradeButtonText = () => {
    switch (upgradeAction) {
      case 'connect':
        return '🔗 Connect Admin Wallet';
      case 'connect_and_authenticate':
        return '🔗 Connect & Authenticate';
      case 'authenticate':
        return '🔐 Authenticate Wallet';
      default:
        return '🔓 Upgrade Access';
    }
  };

  return (
    <div className="flex items-center justify-center min-h-[400px] p-6">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="flex items-center justify-center mb-4">
            {getIcon()}
            <Crown className="h-5 w-5 text-yellow-500 ml-2" />
          </div>
          <CardTitle className="text-lg">Admin Access Required</CardTitle>
        </CardHeader>
        
        <CardContent className="space-y-6">
          {/* Current Status */}
          <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div className="flex items-center gap-2">
              {getStatusIcon()}
              <span className="text-sm font-medium">
                {currentLevel === AuthLevel.AUTHENTICATED && 'Authenticated Admin'}
                {currentLevel === AuthLevel.CONNECTED && 'Wallet Connected'}
                {currentLevel === AuthLevel.PUBLIC && 'Not Connected'}
              </span>
            </div>
            {walletAddress && (
              <span className="text-xs font-mono text-gray-500">
                {walletAddress.slice(0, 6)}...{walletAddress.slice(-4)}
              </span>
            )}
          </div>

          {/* Permission Status */}
          {requiredPermissions.length > 0 && (
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription className="text-sm">
                Required permissions: {requiredPermissions.join(', ')}
                {adminPermissions.length > 0 && (
                  <div className="mt-2">
                    Your permissions: {adminPermissions.slice(0, 3).join(', ')}
                    {adminPermissions.length > 3 && ` +${adminPermissions.length - 3} more`}
                  </div>
                )}
              </AlertDescription>
            </Alert>
          )}

          {/* Auth Message */}
          <div className="text-center">
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              {message}
            </p>
            
            {adminLevel && (
              <p className="text-xs text-gray-500 dark:text-gray-500 mb-4">
                Current admin level: {adminLevel}
              </p>
            )}
          </div>

          {/* Upgrade Button */}
          <Button
            onClick={handleUpgrade}
            disabled={isRedirecting}
            className="w-full"
            variant={requiredLevel === AuthLevel.AUTHENTICATED ? 'default' : 'outline'}
          >
            {isRedirecting ? (
              <span className="flex items-center justify-center">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
                Redirecting...
              </span>
            ) : (
              getUpgradeButtonText()
            )}
          </Button>

          {/* Help Text */}
          <div className="text-center">
            <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
              Only wallets with admin permissions can access this feature. 
              Contact your system administrator if you need elevated access.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}