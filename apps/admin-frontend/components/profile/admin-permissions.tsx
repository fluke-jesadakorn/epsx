'use client';

import { AlertCircle, ChevronDown, ChevronRight, Clock, Eye, Shield } from 'lucide-react';
import { useState } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { type User } from '@/shared/types/auth';

interface AdminPermissionsProps {
  user: User;
}

interface PermissionCategory {
  name: string;
  permissions: string[];
  color: string;
  icon: string;
}

/**
 *
 * @param root0
 * @param root0.user
 */
// eslint-disable-next-line max-lines-per-function
export function AdminPermissions({ user }: AdminPermissionsProps) {
  // Define helper functions before use
  const getColorForPlatform = (platform: string): string => {
    switch (platform) {
      case 'admin': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
      case 'epsx': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
      case 'epsx-pay': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
      case 'epsx-token': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
      default: return 'bg-slate-100 text-slate-800 dark:bg-slate-900/60 dark:text-slate-300';
    }
  };

  const getIconForPlatform = (platform: string): string => {
    switch (platform) {
      case 'admin': return '👑';
      case 'epsx': return '📊';
      case 'epsx-pay': return '💳';
      case 'epsx-token': return '🪙';
      default: return '🔐';
    }
  };

  const [searchTerm, setSearchTerm] = useState('');
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set(['admin']));
  const [filterType, setFilterType] = useState<'all' | 'admin' | 'platform'>('all');

  const formatDate = (timestamp?: number) => {
    if (timestamp === undefined || timestamp === 0) { return 'Not available'; }
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const categorizePermissions = (): PermissionCategory[] => {
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const permissions = user.permissions ?? [];
    const categories = new Map<string, string[]>();

    permissions.forEach(permission => {
      const parts = permission.split(':');
       
      const platform = parts[0] ?? 'unknown';

      if (!categories.has(platform)) {
        categories.set(platform, []);
      }
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      categories.get(platform)!.push(permission);
    });

    return Array.from(categories.entries()).map(([platform, perms]) => ({
      name: platform,
      permissions: perms.sort(),
      color: getColorForPlatform(platform),
      icon: getIconForPlatform(platform),
    }));
  };

  const toggleCategory = (categoryName: string) => {
    const newExpanded = new Set(expandedCategories);
    if (newExpanded.has(categoryName)) {
      newExpanded.delete(categoryName);
    } else {
      newExpanded.add(categoryName);
    }
    setExpandedCategories(newExpanded);
  };

  const filteredCategories = categorizePermissions()
    .filter(category => {
      if (filterType === 'admin' && category.name !== 'admin') { return false; }
      if (filterType === 'platform' && category.name === 'admin') { return false; }
      return true;
    })
    .map(category => ({
      ...category,
      permissions: category.permissions.filter(permission =>
        permission.toLowerCase().includes(searchTerm.toLowerCase())
      ),
    }))
    .filter(category => category.permissions.length > 0);

  const getPermissionLevel = (permission: string): string => {
    if (permission.includes(':admin') || permission.includes(':manage')) { return 'high'; }
    if (permission.includes(':write') || permission.includes(':create') || permission.includes(':update')) { return 'medium'; }
    return 'low';
  };

  const getPermissionLevelColor = (level: string): string => {
    switch (level) {
      case 'high': return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300';
      case 'medium': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300';
      default: return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    }
  };

  const hasTemporaryPermissions = user.permissions.some(p => /:\d+$/.test(p));

  return (
    <div className="space-y-6">
      {/* Permission Overview */}
      { }
      <Card className="border-yellow-200 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-orange-500" />
            Permission Overview
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="text-center p-4 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
              <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                {user.permissions.filter(p => p.startsWith('admin:')).length}
              </div>
              <div className="text-sm text-purple-700 dark:text-purple-300">
                Admin Permissions
              </div>
            </div>
            <div className="text-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
              <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                {user.permissions.filter(p => p.startsWith('epsx:')).length}
              </div>
              <div className="text-sm text-blue-700 dark:text-blue-300">
                Platform Permissions
              </div>
            </div>
            <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                {user.permissions.filter(p => p.includes(':manage') || p.includes(':admin')).length}
              </div>
              <div className="text-sm text-green-700 dark:text-green-300">
                Management Rights
              </div>
            </div>
            <div className="text-center p-4 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
              <div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
                {user.permissions.length}
              </div>
              <div className="text-sm text-orange-700 dark:text-orange-300">
                Total Permissions
              </div>
            </div>
          </div>

          {user.permission_last_updated !== undefined && user.permission_last_updated !== 0 && (
            <div className="flex items-center gap-2 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <Clock className="h-4 w-4 text-slate-500" />
              <span className="text-sm text-slate-600 dark:text-slate-400">
                Last updated: {formatDate(user.permission_last_updated)}
              </span>
              {user.permission_version !== undefined && user.permission_version !== 0 && (
                <Badge variant="outline" className="text-xs ml-auto">
                  v{user.permission_version}
                </Badge>
              )}
            </div>
          )}

          {hasTemporaryPermissions && (
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                You have temporary permissions that will expire automatically.
                Check individual permissions below for expiry times.
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Permission Search and Filter */}
      <Card className="border-yellow-200 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Eye className="h-5 w-5 text-orange-500" />
            Permission Details
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-4">
            <div className="flex-1">
              <Input
                placeholder="Search permissions..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full"
              />
            </div>
            <div className="flex gap-2">
              <Button
                variant={filterType === 'all' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterType('all')}
              >
                All
              </Button>
              <Button
                variant={filterType === 'admin' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterType('admin')}
              >
                Admin
              </Button>
              <Button
                variant={filterType === 'platform' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterType('platform')}
              >
                Platform
              </Button>
            </div>
          </div>

          {/* Permission Categories */}
          <div className="space-y-4">
            {filteredCategories.map((category) => (
              <div key={category.name} className="border border-slate-200 dark:border-slate-700 rounded-lg">
                <button
                  onClick={() => toggleCategory(category.name)}
                  className="w-full flex items-center justify-between p-4 hover:bg-slate-50 dark:hover:bg-gray-100 dark:bg-slate-800 rounded-t-lg"
                >
                  <div className="flex items-center gap-3">
                    <span className="text-lg">{category.icon}</span>
                    <div className="text-left">
                      <div className="font-medium text-slate-900 dark:text-slate-100">
                        {category.name.toUpperCase()} Platform
                      </div>
                      <div className="text-sm text-slate-600 dark:text-slate-400">
                        {category.permissions.length} permissions
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge className={category.color}>
                      {category.permissions.length}
                    </Badge>
                    {expandedCategories.has(category.name) ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                  </div>
                </button>

                {expandedCategories.has(category.name) && (
                  <div className="p-4 border-t border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50">
                    <div className="grid grid-cols-1 gap-2">
                      {category.permissions.map((permission, index) => {
                        const level = getPermissionLevel(permission);
                        const isTemporary = /:\d+$/.test(permission);
                        const parts = permission.split(':');
                        const basePermission = isTemporary ? parts.slice(0, -1).join(':') : permission;
                        const lastPart = parts[parts.length - 1] ?? '';
                        const expiryTimestamp = (isTemporary && lastPart !== '') ? parseInt(lastPart, 10) : undefined;

                        return (
                          <div
                            // eslint-disable-next-line react/no-array-index-key
                            key={index}
                            className="flex items-center justify-between p-3 bg-white dark:bg-slate-800 rounded-lg border border-slate-200 dark:border-slate-700"
                          >
                            <div className="flex-1">
                              <div className="font-mono text-sm text-slate-900 dark:text-slate-100">
                                {basePermission}
                              </div>
                              {isTemporary === true && expiryTimestamp !== undefined && expiryTimestamp !== 0 && (
                                <div className="text-xs text-orange-600 dark:text-orange-400">
                                  Expires: {formatDate(expiryTimestamp)}
                                </div>
                              )}
                            </div>
                            <div className="flex items-center gap-2">
                              {isTemporary && (
                                <Badge variant="outline" className="text-orange-600 border-orange-600 text-xs">
                                  Temporary
                                </Badge>
                              )}
                              <Badge className={`${getPermissionLevelColor(level)} text-xs`}>
                                {level.toUpperCase()}
                              </Badge>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                )}
              </div>
            ))}

            {filteredCategories.length === 0 && (
              <div className="text-center py-8 text-slate-500 dark:text-slate-400">
                {searchTerm ? 'No permissions found matching your search.' : 'No permissions in this category.'}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}