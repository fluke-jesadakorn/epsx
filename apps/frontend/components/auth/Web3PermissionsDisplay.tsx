'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  Crown, 
  Zap, 
  Users, 
  Shield, 
  Clock, 
  ExternalLink,
  AlertTriangle,
  CheckCircle,
  Info
} from 'lucide-react';
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';
import { getPermissionIcon, getPermissionBadgeColor, isPermissionExpired } from '@/lib/auth/web3';
import { formatDistanceToNow } from 'date-fns';

interface Web3PermissionsDisplayProps {
  variant?: 'compact' | 'detailed' | 'card';
  showHeader?: boolean;
  className?: string;
}

export function Web3PermissionsDisplay({ 
  variant = 'detailed', 
  showHeader = true,
  className = '' 
}: Web3PermissionsDisplayProps) {
  const { 
    isAuthenticated, 
    permissions, 
    userTier, 
    hasApiAccess,
    walletAddress 
  } = useWeb3AuthContext();

  if (!isAuthenticated || !walletAddress) {
    return (
      <Card className={className}>
        <CardContent className="p-6">
          <div className="flex flex-col items-center gap-3 text-center">
            <Shield className="h-12 w-12 text-slate-400" />
            <h3 className="font-semibold text-slate-900 dark:text-slate-100">
              Connect Your Wallet
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Connect your Web3 wallet to view your permissions and access levels.
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const getTierInfo = () => {
    switch (userTier) {
      case 'nft':
        return {
          icon: <Crown className="h-5 w-5 text-purple-500" />,
          title: 'NFT Holder',
          description: 'Enhanced access through NFT ownership',
          color: 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300'
        };
      case 'token':
        return {
          icon: <Zap className="h-5 w-5 text-orange-500" />,
          title: 'Token Holder',
          description: 'Token-gated premium features and benefits',
          color: 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300'
        };
      case 'dao':
        return {
          icon: <Users className="h-5 w-5 text-blue-500" />,
          title: 'DAO Member',
          description: 'Governance access and voting rights',
          color: 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300'
        };
      case 'enterprise':
        return {
          icon: <Shield className="h-5 w-5 text-green-500" />,
          title: 'Enterprise',
          description: 'Full API access and team management',
          color: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300'
        };
      default:
        return {
          icon: <Shield className="h-5 w-5 text-slate-500" />,
          title: 'Basic User',
          description: 'Standard platform access',
          color: 'bg-slate-100 text-slate-800 dark:bg-slate-900/20 dark:text-slate-300'
        };
    }
  };

  const tierInfo = getTierInfo();
  const activePermissions = permissions.filter(p => !isPermissionExpired(p));
  const expiredPermissions = permissions.filter(p => isPermissionExpired(p));

  // Compact variant for navigation or small spaces
  if (variant === 'compact') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <Badge className={tierInfo.color}>
          {tierInfo.icon}
          <span className="ml-1 capitalize">{userTier}</span>
        </Badge>
        {activePermissions.length > 0 && (
          <div className="flex items-center gap-1">
            {activePermissions.slice(0, 3).map((permission, index) => (
              <span key={index} className="text-sm" title={permission.permission}>
                {getPermissionIcon(permission.source)}
              </span>
            ))}
            {activePermissions.length > 3 && (
              <span className="text-xs text-slate-500 bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded">
                +{activePermissions.length - 3}
              </span>
            )}
          </div>
        )}
        {hasApiAccess && (
          <Badge variant="secondary" className="text-xs">
            API
          </Badge>
        )}
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
              Web3 Permissions
            </CardTitle>
          </CardHeader>
        )}
        <CardContent className="space-y-6">
          {/* User Tier */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-slate-100 dark:bg-slate-800 rounded-lg">
                {tierInfo.icon}
              </div>
              <div>
                <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                  {tierInfo.title}
                </h3>
                <p className="text-sm text-slate-600 dark:text-slate-400">
                  {tierInfo.description}
                </p>
              </div>
            </div>
            {hasApiAccess && (
              <Badge className="bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300">
                API Access
              </Badge>
            )}
          </div>

          {/* Active Permissions */}
          {activePermissions.length > 0 && (
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-slate-900 dark:text-slate-100">
                Active Permissions ({activePermissions.length})
              </h4>
              <div className="grid grid-cols-1 gap-2">
                {activePermissions.slice(0, 6).map((permission, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800 rounded-lg"
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-lg">{getPermissionIcon(permission.source)}</span>
                      <div>
                        <p className="font-medium text-slate-900 dark:text-slate-100">
                          {permission.permission.split(':').pop()?.replace('_', ' ')}
                        </p>
                        <p className="text-xs text-slate-600 dark:text-slate-400 capitalize">
                          {permission.source} permission
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge className={getPermissionBadgeColor(permission.source)}>
                        {permission.source}
                      </Badge>
                      {permission.expires_at && (
                        <div className="flex items-center gap-1 text-xs text-slate-500">
                          <Clock className="h-3 w-3" />
                          <span>
                            Expires {formatDistanceToNow(new Date(permission.expires_at), { addSuffix: true })}
                          </span>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
              {activePermissions.length > 6 && (
                <p className="text-sm text-center text-slate-600 dark:text-slate-400">
                  And {activePermissions.length - 6} more permissions...
                </p>
              )}
            </div>
          )}

          {/* Expired Permissions Warning */}
          {expiredPermissions.length > 0 && (
            <div className="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
              <div className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400" />
                <span className="text-sm font-medium text-amber-800 dark:text-amber-300">
                  {expiredPermissions.length} expired permission(s)
                </span>
              </div>
            </div>
          )}

          {/* No Permissions */}
          {activePermissions.length === 0 && (
            <div className="text-center py-6">
              <Info className="h-8 w-8 text-slate-400 mx-auto mb-2" />
              <p className="text-sm text-slate-600 dark:text-slate-400">
                No active permissions found. Connect with different wallets or acquire NFTs/tokens for enhanced access.
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    );
  }

  // Detailed variant for profile pages
  return (
    <div className={`space-y-6 ${className}`}>
      {/* Tier Status */}
      <div className="flex items-center justify-between p-6 bg-gradient-to-r from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 rounded-xl border">
        <div className="flex items-center gap-4">
          <div className="p-3 bg-white dark:bg-slate-800 rounded-xl shadow-sm">
            {tierInfo.icon}
          </div>
          <div>
            <h2 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {tierInfo.title}
            </h2>
            <p className="text-slate-600 dark:text-slate-400">
              {tierInfo.description}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          {hasApiAccess && (
            <Badge className="bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300 px-3 py-1">
              <Shield className="h-3 w-3 mr-1" />
              API Access
            </Badge>
          )}
          <Badge className={`${tierInfo.color} px-3 py-1`}>
            {tierInfo.icon}
            <span className="ml-2 font-medium capitalize">{userTier} Tier</span>
          </Badge>
        </div>
      </div>

      {/* Permission Categories */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {/* NFT Permissions */}
        {activePermissions.filter(p => p.source === 'nft').length > 0 && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm">
                <Crown className="h-4 w-4 text-purple-500" />
                NFT Permissions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {activePermissions.filter(p => p.source === 'nft').map((permission, index) => (
                <div key={index} className="flex items-center justify-between p-2 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                  <span className="text-sm font-medium text-purple-900 dark:text-purple-100">
                    {permission.permission.split(':').pop()}
                  </span>
                  {permission.metadata?.nft_collection && (
                    <Button variant="ghost" size="sm" className="h-6 w-6 p-0">
                      <ExternalLink className="h-3 w-3" />
                    </Button>
                  )}
                </div>
              ))}
            </CardContent>
          </Card>
        )}

        {/* Token Permissions */}
        {activePermissions.filter(p => p.source === 'token').length > 0 && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm">
                <Zap className="h-4 w-4 text-orange-500" />
                Token Permissions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {activePermissions.filter(p => p.source === 'token').map((permission, index) => (
                <div key={index} className="flex items-center justify-between p-2 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                  <span className="text-sm font-medium text-orange-900 dark:text-orange-100">
                    {permission.permission.split(':').pop()}
                  </span>
                  {permission.metadata?.required_amount && (
                    <span className="text-xs text-orange-600 dark:text-orange-400">
                      {permission.metadata.required_amount}
                    </span>
                  )}
                </div>
              ))}
            </CardContent>
          </Card>
        )}

        {/* DAO Permissions */}
        {activePermissions.filter(p => p.source === 'dao').length > 0 && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm">
                <Users className="h-4 w-4 text-blue-500" />
                DAO Permissions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {activePermissions.filter(p => p.source === 'dao').map((permission, index) => (
                <div key={index} className="flex items-center justify-between p-2 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <span className="text-sm font-medium text-blue-900 dark:text-blue-100">
                    {permission.permission.split(':').pop()}
                  </span>
                  {permission.metadata?.dao_name && (
                    <span className="text-xs text-blue-600 dark:text-blue-400">
                      {permission.metadata.dao_name}
                    </span>
                  )}
                </div>
              ))}
            </CardContent>
          </Card>
        )}

        {/* Manual Permissions */}
        {activePermissions.filter(p => p.source === 'manual').length > 0 && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm">
                <CheckCircle className="h-4 w-4 text-green-500" />
                Manual Permissions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {activePermissions.filter(p => p.source === 'manual').map((permission, index) => (
                <div key={index} className="flex items-center justify-between p-2 bg-green-50 dark:bg-green-900/20 rounded-lg">
                  <span className="text-sm font-medium text-green-900 dark:text-green-100">
                    {permission.permission.split(':').pop()}
                  </span>
                  {permission.expires_at && (
                    <div className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
                      <Clock className="h-3 w-3" />
                      <span>
                        {formatDistanceToNow(new Date(permission.expires_at), { addSuffix: true })}
                      </span>
                    </div>
                  )}
                </div>
              ))}
            </CardContent>
          </Card>
        )}
      </div>

      {/* Permission Summary */}
      <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-medium text-slate-900 dark:text-slate-100">
              Permission Summary
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              {activePermissions.length} active, {expiredPermissions.length} expired
            </p>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-lg">🎨</span>
            <span className="text-xs">{activePermissions.filter(p => p.source === 'nft').length}</span>
            <span className="text-lg">🪙</span>
            <span className="text-xs">{activePermissions.filter(p => p.source === 'token').length}</span>
            <span className="text-lg">🗳️</span>
            <span className="text-xs">{activePermissions.filter(p => p.source === 'dao').length}</span>
            <span className="text-lg">👤</span>
            <span className="text-xs">{activePermissions.filter(p => p.source === 'manual').length}</span>
          </div>
        </div>
      </div>
    </div>
  );
}