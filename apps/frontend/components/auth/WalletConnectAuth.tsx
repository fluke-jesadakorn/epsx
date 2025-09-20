'use client';

import { ConnectButton } from '@rainbow-me/rainbowkit';
import { Wallet, LogOut, Shield, AlertCircle, Crown, Zap, Users } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useWeb3Auth, getPermissionIcon, formatAddress, getTierDescription } from '@/lib/auth/web3';
import { useRouter } from 'next/navigation';

interface WalletConnectAuthProps {
  onAuthSuccess?: (walletAddress: string) => void;
  onAuthError?: (error: string) => void;
  className?: string;
  variant?: 'default' | 'compact' | 'detailed';
}

export function WalletConnectAuth({ 
  onAuthSuccess, 
  onAuthError, 
  className = '',
  variant = 'default'
}: WalletConnectAuthProps) {
  const router = useRouter();
  const {
    isConnected,
    isAuthenticated,
    isAuthenticating,
    walletAddress,
    permissions,
    userTier,
    hasApiAccess,
    error,
    authenticate,
    disconnect,
  } = useWeb3Auth();

  // Handle auth success callback
  if (isAuthenticated && walletAddress && onAuthSuccess) {
    onAuthSuccess(walletAddress);
  }

  // Handle auth error callback
  if (error && onAuthError) {
    onAuthError(error);
  }

  const getTierIcon = () => {
    switch (userTier) {
      case 'nft': return <Crown className="h-4 w-4 text-purple-500" />;
      case 'token': return <Zap className="h-4 w-4 text-orange-500" />;
      case 'dao': return <Users className="h-4 w-4 text-blue-500" />;
      case 'enterprise': return <Shield className="h-4 w-4 text-green-500" />;
      default: return <Wallet className="h-4 w-4 text-slate-500" />;
    }
  };

  const getTierBadgeColor = () => {
    switch (userTier) {
      case 'nft': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
      case 'token': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
      case 'dao': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
      case 'enterprise': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
      default: return 'bg-slate-100 text-slate-800 dark:bg-slate-900/20 dark:text-slate-300';
    }
  };

  // Compact variant for navigation
  if (variant === 'compact') {
    if (!isConnected) {
      return (
        <ConnectButton.Custom>
          {({ openConnectModal }) => (
            <Button
              onClick={openConnectModal}
              size="sm"
              className="flex items-center gap-2 bg-gradient-to-r from-orange-500 to-purple-600 text-white hover:from-orange-600 hover:to-purple-700"
            >
              <Wallet className="h-4 w-4" />
              Connect
            </Button>
          )}
        </ConnectButton.Custom>
      );
    }

    if (!isAuthenticated) {
      return (
        <div className="flex items-center gap-2">
          <Button
            onClick={authenticate}
            disabled={isAuthenticating}
            size="sm"
            className="bg-orange-500 text-white hover:bg-orange-600"
          >
            <Shield className="h-4 w-4 mr-1" />
            {isAuthenticating ? 'Signing...' : 'Sign In'}
          </Button>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        <Badge className={getTierBadgeColor()}>
          {getTierIcon()}
          <span className="ml-1 capitalize">{userTier}</span>
        </Badge>
        <Button
          onClick={disconnect}
          variant="ghost"
          size="sm"
          className="p-1"
        >
          <LogOut className="h-4 w-4" />
        </Button>
      </div>
    );
  }

  // Default variant for main authentication
  if (!isConnected) {
    return (
      <div className={`flex flex-col items-center gap-3 ${className}`}>
        <ConnectButton.Custom>
          {({ openConnectModal }) => (
            <Button
              onClick={openConnectModal}
              className="flex items-center gap-3 bg-gradient-to-r from-orange-500 to-purple-600 text-white hover:from-orange-600 hover:to-purple-700 px-6 py-3 text-base font-semibold rounded-xl shadow-lg"
            >
              <Wallet className="h-5 w-5" />
              Connect Web3 Wallet
            </Button>
          )}
        </ConnectButton.Custom>
        <p className="text-xs text-slate-600 dark:text-slate-400 text-center">
          Connect your wallet to access Web3 features and permissions
        </p>
      </div>
    );
  }

  // Connected but not authenticated
  if (!isAuthenticated) {
    return (
      <div className={`flex flex-col gap-4 ${className}`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Wallet className="h-5 w-5 text-orange-500" />
            <span className="font-medium text-slate-900 dark:text-slate-100">
              {formatAddress(walletAddress!)}
            </span>
          </div>
          <ConnectButton.Custom>
            {({ openAccountModal }) => (
              <Button variant="outline" size="sm" onClick={openAccountModal}>
                Manage
              </Button>
            )}
          </ConnectButton.Custom>
        </div>

        <div className="flex flex-col gap-3">
          <Button
            onClick={authenticate}
            disabled={isAuthenticating}
            className="flex items-center gap-3 bg-gradient-to-r from-orange-500 to-purple-600 text-white hover:from-orange-600 hover:to-purple-700 px-6 py-3 text-base font-semibold rounded-xl"
          >
            <Shield className="h-5 w-5" />
            {isAuthenticating ? 'Signing Message...' : 'Sign In with Wallet'}
          </Button>
          
          {error && (
            <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
              <AlertCircle className="h-4 w-4 text-red-600 dark:text-red-400" />
              <span className="text-sm text-red-700 dark:text-red-400">{error}</span>
            </div>
          )}
        </div>

        <div className="text-center">
          <p className="text-xs text-slate-600 dark:text-slate-400">
            Sign a message to prove wallet ownership and access your Web3 permissions
          </p>
        </div>
      </div>
    );
  }

  // Authenticated - show detailed user info
  if (variant === 'detailed') {
    return (
      <div className={`flex flex-col gap-4 ${className}`}>
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-100 dark:bg-green-900/20 rounded-lg">
              <Wallet className="h-5 w-5 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                {formatAddress(walletAddress!)}
              </h3>
              <p className="text-sm text-slate-600 dark:text-slate-400">
                Web3 Authenticated
              </p>
            </div>
          </div>
          <Button
            onClick={disconnect}
            variant="outline"
            size="sm"
            className="text-red-600 hover:text-red-700 hover:bg-red-50"
          >
            <LogOut className="h-4 w-4 mr-1" />
            Disconnect
          </Button>
        </div>

        {/* User Tier */}
        <div className="flex items-center gap-2">
          <Badge className={`${getTierBadgeColor()} px-3 py-1`}>
            {getTierIcon()}
            <span className="ml-2 font-medium capitalize">{userTier} Tier</span>
          </Badge>
          {hasApiAccess && (
            <Badge className="bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300">
              API Access
            </Badge>
          )}
        </div>

        {/* Permissions */}
        {permissions.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium text-slate-900 dark:text-slate-100">
              Active Permissions ({permissions.length})
            </h4>
            <div className="grid grid-cols-2 gap-2">
              {permissions.slice(0, 4).map((permission, index) => (
                <div
                  key={index}
                  className="flex items-center gap-2 p-2 bg-slate-50 dark:bg-slate-800 rounded-lg"
                >
                  <span className="text-sm">{getPermissionIcon(permission.source)}</span>
                  <span className="text-xs font-medium text-slate-700 dark:text-slate-300 truncate">
                    {permission.permission.split(':').pop()}
                  </span>
                </div>
              ))}
            </div>
            {permissions.length > 4 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => router.push('/profile')}
                className="w-full text-slate-600 hover:text-slate-900"
              >
                View all {permissions.length} permissions
              </Button>
            )}
          </div>
        )}

        {/* Quick Actions */}
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => router.push('/profile')}
            className="flex-1"
          >
            View Profile
          </Button>
          {hasApiAccess && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => router.push('/profile?tab=api')}
              className="flex-1"
            >
              API Keys
            </Button>
          )}
        </div>
      </div>
    );
  }

  // Default authenticated view
  return (
    <div className={`flex items-center gap-3 ${className}`}>
      {/* Wallet Info */}
      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-green-100 dark:bg-green-900/20 rounded-lg">
          <Wallet className="h-4 w-4 text-green-600 dark:text-green-400" />
        </div>
        <span className="text-sm font-medium text-slate-900 dark:text-slate-100">
          {formatAddress(walletAddress!)}
        </span>
      </div>

      {/* User Tier Badge */}
      <Badge className={getTierBadgeColor()}>
        {getTierIcon()}
        <span className="ml-1 capitalize">{userTier}</span>
      </Badge>

      {/* Permissions Indicator */}
      {permissions.length > 0 && (
        <div className="flex items-center gap-1">
          {permissions.slice(0, 3).map((permission, index) => (
            <span
              key={index}
              title={`${permission.permission} (${permission.source})`}
              className="text-sm"
            >
              {getPermissionIcon(permission.source)}
            </span>
          ))}
          {permissions.length > 3 && (
            <span className="text-xs text-slate-500 bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded">
              +{permissions.length - 3}
            </span>
          )}
        </div>
      )}

      {/* Disconnect Button */}
      <Button
        onClick={disconnect}
        variant="ghost"
        size="sm"
        className="px-2 py-2 hover:text-red-600"
        title="Disconnect Wallet"
      >
        <LogOut className="h-4 w-4" />
      </Button>
    </div>
  );
}