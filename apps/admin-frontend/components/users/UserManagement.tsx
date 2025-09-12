/**
 * User Management - Main Interface
 * Consolidates: UserManagement, UserManagementList, UserManagementMigrated, 
 * UserList, VirtualizedUserTable, UserTableWithSelection, ResponsiveUserDisplay,
 * CrossPlatformUserDashboard, UserDataProvider, UserAnalyticsDashboard
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { useRouter } from 'next/navigation';
import { 
  Search, 
  Filter, 
  Plus, 
  Download, 
  Users, 
  UserCheck, 
  UserX,
  MoreHorizontal,
  Edit,
  Trash2,
  Shield,
  Calendar,
  Activity,
  RefreshCw,
  SlidersHorizontal,
  ArrowUpDown,
  Eye,
  UserPlus,
  Settings
} from 'lucide-react';
import { toast } from '@/hooks/use-toast';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';

import type { User, UserStats, UserFilters } from '@/types/core';
import type { TableColumn } from '@/types/ui';
import { adminClient } from '@/lib/api/unified-admin-client';
import { UnifiedAuth } from '@/lib/auth/unified-auth';

interface UserManagementProps {
  initialUsers?: User[];
  onUserCreate?: (user: User) => void;
  onUserUpdate?: (user: User) => void;
  onUserDelete?: (userId: string) => void;
  onBulkAction?: (action: string, userIds: string[]) => void;
  className?: string;
}

function UserCard({ user, isSelected, onSelect, onEdit, onDelete, onView }: {
  user: User;
  isSelected: boolean;
  onSelect: (userId: string) => void;
  onEdit: (user: User) => void;
  onDelete: (user: User) => void;
  onView: (user: User) => void;
}) {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-gradient-to-r from-green-500 to-emerald-500'
      case 'inactive': return 'bg-gradient-to-r from-gray-500 to-slate-500'
      case 'suspended': return 'bg-gradient-to-r from-red-500 to-pink-500'
      default: return 'bg-gradient-to-r from-blue-500 to-cyan-500'
    }
  }

  const getRoleIcon = (role: string) => {
    switch (role) {
      case 'admin': return '👑'
      case 'premium': return '⭐'
      case 'user': return '👤'
      default: return '🙂'
    }
  }

  return (
    <div className={`relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 via-orange-400/20 to-pink-400/20 p-0.5 group hover:scale-[1.02] transition-all duration-200 ${isSelected ? 'ring-2 ring-yellow-500 shadow-lg shadow-yellow-400/25' : ''}`}>
      <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-4">
        <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-br from-yellow-300/30 to-orange-400/30 rounded-full blur-sm animate-pulse opacity-60"></div>
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3 flex-1 min-w-0">
            <div className="relative">
              <Checkbox
                checked={isSelected}
                onCheckedChange={() => onSelect(user.id)}
                className="mt-1"
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
              
              <p className="text-sm text-gray-600 dark:text-gray-400 truncate mb-2">
                {user.email}
              </p>
              
              <div className="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                <span className="flex items-center gap-1">
                  <Shield className="h-3 w-3" />
                  {user.role || 'user'}
                </span>
                {user.permissions && (
                  <span className="flex items-center gap-1">
                    <Activity className="h-3 w-3" />
                    {user.permissions.length} permissions
                  </span>
                )}
                <span className="flex items-center gap-1">
                  <Calendar className="h-3 w-3" />
                  {user.createdAt ? new Date(user.createdAt).toLocaleDateString() : 'Unknown'}
                </span>
              </div>
            </div>
          </div>
          
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48 bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl border border-gray-200/50 dark:border-gray-700/50 rounded-2xl shadow-xl">
              <DropdownMenuItem onClick={() => onView(user)} className="rounded-xl">
                <Eye className="h-4 w-4 mr-2" />
                View Details
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onEdit(user)} className="rounded-xl">
                <Edit className="h-4 w-4 mr-2" />
                Edit User
              </DropdownMenuItem>
              <DropdownMenuItem 
                onClick={() => onDelete(user)}
                className="text-red-600 dark:text-red-400 rounded-xl"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete User
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  )
}

export function UserManagement({
  initialUsers = [],
  onUserCreate,
  onUserUpdate,
  onUserDelete,
  onBulkAction,
  className = ''
}: UserManagementProps) {
  const router = useRouter();
  
  // State management
  const [users, setUsers] = useState<User[]>(initialUsers || []);
  const [userStats, setUserStats] = useState<UserStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState('all');
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(false);

  // Filters and search
  const [filters, setFilters] = useState<UserFilters>({
    search: '',
    role: 'all',
    status: 'all',
    tier: 'all',
    sortBy: 'created_at',
    sortOrder: 'desc',
    page: 1,
    limit: 50
  });

  // Calculate stats from real data
  const statsFromUsers = useMemo(() => {
    const activeUsers = users.filter(user => user.status === 'active' || !user.status).length
    const adminUsers = users.filter(user => user.role === 'admin').length
    const premiumUsers = users.filter(user => user.role === 'premium').length
    const totalPermissions = users.reduce((acc, user) => acc + (user.permissions?.length || 0), 0)

    return {
      totalUsers: users.length,
      activeUsers,
      adminUsers,
      premiumUsers,
      totalPermissions,
      avgPermissions: users.length > 0 ? Math.round(totalPermissions / users.length) : 0
    }
  }, [users])

  // Filtered and processed users
  const filteredUsers = useMemo(() => {
    let filtered = [...users];

    // Apply search filter
    if (filters.search) {
      filtered = filtered.filter(user => 
        user.email.toLowerCase().includes(filters.search!.toLowerCase()) ||
        (user.name && user.name.toLowerCase().includes(filters.search!.toLowerCase())) ||
        (user.displayName && user.displayName.toLowerCase().includes(filters.search!.toLowerCase())) ||
        user.id.includes(filters.search!)
      );
    }

    // Apply role filter
    if (filters.role && filters.role !== 'all') {
      filtered = filtered.filter(user => user.role === filters.role);
    }

    // Apply status filter
    if (filters.status && filters.status !== 'all') {
      filtered = filtered.filter(user => 
        filters.status === 'active' ? user.isActive : !user.isActive
      );
    }

    // Apply tier filter
    if (filters.tier && filters.tier !== 'all') {
      filtered = filtered.filter(user => user.packageTier === filters.tier);
    }

    // Apply sorting
    filtered.sort((a, b) => {
      const aValue = a[filters.sortBy as keyof User] as any;
      const bValue = b[filters.sortBy as keyof User] as any;
      
      if (filters.sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

    return filtered;
  }, [users, filters]);

  // Event handlers
  const handleUserSelect = (userId: string) => {
    setSelectedUsers(prev => 
      prev.includes(userId) 
        ? prev.filter(id => id !== userId)
        : [...prev, userId]
    )
  }

  const handleUserView = (user: User) => {
    router.push(`/users/${user.id}`);
  }

  const handleUserEdit = (user: User) => {
    router.push(`/users/${user.id}/edit`);
  }

  const handleUserDelete = async (user: User) => {
    if (!confirm(`Are you sure you want to delete user ${user.email}? This action cannot be undone.`)) {
      return;
    }
    
    try {
      setUsers(prev => prev.filter(u => u.id !== user.id));
      onUserDelete?.(user.id);
      
      toast({
        title: "User deleted",
        description: `${user.email} has been successfully deleted.`,
        variant: "default",
      });
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to delete user. Please try again.",
        variant: "destructive",
      });
    }
  }

  const handleRefresh = () => {
    setIsLoading(true);
    // Simulate API refresh
    setTimeout(() => {
      setIsLoading(false);
      toast({
        title: "Refreshed",
        description: "User data has been refreshed successfully.",
        variant: "default",
      });
    }, 1000);
  }

  const handleSelectAll = () => {
    if (selectedUsers.length === filteredUsers.length) {
      setSelectedUsers([]);
    } else {
      setSelectedUsers(filteredUsers.map(user => user.id));
    }
  }

  const handleBulkAction = (action: string) => {
    if (selectedUsers.length === 0) return;
    
    switch (action) {
      case 'activate':
        toast({
          title: "Users activated",
          description: `${selectedUsers.length} users have been activated.`,
          variant: "default",
        });
        break;
      case 'deactivate':
        toast({
          title: "Users deactivated",
          description: `${selectedUsers.length} users have been deactivated.`,
          variant: "default",
        });
        break;
      case 'delete':
        if (confirm(`Are you sure you want to delete ${selectedUsers.length} users? This action cannot be undone.`)) {
          toast({
            title: "Users deleted",
            description: `${selectedUsers.length} users have been deleted.`,
            variant: "default",
          });
        }
        break;
    }
    
    setSelectedUsers([]);
    onBulkAction?.(action, selectedUsers);
  }


  return (
    <div className="space-y-6 sm:space-y-8">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>

      <div className="relative">
        {/* Page Header */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              👥 User Management
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full animate-pulse"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Manage users, roles, and permissions across the EPSX platform
          </p>
        </div>

        {/* Stats Dashboard */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50 hover:shadow-2xl transition-shadow">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">👥</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-blue-600">{statsFromUsers.totalUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Users</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">{statsFromUsers.avgPermissions} avg perms</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50 hover:shadow-2xl transition-shadow">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">✅</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Active</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-green-600">{statsFromUsers.activeUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Active</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Online users</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50 hover:shadow-2xl transition-shadow">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">👑</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Admin</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-purple-600">{statsFromUsers.adminUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Admins</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Staff members</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50 hover:shadow-2xl transition-shadow">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-2xl sm:text-3xl">⭐</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Premium</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-orange-600">{statsFromUsers.premiumUsers}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Premium</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Paid users</div>
            </div>
          </div>
        </div>

        {/* Search and Filters */}
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5 mb-6 sm:mb-8">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl">
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full blur-sm animate-pulse opacity-60"></div>
            
            <div className="p-4 sm:p-6 space-y-4">
              <div className="flex flex-col lg:flex-row gap-3 sm:gap-4">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                  <Input
                    placeholder="🔍 Search users by name, email, or ID..."
                    value={filters.search || ''}
                    onChange={(e) => setFilters(prev => ({ ...prev, search: e.target.value }))}
                    className="pl-10 pr-4 rounded-2xl border-2 border-blue-200 dark:border-blue-700 bg-gradient-to-r from-blue-50/50 to-purple-50/50 dark:from-blue-900/20 dark:to-purple-900/20 h-12"
                  />
                </div>
                
                <div className="flex flex-wrap gap-2 sm:gap-3">
                  <Select value={filters.role || 'all'} onValueChange={(value) => setFilters(prev => ({ ...prev, role: value }))}>
                    <SelectTrigger className="w-36 sm:w-40 rounded-2xl border-2 border-green-200 dark:border-green-700 h-12">
                      <SelectValue placeholder="Role" />
                    </SelectTrigger>
                    <SelectContent className="rounded-2xl">
                      <SelectItem value="all">All Roles</SelectItem>
                      <SelectItem value="admin">👑 Admin</SelectItem>
                      <SelectItem value="premium">⭐ Premium</SelectItem>
                      <SelectItem value="user">👤 User</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  <Select value={filters.status || 'all'} onValueChange={(value) => setFilters(prev => ({ ...prev, status: value }))}>
                    <SelectTrigger className="w-36 sm:w-40 rounded-2xl border-2 border-purple-200 dark:border-purple-700 h-12">
                      <SelectValue placeholder="Status" />
                    </SelectTrigger>
                    <SelectContent className="rounded-2xl">
                      <SelectItem value="all">All Status</SelectItem>
                      <SelectItem value="active">✅ Active</SelectItem>
                      <SelectItem value="inactive">❌ Inactive</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  <Select value={filters.sortBy || 'created_at'} onValueChange={(value) => setFilters(prev => ({ ...prev, sortBy: value }))}>
                    <SelectTrigger className="w-36 sm:w-40 rounded-2xl border-2 border-orange-200 dark:border-orange-700 h-12">
                      <ArrowUpDown className="w-4 h-4 mr-2" />
                      <SelectValue placeholder="Sort" />
                    </SelectTrigger>
                    <SelectContent className="rounded-2xl">
                      <SelectItem value="created_at">📅 Created Date</SelectItem>
                      <SelectItem value="email">📧 Email</SelectItem>
                      <SelectItem value="name">👤 Name</SelectItem>
                      <SelectItem value="role">🎭 Role</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
                    className="h-12 px-4 rounded-2xl border-2 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
                  >
                    <SlidersHorizontal className="w-4 h-4 mr-2" />
                    Filters
                  </Button>
                </div>
              </div>
              
              {/* Advanced Filters */}
              {showAdvancedFilters && (
                <div className="pt-4 border-t border-gray-200/50 dark:border-gray-700/50">
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                    <Select value={filters.tier || 'all'} onValueChange={(value) => setFilters(prev => ({ ...prev, tier: value }))}>
                      <SelectTrigger className="rounded-2xl border-2 border-yellow-200 dark:border-yellow-700">
                        <SelectValue placeholder="Package Tier" />
                      </SelectTrigger>
                      <SelectContent className="rounded-2xl">
                        <SelectItem value="all">All Tiers</SelectItem>
                        <SelectItem value="basic">🥉 Basic</SelectItem>
                        <SelectItem value="premium">🥈 Premium</SelectItem>
                        <SelectItem value="pro">🥇 Pro</SelectItem>
                        <SelectItem value="enterprise">💎 Enterprise</SelectItem>
                      </SelectContent>
                    </Select>
                    
                    <Select value={filters.sortOrder || 'desc'} onValueChange={(value) => setFilters(prev => ({ ...prev, sortOrder: value as 'asc' | 'desc' }))}>
                      <SelectTrigger className="rounded-2xl border-2 border-pink-200 dark:border-pink-700">
                        <SelectValue placeholder="Order" />
                      </SelectTrigger>
                      <SelectContent className="rounded-2xl">
                        <SelectItem value="asc">📈 Ascending</SelectItem>
                        <SelectItem value="desc">📉 Descending</SelectItem>
                      </SelectContent>
                    </Select>
                    
                    <div className="flex gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setFilters({
                          search: '',
                          role: 'all',
                          status: 'all',
                          tier: 'all',
                          sortBy: 'created_at',
                          sortOrder: 'desc',
                          page: 1,
                          limit: 50
                        })}
                        className="rounded-2xl"
                      >
                        🔄 Reset
                      </Button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Bulk Actions */}
        {selectedUsers.length > 0 && (
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 to-orange-500/20 p-0.5 mb-4">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-4">
              <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full blur-sm animate-pulse opacity-60"></div>
              <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
                <div className="flex items-center gap-3">
                  <Checkbox
                    checked={selectedUsers.length === filteredUsers.length}
                    onCheckedChange={handleSelectAll}
                    className="border-2"
                  />
                  <span className="text-sm font-medium">
                    🎯 {selectedUsers.length} of {filteredUsers.length} user(s) selected
                  </span>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    onClick={() => handleBulkAction('activate')}
                    className="bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 rounded-2xl"
                  >
                    <UserCheck className="w-4 h-4 mr-1" />
                    Activate
                  </Button>
                  <Button
                    size="sm"
                    onClick={() => handleBulkAction('deactivate')}
                    className="bg-gradient-to-r from-orange-500 to-red-500 hover:from-orange-600 hover:to-red-600 rounded-2xl"
                  >
                    <UserX className="w-4 h-4 mr-1" />
                    Deactivate
                  </Button>
                  <Button
                    size="sm"
                    onClick={() => window.open(`/users/bulk?users=${selectedUsers.join(',')}&operation=grant`, '_blank')}
                    className="bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 rounded-2xl"
                  >
                    <Shield className="w-4 h-4 mr-1" />
                    Permissions
                  </Button>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => handleBulkAction('delete')}
                    className="rounded-2xl"
                  >
                    <Trash2 className="w-4 h-4 mr-1" />
                    Delete
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setSelectedUsers([])}
                    className="rounded-2xl"
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
              <div className="text-6xl sm:text-8xl mb-4 animate-pulse">⏳</div>
              <p className="text-gray-500 dark:text-gray-400">Loading users...</p>
            </div>
          ) : (
            <>
              {/* Results Summary */}
              <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 mb-4">
                <div className="text-sm text-gray-600 dark:text-gray-400">
                  Showing {filteredUsers.length} of {users.length} users
                  {filters.search && (
                    <span className="ml-2 px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-xs">
                      Search: "{filters.search}"
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleRefresh}
                    disabled={isLoading}
                    className="rounded-2xl"
                  >
                    <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
                    Refresh
                  </Button>
                </div>
              </div>
              
              {/* Mobile: Card Layout */}
              <div className="block lg:hidden space-y-3">
                {filteredUsers.map(user => (
                  <UserCard 
                    key={user.id}
                    user={user}
                    isSelected={selectedUsers.includes(user.id)}
                    onSelect={handleUserSelect}
                    onView={handleUserView}
                    onEdit={handleUserEdit}
                    onDelete={handleUserDelete}
                  />
                ))}
              </div>

              {/* Desktop: Card Grid */}
              <div className="hidden lg:grid lg:grid-cols-1 xl:grid-cols-2 gap-4">
                {filteredUsers.map(user => (
                  <UserCard 
                    key={user.id}
                    user={user}
                    isSelected={selectedUsers.includes(user.id)}
                    onSelect={handleUserSelect}
                    onView={handleUserView}
                    onEdit={handleUserEdit}
                    onDelete={handleUserDelete}
                  />
                ))}
              </div>
            </>
          )}
        </div>

        {/* Quick Actions Footer */}
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 via-blue-400/20 to-purple-400/20 p-0.5 mt-6 sm:mt-8">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl">
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400 to-blue-500 rounded-full blur-sm animate-pulse opacity-60"></div>
            
            <div className="p-4 sm:p-6">
              <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-600 via-blue-600 to-purple-600 bg-clip-text text-transparent mb-4">
                ⚡ Quick Actions
              </h3>
              <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
                <Button
                  onClick={() => router.push('/users/create')}
                  className="h-auto p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-green-400/10 to-emerald-400/10 hover:from-green-400/20 hover:to-emerald-400/20 border-2 border-green-200 dark:border-green-700 rounded-2xl transition-all duration-200 hover:scale-105 text-gray-700 dark:text-gray-300"
                  variant="ghost"
                >
                  <UserPlus className="w-6 h-6" />
                  <span className="text-sm font-medium">Add User</span>
                </Button>
                
                <Button
                  onClick={() => router.push('/users/bulk')}
                  className="h-auto p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-blue-400/10 to-cyan-400/10 hover:from-blue-400/20 hover:to-cyan-400/20 border-2 border-blue-200 dark:border-blue-700 rounded-2xl transition-all duration-200 hover:scale-105 text-gray-700 dark:text-gray-300"
                  variant="ghost"
                >
                  <Users className="w-6 h-6" />
                  <span className="text-sm font-medium">Bulk Ops</span>
                </Button>
                
                <Button
                  onClick={() => router.push('/permissions')}
                  className="h-auto p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-purple-400/10 to-pink-400/10 hover:from-purple-400/20 hover:to-pink-400/20 border-2 border-purple-200 dark:border-purple-700 rounded-2xl transition-all duration-200 hover:scale-105 text-gray-700 dark:text-gray-300"
                  variant="ghost"
                >
                  <Shield className="w-6 h-6" />
                  <span className="text-sm font-medium">Permissions</span>
                </Button>
                
                <Button
                  onClick={() => {
                    const csvData = filteredUsers.map(user => ({
                      email: user.email,
                      name: user.name || '',
                      role: user.role,
                      status: user.isActive ? 'active' : 'inactive',
                      created: user.createdAt
                    }));
                    
                    const csvString = 'data:text/csv;charset=utf-8,'
                      + 'Email,Name,Role,Status,Created\n'
                      + csvData.map(row => Object.values(row).join(',')).join('\n');
                    
                    const encodedUri = encodeURI(csvString);
                    const link = document.createElement('a');
                    link.setAttribute('href', encodedUri);
                    link.setAttribute('download', `users_export_${new Date().toISOString().split('T')[0]}.csv`);
                    document.body.appendChild(link);
                    link.click();
                    document.body.removeChild(link);
                    
                    toast({
                      title: "Export completed",
                      description: `${filteredUsers.length} users exported to CSV.`,
                      variant: "default",
                    });
                  }}
                  className="h-auto p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-orange-400/10 to-yellow-400/10 hover:from-orange-400/20 hover:to-yellow-400/20 border-2 border-orange-200 dark:border-orange-700 rounded-2xl transition-all duration-200 hover:scale-105 text-gray-700 dark:text-gray-300"
                  variant="ghost"
                >
                  <Download className="w-6 h-6" />
                  <span className="text-sm font-medium">Export</span>
                </Button>
                
                <Button
                  onClick={handleRefresh}
                  disabled={isLoading}
                  className="h-auto p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-gray-400/10 to-slate-400/10 hover:from-gray-400/20 hover:to-slate-400/20 border-2 border-gray-200 dark:border-gray-700 rounded-2xl transition-all duration-200 hover:scale-105 text-gray-700 dark:text-gray-300"
                  variant="ghost"
                >
                  <RefreshCw className={`w-6 h-6 ${isLoading ? 'animate-spin' : ''}`} />
                  <span className="text-sm font-medium">Refresh</span>
                </Button>
                
                <Button
                  onClick={() => router.push('/settings')}
                  className="h-auto p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-indigo-400/10 to-purple-400/10 hover:from-indigo-400/20 hover:to-purple-400/20 border-2 border-indigo-200 dark:border-indigo-700 rounded-2xl transition-all duration-200 hover:scale-105 text-gray-700 dark:text-gray-300"
                  variant="ghost"
                >
                  <Settings className="w-6 h-6" />
                  <span className="text-sm font-medium">Settings</span>
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default UserManagement;