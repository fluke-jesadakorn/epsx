/**
 * Progressive Authentication Gate Component
 * Enforces authentication levels and provides upgrade prompts
 */
'use client';

import { useState } from 'react';
import { useProgressiveAuth } from '@/hooks/useProgressiveAuth';
import { WalletConnectAuth } from './WalletConnectAuth';
import { AuthLevel, type AuthGateProps } from '@/types/progressive-auth';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Wallet, Shield, Lock, Eye } from 'lucide-react';

export function ProgressiveAuthGate({
  requiredLevel,
  children,
  fallback,
  authMessage,
  showUpgradePrompts = true,
  actionName,
  loading
}: AuthGateProps) {
  const auth = useProgressiveAuth();
  const [showConnectModal, setShowConnectModal] = useState(false);

  // Show loading state if auth is still initializing
  if (!auth && loading) {
    return <>{loading}</>;
  }

  // User has sufficient auth level - show content
  if (auth.canAccess(requiredLevel)) {
    return <>{children}</>;
  }

  // User doesn't have sufficient auth - show upgrade prompt or fallback
  if (fallback) {
    return <>{fallback}</>;
  }

  if (!showUpgradePrompts) {
    return null;
  }

  // Generate appropriate messaging
  const message = authMessage || auth.getAuthMessage(requiredLevel, actionName);
  const upgradeAction = auth.getUpgradeAction(requiredLevel);

  // Get appropriate icon for auth level
  const getAuthIcon = (level: AuthLevel) => {
    switch (level) {
      case AuthLevel.PUBLIC:
        return <Eye className="h-6 w-6" />;
      case AuthLevel.CONNECTED:
        return <Wallet className="h-6 w-6" />;
      case AuthLevel.AUTHENTICATED:
        return <Shield className="h-6 w-6" />;
      default:
        return <Lock className="h-6 w-6" />;
    }
  };

  // Get level display name
  const getLevelName = (level: AuthLevel) => {
    switch (level) {
      case AuthLevel.PUBLIC:
        return 'Public Access';
      case AuthLevel.CONNECTED:
        return 'Wallet Connected';
      case AuthLevel.AUTHENTICATED:
        return 'Authenticated Access';
      default:
        return 'Authentication Required';
    }
  };

  // Get level description
  const getLevelDescription = (level: AuthLevel) => {
    switch (level) {
      case AuthLevel.PUBLIC:
        return 'No authentication needed';
      case AuthLevel.CONNECTED:
        return 'Connect your wallet for personalized features';
      case AuthLevel.AUTHENTICATED:
        return 'Sign in with your wallet for full access';
      default:
        return 'Authentication required for this feature';
    }
  };

  return (
    <div className="flex items-center justify-center min-h-[300px] p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="flex justify-center mb-4">
            <div className="p-3 bg-orange-100 dark:bg-orange-900/20 rounded-full">
              {getAuthIcon(requiredLevel)}
            </div>
          </div>
          <CardTitle className="text-xl">
            {getLevelName(requiredLevel)} Required
          </CardTitle>
          <CardDescription>
            {getLevelDescription(requiredLevel)}
          </CardDescription>
        </CardHeader>
        
        <CardContent className="space-y-4">
          <p className="text-sm text-center text-slate-600 dark:text-slate-400">
            {message}
          </p>

          {/* Show current auth status */}
          <div className="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
            <div className="flex items-center gap-2 text-sm">
              <div className="flex items-center gap-2">
                {getAuthIcon(auth.level)}
                <span className="font-medium">Current: {getLevelName(auth.level)}</span>
              </div>
            </div>
            {auth.walletAddress && (
              <p className="text-xs text-slate-500 mt-1">
                Wallet: {auth.walletAddress.slice(0, 6)}...{auth.walletAddress.slice(-4)}
              </p>
            )}
          </div>

          {/* Upgrade action buttons */}
          <div className="space-y-3">
            {upgradeAction === 'connect' && (
              <WalletConnectAuth 
                className="w-full"
                onAuthSuccess={() => {
                  // Component will re-render when auth state changes
                }}
              />
            )}

            {upgradeAction === 'sign' && (
              <WalletConnectAuth 
                className="w-full"
                onAuthSuccess={() => {
                  // Component will re-render when auth state changes
                }}
              />
            )}

            {upgradeAction === 'connect_and_sign' && (
              <div className="space-y-2">
                <p className="text-xs text-center text-slate-500">
                  First connect your wallet, then sign in
                </p>
                <WalletConnectAuth 
                  className="w-full"
                  onAuthSuccess={() => {
                    // Component will re-render when auth state changes
                  }}
                />
              </div>
            )}
          </div>

          {/* Show benefits of upgrading */}
          {requiredLevel === AuthLevel.CONNECTED && (
            <div className="text-xs text-center text-slate-500">
              <p>🎯 Save preferences • 📊 Personalized data • ⚡ Faster experience</p>
            </div>
          )}

          {requiredLevel === AuthLevel.AUTHENTICATED && (
            <div className="text-xs text-center text-slate-500">
              <p>💳 Payments • ⚙️ Settings • 🔑 API Access • 🎁 Premium Features</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}