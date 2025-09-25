/**
 * Permission Management - Main Interface
 * Consolidates: AdminPermissionDashboard, CrossPlatformPermissionManager, 
 * DynamicPermissionAssignmentDashboard, PermissionsOverview, PermissionsTable,
 * RBACPermissionManager, PermissionBuilder, PermissionTemplateManager
 */

'use client';

import { useState, useMemo } from 'react';
import { Search, Filter, Download, Users, Shield, Clock, Activity } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { User, Permission, PermissionAnalytics, Platform } from '@/types/core';
import type { TableColumn } from '@/types/ui';
import { adminClient } from '@/lib/api/unified-admin-client';
import { UnifiedAuth } from '@/lib/auth/unified-auth';

interface PermissionManagementProps {
  users?: User[];
  currentUser?: User;
  onPermissionChange?: (userId: string, permissions: string[]) => void;
  onBulkAction?: (action: string, items: any[]) => void;
  className?: string;
}

interface PermissionTableItem {
  id: string;
  user: User;
  permission: string;
  platform: Platform;
  resource: string;
  action: string;
  expiresAt?: string;
  assignedBy: string;
  assignedAt: string;
  status: 'active' | 'expiring' | 'expired';
}

function PermissionCard({ user, isSelected, onSelect }: {
  user: User;
  isSelected: boolean;
  onSelect: (userId: string) => void;
}) {
  const getUserPermissionStats = () => {
    if (!user.permissions || !Array.isArray(user.permissions)) {
      return { total: 0, adminPerms: 0, expiring: 0 };
    }
    
    const total = user.permissions.length;
    const adminPerms = user.permissions.filter(p => 
      typeof p === 'string' && p.includes('admin')
    ).length;
    const expiring = 0; // Would need timestamp parsing for real expiring perms
    
    return { total, adminPerms, expiring };
  };

  const stats = getUserPermissionStats();
  
  const getRoleIcon = (role: string) => {
    switch (role) {
      case 'admin': return '👑'
      case 'premium': return '⭐'
      case 'user': return '👤'
      default: return '🙂'
    }
  };

  const getStatusColor = (status?: string) => {
    switch (status) {
      case 'active': return 'bg-gradient-to-r from-green-500 to-emerald-500'
      case 'inactive': return 'bg-gradient-to-r from-gray-500 to-slate-500'
      case 'suspended': return 'bg-gradient-to-r from-red-500 to-pink-500'
      default: return 'bg-gradient-to-r from-blue-500 to-cyan-500'
    }
  };

  return (
    <div className={`relative overflow-hidden rounded-2xl bg-gradient-to-r from-gray-100/50 to-gray-200/50 dark:from-gray-800/50 dark:to-gray-700/50 p-0.5 group   ${isSelected ? 'ring-2 ring-yellow-400' : ''}`}>
      <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-sm rounded-2xl p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3 flex-1 min-w-0">
            <div className="relative">
              <input
                type="checkbox"
                checked={isSelected}
                onChange={() => onSelect(user.id)}
                className="h-4 w-4 rounded border-gray-300 text-yellow-600 focus:ring-yellow-500 mt-1"
              />
              <div className="absolute -top-1 -right-1 text-lg">
                {getRoleIcon(user.role || 'user')}
              </div>
            </div>
            
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <h4 className="font-semibold text-sm truncate text-gray-800 dark:text-gray-200">
                  {user.displayName || user.name || user.email}
                </h4>
                <div className={`px-2 py-0.5 rounded-full text-xs font-medium text-white ${getStatusColor(user.status || 'active')}`}>
                  {user.status || 'active'}
                </div>
              </div>
              
              <p className="text-sm text-gray-600 dark:text-gray-400 truncate mb-3">
                {user.email}
              </p>
              
              <div className="grid grid-cols-3 gap-2 mb-3">
                <div className="text-center">
                  <div className="text-lg font-bold text-blue-600">{stats.total}</div>
                  <div className="text-xs text-gray-500">Total</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-bold text-purple-600">{stats.adminPerms}</div>
                  <div className="text-xs text-gray-500">Admin</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-bold text-orange-600">{stats.expiring}</div>
                  <div className="text-xs text-gray-500">Expiring</div>
                </div>
              </div>

              <div className="flex flex-wrap gap-1 max-h-12 overflow-hidden">
                {user.permissions && Array.isArray(user.permissions) ? (
                  user.permissions.slice(0, 3).map((perm, idx) => (
                    <span key={idx} className="inline-block px-2 py-0.5 bg-gradient-to-r from-purple-400/20 to-pink-400/20 text-xs rounded-full border border-purple-200 dark:border-purple-700">
                      {typeof perm === 'string' ? perm.split(':').slice(-1)[0] : 'permission'}
                    </span>
                  ))
                ) : (
                  <span className="text-xs text-gray-500">No permissions</span>
                )}
                {user.permissions && user.permissions.length > 3 && (
                  <span className="inline-block px-2 py-0.5 bg-gray-200 dark:bg-gray-700 text-xs rounded-full">
                    +{user.permissions.length - 3}
                  </span>
                )}
              </div>
            </div>
          </div>
          
          <div className="flex flex-col gap-1">
            <a 
              href={`/permissions/grant?user=${user.id}`}
              className="px-3 py-1 bg-gradient-to-r from-green-400 to-emerald-500 hover:from-green-500 hover:to-emerald-600 text-white text-xs rounded-full  "
            >
              Grant
            </a>
            <a 
              href={`/users/${user.id}/edit`}
              className="px-3 py-1 bg-gradient-to-r from-blue-400 to-cyan-500 hover:from-blue-500 hover:to-cyan-600 text-white text-xs rounded-full  "
            >
              Edit
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}

export function PermissionManagement({
  users = [],
  currentUser,
  onPermissionChange,
  onBulkAction,
  className = ''
}: PermissionManagementProps) {
  // State management
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedPlatform, setSelectedPlatform] = useState<string>('all');
  const [selectedStatus, setSelectedStatus] = useState<string>('all');
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Calculate real stats from user data
  const permissionStats = useMemo(() => {
    // Ensure users is an array
    const userArray = Array.isArray(users) ? users : [];
    
    const allPermissions = userArray.flatMap(user => user.permissions || []);
    const totalPermissions = allPermissions.length;
    const uniqueUsers = userArray.filter(user => user.permissions && user.permissions.length > 0).length;
    const adminUsers = userArray.filter(user => 
      user.permissions && user.permissions.some(p => 
        typeof p === 'string' && p.includes('admin')
      )
    ).length;
    const premiumUsers = userArray.filter(user => user.role === 'premium_user').length;
    
    return {
      totalPermissions,
      uniqueUsers, 
      adminUsers,
      premiumUsers,
      avgPermissions: uniqueUsers > 0 ? Math.round(totalPermissions / uniqueUsers) : 0
    };
  }, [users]);

  // Filter users based on search and filters
  const filteredUsers = useMemo(() => {
    // Ensure users is an array
    const userArray = Array.isArray(users) ? users : [];
    
    return userArray.filter(user => {
      const matchesSearch = searchQuery === '' || 
        (user.email && user.email.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (user.name && user.name.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (user.displayName && user.displayName.toLowerCase().includes(searchQuery.toLowerCase()));
      
      const matchesStatus = selectedStatus === 'all' || 
        (user.status || 'active') === selectedStatus;
      
      return matchesSearch && matchesStatus;
    });
  }, [users, searchQuery, selectedStatus]);

  // Event handlers
  const handleUserSelect = (userId: string) => {
    setSelectedUsers(prev => 
      prev.includes(userId) 
        ? prev.filter(id => id !== userId)
        : [...prev, userId]
    )
  }

  const handleRefresh = () => {
    setIsLoading(true)
    setTimeout(() => setIsLoading(false), 1000)
  }

  return (
    <div className="space-y-6 sm:space-y-8">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl "></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg  "></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl  "></div>
      </div>

      <div className="relative">
        {/* Page Header */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              🔐 Permission Center
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full "></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Manage user permissions and access control across the EPSX platform
          </p>
        </div>

        {/* Stats Dashboard */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50 hover:shadow-2xl ">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">🔐</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-purple-600">{permissionStats.totalPermissions}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Permissions</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">{permissionStats.avgPermissions} avg/user</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50 hover:shadow-2xl ">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">👥</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Users</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-blue-600">{permissionStats.uniqueUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">With Perms</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Active accounts</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50 hover:shadow-2xl ">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">👑</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Admin</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-orange-600">{permissionStats.adminUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Admin Users</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Elevated access</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50 hover:shadow-2xl ">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">⭐</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Premium</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-green-600">{permissionStats.premiumUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Premium</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Enhanced perms</div>
            </div>
          </div>
        </div>

        {/* Search and Filters */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5 mb-6 sm:mb-8">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-purple-400 to-blue-500 rounded-full blur-sm  opacity-60"></div>
            
            <div className="p-4 sm:p-6 space-y-4">
              <div className="flex flex-col sm:flex-row gap-3 sm:gap-4">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                  <Input
                    placeholder="🔍 Search users by name or email..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-10 rounded-2xl border-2 border-purple-200 dark:border-purple-700 bg-gradient-to-r from-purple-50/50 to-blue-50/50 dark:from-purple-900/20 dark:to-blue-900/20"
                  />
                </div>
                
                <div className="flex gap-2 sm:gap-3">
                  <Select value={selectedPlatform} onValueChange={setSelectedPlatform}>
                    <SelectTrigger className="w-32 sm:w-40 rounded-2xl border-2 border-blue-200 dark:border-blue-700">
                      <SelectValue placeholder="Platform" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Platforms</SelectItem>
                      <SelectItem value="admin">🔧 Admin</SelectItem>
                      <SelectItem value="epsx">📊 EPSX</SelectItem>
                      <SelectItem value="premium">⭐ Premium</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  <Select value={selectedStatus} onValueChange={setSelectedStatus}>
                    <SelectTrigger className="w-32 sm:w-40 rounded-2xl border-2 border-green-200 dark:border-green-700">
                      <SelectValue placeholder="Status" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Status</SelectItem>
                      <SelectItem value="active">✅ Active</SelectItem>
                      <SelectItem value="inactive">❌ Inactive</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Bulk Actions */}
        {selectedUsers.length > 0 && (
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-yellow-400/20 to-orange-500/20 p-0.5 mb-4">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-4">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">
                  🎯 {selectedUsers.length} user(s) selected
                </span>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    onClick={() => window.open(`/permissions/grant?users=${selectedUsers.join(',')}&operation=bulk`, '_blank')}
                    className="bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600"
                  >
                    Bulk Grant
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setSelectedUsers([])}
                  >
                    Clear
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Users Grid */}
        <div className="space-y-3 sm:space-y-4">
          {filteredUsers.length === 0 && !isLoading ? (
            <div className="text-center py-12 sm:py-16">
              <div className="text-6xl sm:text-8xl mb-4">🤷‍♂️</div>
              <h3 className="text-xl sm:text-2xl font-bold text-gray-700 dark:text-gray-300 mb-2">No users found</h3>
              <p className="text-gray-500 dark:text-gray-400">Try adjusting your search or filters</p>
            </div>
          ) : isLoading ? (
            <div className="text-center py-12 sm:py-16">
              <div className="text-6xl sm:text-8xl mb-4 ">⏳</div>
              <p className="text-gray-500 dark:text-gray-400">Loading permissions...</p>
            </div>
          ) : (
            <>
              {/* Mobile: Card Layout */}
              <div className="block sm:hidden space-y-3">
                {filteredUsers.map(user => (
                  <PermissionCard 
                    key={user.id}
                    user={user}
                    isSelected={selectedUsers.includes(user.id)}
                    onSelect={handleUserSelect}
                  />
                ))}
              </div>

              {/* Desktop: Card Grid */}
              <div className="hidden sm:grid sm:grid-cols-1 lg:grid-cols-2 gap-4">
                {filteredUsers.map(user => (
                  <PermissionCard 
                    key={user.id}
                    user={user}
                    isSelected={selectedUsers.includes(user.id)}
                    onSelect={handleUserSelect}
                  />
                ))}
              </div>
            </>
          )}
        </div>

        {/* Quick Actions Footer */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-blue-400/20 p-0.5 mt-6 sm:mt-8">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-purple-400 to-pink-500 rounded-full blur-sm  opacity-60"></div>
            
            <div className="p-4 sm:p-6">
              <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-600 via-pink-600 to-blue-600 bg-clip-text text-transparent mb-4">
                ⚡ Permission Actions
              </h3>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <a href="/permissions/grant" className="block">
                  <div className="text-center p-3 sm:p-4 bg-gradient-to-r from-green-400/10 to-emerald-400/10 hover:from-green-400/20 hover:to-emerald-400/20 border border-green-200 dark:border-green-700 rounded-2xl  ">
                    <div className="text-2xl sm:text-3xl mb-2">✅</div>
                    <div className="text-sm font-medium">Grant Perms</div>
                  </div>
                </a>
                <a href="/permissions/request" className="block">
                  <div className="text-center p-3 sm:p-4 bg-gradient-to-r from-blue-400/10 to-cyan-400/10 hover:from-blue-400/20 hover:to-cyan-400/20 border border-blue-200 dark:border-blue-700 rounded-2xl  ">
                    <div className="text-2xl sm:text-3xl mb-2">📝</div>
                    <div className="text-sm font-medium">Request</div>
                  </div>
                </a>
                <a href="/users" className="block">
                  <div className="text-center p-3 sm:p-4 bg-gradient-to-r from-purple-400/10 to-pink-400/10 hover:from-purple-400/20 hover:to-pink-400/20 border border-purple-200 dark:border-purple-700 rounded-2xl  ">
                    <div className="text-2xl sm:text-3xl mb-2">👥</div>
                    <div className="text-sm font-medium">Users</div>
                  </div>
                </a>
                <div onClick={handleRefresh} className="cursor-pointer">
                  <div className="text-center p-3 sm:p-4 bg-gradient-to-r from-orange-400/10 to-yellow-400/10 hover:from-orange-400/20 hover:to-yellow-400/20 border border-orange-200 dark:border-orange-700 rounded-2xl  ">
                    <div className={`text-2xl sm:text-3xl mb-2 ${isLoading ? 'animate-spin' : ''}`}>🔄</div>
                    <div className="text-sm font-medium">Refresh</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Helper functions
function convertToCSV(data: any[]): string {
  if (!data.length) return '';
  
  const headers = Object.keys(data[0]);
  const csvContent = [
    headers.join(','),
    ...data.map(row => 
      headers.map(header => JSON.stringify(row[header] || '')).join(',')
    )
  ].join('\n');
  
  return csvContent;
}

function downloadCSV(content: string, filename: string): void {
  const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
  const link = document.createElement('a');
  const url = URL.createObjectURL(blob);
  link.setAttribute('href', url);
  link.setAttribute('download', filename);
  link.style.visibility = 'hidden';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}

export default PermissionManagement;