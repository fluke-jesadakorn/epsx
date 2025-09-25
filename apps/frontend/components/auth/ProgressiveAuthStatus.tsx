/**
 * Progressive Authentication Status Component
 * Shows current auth level in navigation with upgrade prompts
 */
'use client';

import { useProgressiveAuth } from '@/hooks/useProgressiveAuth';
import { WalletConnectAuth } from './WalletConnectAuth';
import { AuthLevel, AuthLevelType } from '@/types/progressive-auth';
import { Badge } from '@/components/ui/badge';
import { Eye, Wallet, Shield, ChevronRight } from 'lucide-react';

interface ProgressiveAuthStatusProps {
  /**
   * Compact mode for navigation bar
   */
  compact?: boolean;
  
  /**
   * Show upgrade hints
   */
  showUpgradeHints?: boolean;
  
  /**
   * Custom className
   */
  className?: string;
}

export function ProgressiveAuthStatus({ 
  compact = false, 
  showUpgradeHints = true,
  className = ''
}: ProgressiveAuthStatusProps) {
  const auth = useProgressiveAuth();

  // Get auth level info
  const getAuthLevelInfo = (level: AuthLevelType) => {
    switch (level) {
      case AuthLevel.PUBLIC:
        return {
          icon: <Eye className="h-4 w-4" />,
          label: 'Public',
          color: 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400',
          description: 'Browse freely'
        };
      case AuthLevel.CONNECTED:
        return {
          icon: <Wallet className="h-4 w-4" />,
          label: 'Connected',
          color: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
          description: 'Wallet connected'
        };
      case AuthLevel.AUTHENTICATED:
        return {
          icon: <Shield className="h-4 w-4" />,
          label: 'Authenticated',
          color: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
          description: 'Fully signed in'
        };
      default:
        return {
          icon: <Eye className="h-4 w-4" />,
          label: 'Unknown',
          color: 'bg-slate-100 text-slate-600',
          description: 'Unknown state'
        };
    }
  };

  const levelInfo = getAuthLevelInfo(auth.level);

  // Compact mode for navigation bar
  if (compact) {
    return (
      <div className={`flex items-center gap-3 ${className}`}>
        {/* Wallet connection component only - no status badges */}
        <WalletConnectAuth 
          className="text-sm" 
          preferredLevel={auth.level}
          compact={true}
        />
      </div>
    );
  }

  // Full mode for standalone display
  return (
    <div className={`space-y-3 ${className}`}>
      {/* Current status */}
      <div className="flex items-center gap-3">
        <div className={`flex items-center gap-2 px-3 py-2 rounded-lg ${levelInfo.color}`}>
          {levelInfo.icon}
          <div>
            <div className="font-medium text-sm">{levelInfo.label}</div>
            <div className="text-xs opacity-75">{levelInfo.description}</div>
          </div>
        </div>
        
        {auth.walletAddress && (
          <div className="text-xs text-slate-500">
            {auth.walletAddress.slice(0, 6)}...{auth.walletAddress.slice(-4)}
          </div>
        )}
      </div>

      {/* Wallet connection */}
      <WalletConnectAuth 
        preferredLevel={auth.level}
        compact={false}
      />

      {/* Upgrade hints */}
      {showUpgradeHints && (
        <div className="space-y-2">
          {auth.level === AuthLevel.PUBLIC && (
            <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
              <div className="flex items-start gap-2">
                <Wallet className="h-4 w-4 text-blue-500 mt-0.5" />
                <div>
                  <div className="text-sm font-medium text-blue-700 dark:text-blue-400">
                    Connect Your Wallet
                  </div>
                  <div className="text-xs text-blue-600 dark:text-blue-500">
                    Save preferences • Personalized data • Faster experience
                  </div>
                </div>
              </div>
            </div>
          )}

          {auth.level === AuthLevel.CONNECTED && (
            <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <div className="flex items-start gap-2">
                <Shield className="h-4 w-4 text-green-500 mt-0.5" />
                <div>
                  <div className="text-sm font-medium text-green-700 dark:text-green-400">
                    Sign In for Full Access
                  </div>
                  <div className="text-xs text-green-600 dark:text-green-500">
                    Payments • Settings • API Access • Premium Features
                  </div>
                </div>
              </div>
            </div>
          )}

          {auth.level === AuthLevel.AUTHENTICATED && (
            <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <div className="flex items-start gap-2">
                <Shield className="h-4 w-4 text-green-500 mt-0.5" />
                <div>
                  <div className="text-sm font-medium text-green-700 dark:text-green-400">
                    Full Access Active
                  </div>
                  <div className="text-xs text-green-600 dark:text-green-500">
                    You have access to all features and sensitive operations
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}