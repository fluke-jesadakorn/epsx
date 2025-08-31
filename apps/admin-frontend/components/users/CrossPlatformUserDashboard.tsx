'use client';

import { useState } from 'react';
import { Globe, Coins, Vote, Users, Activity, Clock, Shield, ExternalLink } from 'lucide-react';

interface Platform {
  id: string;
  code: string;
  name: string;
  description: string;
  baseUrl?: string;
}

interface PlatformAccess {
  platform: Platform;
  accessLevel: 'read' | 'write' | 'admin';
  permissions: string[];
  lastAccess?: Date;
  isActive: boolean;
}

interface UserCompanySummary {
  primaryPlatform: Platform;
  accessiblePlatforms: PlatformAccess[];
  crossPlatformActivity: ActivityLog[];
  platformSwitchHistory: PlatformSwitchLog[];
  permissionAuditTrail: PermissionAuditLog[];
  tokenBalances: { [platformId: string]: TokenBalance };
}

interface ActivityLog {
  id: string;
  platform: string;
  action: string;
  resource: string;
  timestamp: Date;
  ipAddress?: string;
  userAgent?: string;
}

interface PlatformSwitchLog {
  id: string;
  fromPlatform: string;
  toPlatform: string;
  timestamp: Date;
  sessionId: string;
}

interface PermissionAuditLog {
  id: string;
  platform: string;
  permission: string;
  action: 'granted' | 'revoked' | 'modified';
  grantedBy: string;
  timestamp: Date;
  reason?: string;
}

interface TokenBalance {
  platform: string;
  balance: number;
  symbol: string;
  usdValue?: number;
}

interface CrossPlatformUserDashboardProps {
  userId: string;
  userSummary: UserCompanySummary;
  onSwitchToPlatform: (platformCode: string) => void;
  onUpdatePermission: (platformId: string, permission: string, action: 'grant' | 'revoke') => void;
}

const platformIcons: { [key: string]: typeof Globe } = {
  'epsx': Globe,
  'epsx-pay': Coins,
  'epsx-token': Vote,
};

const platformColors: { [key: string]: string } = {
  'epsx': 'border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-900/20',
  'epsx-pay': 'border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-900/20',
  'epsx-token': 'border-purple-200 bg-purple-50 dark:border-purple-800 dark:bg-purple-900/20',
};

const accessLevelColors: { [key: string]: string } = {
  'read': 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300',
  'write': 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400',
  'admin': 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400',
};

export function CrossPlatformUserDashboard({
  userId,
  userSummary,
  onSwitchToPlatform,
  onUpdatePermission,
}: CrossPlatformUserDashboardProps) {
  const [activeTab, setActiveTab] = useState<'overview' | 'permissions' | 'activity' | 'tokens'>('overview');

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  const getTotalPermissions = () => {
    return userSummary.accessiblePlatforms.reduce((total, access) => 
      total + access.permissions.length, 0
    );
  };

  const getTotalTokenValue = () => {
    return Object.values(userSummary.tokenBalances).reduce((total, balance) => 
      total + (balance.usdValue || 0), 0
    );
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg">
      {/* Header */}
      <div className="p-6 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
              Cross-Platform User Dashboard
            </h2>
            <p className="text-gray-600 dark:text-gray-400">
              Unified view across EPSX ecosystem
            </p>
          </div>
          <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
            <div className="flex items-center gap-1">
              <Globe className="h-4 w-4" />
              <span>{userSummary.accessiblePlatforms.length} platforms</span>
            </div>
            <div className="flex items-center gap-1">
              <Shield className="h-4 w-4" />
              <span>{getTotalPermissions()} permissions</span>
            </div>
            <div className="flex items-center gap-1">
              <Activity className="h-4 w-4" />
              <span>{userSummary.crossPlatformActivity.length} activities</span>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-gray-700">
        <nav className="flex space-x-8 px-6">
          {[
            { id: 'overview', label: 'Overview', icon: Globe },
            { id: 'permissions', label: 'Permissions', icon: Shield },
            { id: 'activity', label: 'Activity', icon: Activity },
            { id: 'tokens', label: 'Tokens', icon: Coins },
          ].map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id as any)}
              className={`flex items-center gap-2 py-4 px-1 border-b-2 font-medium text-sm transition-colors ${
                activeTab === id
                  ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                  : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
              }`}
            >
              <Icon className="h-4 w-4" />
              {label}
            </button>
          ))}
        </nav>
      </div>

      {/* Content */}
      <div className="p-6">
        {activeTab === 'overview' && (
          <div className="space-y-6">
            {/* Platform Access Cards */}
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
                Platform Access
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {userSummary.accessiblePlatforms.map((access) => {
                  const Icon = platformIcons[access.platform.code] || Globe;
                  const colorClass = platformColors[access.platform.code] || 'border-gray-200 bg-gray-50';
                  const isPrimary = access.platform.id === userSummary.primaryPlatform.id;
                  
                  return (
                    <div
                      key={access.platform.id}
                      className={`border rounded-lg p-4 ${colorClass} ${
                        isPrimary ? 'ring-2 ring-blue-500' : ''
                      }`}
                    >
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center gap-2">
                          <Icon className="h-5 w-5" />
                          <h4 className="font-medium text-gray-900 dark:text-white">
                            {access.platform.name}
                          </h4>
                        </div>
                        {isPrimary && (
                          <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs font-medium rounded">
                            Primary
                          </span>
                        )}
                      </div>
                      
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                        {access.platform.description}
                      </p>
                      
                      <div className="flex items-center justify-between">
                        <span className={`px-2 py-1 rounded text-xs font-medium ${accessLevelColors[access.accessLevel]}`}>
                          {access.accessLevel}
                        </span>
                        <button
                          onClick={() => onSwitchToPlatform(access.platform.code)}
                          className="flex items-center gap-1 text-xs text-blue-600 dark:text-blue-400 hover:underline"
                        >
                          Switch to
                          <ExternalLink className="h-3 w-3" />
                        </button>
                      </div>
                      
                      <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
                        <div className="flex items-center justify-between text-xs">
                          <span className="text-gray-500">{access.permissions.length} permissions</span>
                          {access.lastAccess && (
                            <span className="text-gray-500">
                              {formatDate(access.lastAccess)}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Quick Stats */}
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
                Quick Stats
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Users className="h-4 w-4 text-blue-500" />
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      Total Platforms
                    </span>
                  </div>
                  <span className="text-2xl font-bold text-gray-900 dark:text-white">
                    {userSummary.accessiblePlatforms.length}
                  </span>
                </div>
                
                <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Shield className="h-4 w-4 text-green-500" />
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      Permissions
                    </span>
                  </div>
                  <span className="text-2xl font-bold text-gray-900 dark:text-white">
                    {getTotalPermissions()}
                  </span>
                </div>
                
                <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Activity className="h-4 w-4 text-orange-500" />
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      Activities
                    </span>
                  </div>
                  <span className="text-2xl font-bold text-gray-900 dark:text-white">
                    {userSummary.crossPlatformActivity.length}
                  </span>
                </div>
                
                <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Coins className="h-4 w-4 text-purple-500" />
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      Token Value
                    </span>
                  </div>
                  <span className="text-2xl font-bold text-gray-900 dark:text-white">
                    ${getTotalTokenValue().toLocaleString()}
                  </span>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'permissions' && (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
                Permission Details by Platform
              </h3>
              
              {userSummary.accessiblePlatforms.map((access) => (
                <div key={access.platform.id} className="mb-6">
                  <h4 className="font-medium text-gray-900 dark:text-white mb-3 flex items-center gap-2">
                    {React.createElement(platformIcons[access.platform.code] || Globe, { className: "h-4 w-4" })}
                    {access.platform.name}
                  </h4>
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
                    {access.permissions.map((permission, index) => {
                      const [platform, resource, action] = permission.split(':');
                      return (
                        <div
                          key={index}
                          className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-900 rounded"
                        >
                          <code className="text-sm font-mono">
                            {resource}:{action}
                          </code>
                        </div>
                      );
                    })}
                  </div>
                  
                  {access.permissions.length === 0 && (
                    <p className="text-gray-500 dark:text-gray-400 text-sm">
                      No specific permissions granted
                    </p>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'activity' && (
          <div className="space-y-4">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Recent Cross-Platform Activity
            </h3>
            
            <div className="space-y-3">
              {userSummary.crossPlatformActivity.map((activity) => {
                const Icon = platformIcons[activity.platform] || Globe;
                
                return (
                  <div
                    key={activity.id}
                    className="flex items-center gap-4 p-3 bg-gray-50 dark:bg-gray-900 rounded-lg"
                  >
                    <Icon className="h-5 w-5 text-gray-500" />
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-gray-900 dark:text-white">
                          {activity.action}
                        </span>
                        <span className="text-gray-500">on</span>
                        <code className="text-sm bg-gray-200 dark:bg-gray-700 px-1 rounded">
                          {activity.resource}
                        </code>
                        <span className="text-gray-500">in</span>
                        <span className="text-blue-600 dark:text-blue-400">
                          {activity.platform}
                        </span>
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">
                        {formatDate(activity.timestamp)}
                      </div>
                    </div>
                  </div>
                );
              })}
              
              {userSummary.crossPlatformActivity.length === 0 && (
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                  No recent activity
                </p>
              )}
            </div>
          </div>
        )}

        {activeTab === 'tokens' && (
          <div className="space-y-4">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Token Balances
            </h3>
            
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {Object.values(userSummary.tokenBalances).map((balance) => {
                const Icon = platformIcons[balance.platform] || Coins;
                
                return (
                  <div
                    key={balance.platform}
                    className="border border-gray-200 dark:border-gray-700 rounded-lg p-4"
                  >
                    <div className="flex items-center gap-2 mb-3">
                      <Icon className="h-5 w-5" />
                      <span className="font-medium text-gray-900 dark:text-white">
                        {balance.symbol}
                      </span>
                    </div>
                    
                    <div className="space-y-1">
                      <div className="text-2xl font-bold text-gray-900 dark:text-white">
                        {balance.balance.toLocaleString()}
                      </div>
                      {balance.usdValue && (
                        <div className="text-sm text-gray-500 dark:text-gray-400">
                          ${balance.usdValue.toLocaleString()} USD
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
            
            {Object.keys(userSummary.tokenBalances).length === 0 && (
              <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                No token balances
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}