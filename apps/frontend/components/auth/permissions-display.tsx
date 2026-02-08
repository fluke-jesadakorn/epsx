'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { 
  UserWalletDisplay, 
  UserTierBadge, 
  UserAuthStatus, 
  UserPermissionsDisplay 
} from '@/shared/components/display/user-display';
import { Shield } from 'lucide-react';

interface PermissionsDisplayProps {
  variant?: 'compact' | 'detailed' | 'card';
  showHeader?: boolean;
  className?: string;
}

export function PermissionsDisplay({
  variant = 'detailed',
  showHeader = true,
  className = ''
}: PermissionsDisplayProps) {
  const { isAuthenticated, user } = useSharedAuth();

  if (!isAuthenticated || !user) {
    return (
      <Card className={className}>
        <CardContent className="p-6">
          <div className="flex flex-col items-center gap-3 text-center">
            <Shield className="h-12 w-12 text-slate-400" />
            <h3 className="font-semibold text-slate-900 dark:text-slate-100">
              Connect Your Wallet
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Connect your wallet to view your permissions and access levels.
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  // All logic is now handled by shared components

  // Compact variant for navigation or small spaces
  if (variant === 'compact') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <UserTierBadge />
        <UserAuthStatus />
      </div>
    );
  }

  // Card variant for standalone display
  if (variant === 'card') {
    return (
      <Card className={className}>
        {showHeader && (
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-orange-500" />
              Permissions
            </CardTitle>
          </CardHeader>
        )}
        <CardContent className="space-y-6">
          {/* User Information */}
          <div className="flex items-center justify-between">
            <UserWalletDisplay showFullAddress={false} />
            <UserTierBadge />
          </div>
          
          {/* Authentication Status */}
          <UserAuthStatus />
          
          {/* Permissions Display */}
          <UserPermissionsDisplay maxDisplay={10} />
        </CardContent>
      </Card>
    );
  }

  // Detailed variant for profile pages
  return (
    <div className={`space-y-6 ${className}`}>
      {/* User Status */}
      <div className="flex items-center justify-between p-6 bg-gradient-to-r from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 rounded-xl border">
        <UserWalletDisplay showFullAddress={true} className="text-xl" />
        <div className="flex items-center gap-3">
          <UserTierBadge className="px-3 py-1" />
          <UserAuthStatus />
        </div>
      </div>

      {/* Permissions Display */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-orange-500" />
            Your Permissions
          </CardTitle>
        </CardHeader>
        <CardContent>
          <UserPermissionsDisplay maxDisplay={50} />
        </CardContent>
      </Card>
    </div>
  );
}