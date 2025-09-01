'use client';

import { useState, useEffect, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useRouter, useSearchParams } from 'next/navigation';
import { 
  Users, 
  UserCheck, 
  UserPlus, 
  UserX, 
  Search, 
  Filter,
  MoreVertical,
  Edit,
  Trash2,
  Shield,
  Clock,
  Mail,
  MapPin,
  Calendar,
  Eye,
  CheckCircle,
  AlertCircle,
  XCircle,
  RefreshCw
} from 'lucide-react';
import { useSmartPolling } from '@/hooks/useSmartPolling';
import { cn } from '@/lib/utils';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
  permissions: string[];
  isActive: boolean;
  isVerified: boolean;
  createdAt: string;
  lastLoginAt?: string;
  packageTier: string;
}

interface PivotSection {
  id: string;
  label: string;
  icon: any;
  count: number;
  filter: (users: User[]) => User[];
  color: string;
}

interface UserPivotViewProps {
  className?: string;
}

// Mock user data fetcher (replace with real API calls)
async function fetchUsers(): Promise<User[]> {
  try {
    const response = await fetch('/api/v1/admin/users', {
      headers: { 'Cache-Control': 'no-cache' }
    });
    
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    
    const data = await response.json();
    return data.users || [];
  } catch (error) {
    console.error('Failed to fetch users:', error);
    // Return mock data
    return [
      {
        id: '1',
        email: 'john@example.com',
        name: 'John Martinez',
        role: 'admin',
        permissions: ['admin:*:*'],
        isActive: true,
        isVerified: true,
        createdAt: '2024-01-15T10:30:00Z',
        lastLoginAt: '2024-01-20T14:22:00Z',
        packageTier: 'premium'
      },
      {
        id: '2',
        email: 'jane@example.com',
        name: 'Jane Smith',
        role: 'user',
        permissions: ['epsx:analytics:view'],
        isActive: true,
        isVerified: true,
        createdAt: '2024-01-10T09:15:00Z',
        lastLoginAt: '2024-01-20T11:45:00Z',
        packageTier: 'basic'
      },
      {
        id: '3',
        email: 'pending@example.com',
        name: 'Pending User',
        role: 'user',
        permissions: [],
        isActive: false,
        isVerified: false,
        createdAt: '2024-01-20T16:20:00Z',
        packageTier: 'basic'
      },
      {
        id: '4',
        email: 'suspended@example.com',
        name: 'Suspended User',
        role: 'user',
        permissions: ['epsx:analytics:view'],
        isActive: false,
        isVerified: true,
        createdAt: '2024-01-05T12:00:00Z',
        lastLoginAt: '2024-01-18T08:30:00Z',
        packageTier: 'basic'
      }
    ];
  }
}

export function UserPivotView({ className }: UserPivotViewProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [activeSection, setActiveSection] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [showActions, setShowActions] = useState(false);

  // Smart polling for real-time user updates
  const {
    data: users = [],
    error,
    isLoading,
    refresh
  } = useSmartPolling<User[]>('users-list', fetchUsers, {
    priority: 'important'
  });

  // Define pivot sections with Windows Phone style
  const pivotSections: PivotSection[] = useMemo(() => [
    {
      id: 'all',
      label: 'All',
      icon: Users,
      count: users.length,
      filter: (users) => users,
      color: 'blue'
    },
    {
      id: 'active',
      label: 'Active',
      icon: UserCheck,
      count: users.filter(u => u.isActive && u.isVerified).length,
      filter: (users) => users.filter(u => u.isActive && u.isVerified),
      color: 'green'
    },
    {
      id: 'pending',
      label: 'Pending',
      icon: UserPlus,
      count: users.filter(u => !u.isVerified).length,
      filter: (users) => users.filter(u => !u.isVerified),
      color: 'yellow'
    },
    {
      id: 'suspended',
      label: 'Suspended',
      icon: UserX,
      count: users.filter(u => !u.isActive && u.isVerified).length,
      filter: (users) => users.filter(u => !u.isActive && u.isVerified),
      color: 'red'
    }
  ], [users]);

  // Filter users based on active section and search query
  const filteredUsers = useMemo(() => {
    const activeFilter = pivotSections.find(s => s.id === activeSection);
    if (!activeFilter) return [];

    let filtered = activeFilter.filter(users);

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(user =>
        user.email.toLowerCase().includes(query) ||
        user.name?.toLowerCase().includes(query) ||
        user.role.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [users, activeSection, searchQuery, pivotSections]);

  // Handle section change
  const handleSectionChange = (sectionId: string) => {
    setActiveSection(sectionId);
    setSelectedUsers([]);
    setShowActions(false);
  };

  // Handle user selection
  const toggleUserSelection = (userId: string) => {
    setSelectedUsers(prev => {
      const newSelection = prev.includes(userId)
        ? prev.filter(id => id !== userId)
        : [...prev, userId];
      setShowActions(newSelection.length > 0);
      return newSelection;
    });
  };

  // Handle user actions
  const handleUserAction = (action: string, userId?: string) => {
    console.log('User action:', action, userId || selectedUsers);
    // Implement actions here
  };

  // Navigate to user detail
  const navigateToUser = (userId: string) => {
    router.push(`/users/${userId}`);
  };

  const getStatusIcon = (user: User) => {
    if (!user.isVerified) return AlertCircle;
    if (!user.isActive) return XCircle;
    return CheckCircle;
  };

  const getStatusColor = (user: User) => {
    if (!user.isVerified) return 'text-yellow-500';
    if (!user.isActive) return 'text-red-500';
    return 'text-green-500';
  };

  return (
    <div className={cn('flex flex-col h-full', className)}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
              User Management
            </h2>
            <p className="text-gray-600 dark:text-gray-400">
              Manage system users and permissions
            </p>
          </div>
          
          <button
            onClick={refresh}
            disabled={isLoading}
            className="p-2 rounded-lg bg-blue-600 hover:bg-blue-700 text-white transition-colors disabled:opacity-50"
          >
            <RefreshCw className={cn('h-4 w-4', isLoading && 'animate-spin')} />
          </button>
        </div>

        {/* Search Bar */}
        <div className="relative mb-4">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search users..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-12 py-3 bg-gray-100 dark:bg-gray-700 border-0 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 transition-colors"
          />
          <button className="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
            <Filter className="h-4 w-4" />
          </button>
        </div>

        {/* Pivot Navigation */}
        <div className="flex space-x-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
          {pivotSections.map((section, index) => {
            const isActive = activeSection === section.id;
            const Icon = section.icon;
            
            return (
              <button
                key={section.id}
                onClick={() => handleSectionChange(section.id)}
                className={cn(
                  'relative flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200',
                  isActive
                    ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                )}
              >
                <Icon className="h-4 w-4" />
                <span>{section.label}</span>
                <span className={cn(
                  'px-1.5 py-0.5 rounded-full text-xs',
                  isActive
                    ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
                    : 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-400'
                )}>
                  {section.count}
                </span>
                
                {/* Active indicator */}
                {isActive && (
                  <motion.div
                    layoutId="activeTab"
                    className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400"
                  />
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* User List */}
      <div className="flex-1 overflow-auto">
        {isLoading && users.length === 0 ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-center">
              <RefreshCw className="h-8 w-8 animate-spin text-blue-600 mx-auto mb-4" />
              <p className="text-gray-600 dark:text-gray-400">Loading users...</p>
            </div>
          </div>
        ) : error ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-center text-red-600">
              <XCircle className="h-8 w-8 mx-auto mb-4" />
              <p>Failed to load users</p>
              <button
                onClick={refresh}
                className="mt-2 text-sm text-blue-600 hover:text-blue-700"
              >
                Try again
              </button>
            </div>
          </div>
        ) : filteredUsers.length === 0 ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-center text-gray-500">
              <Users className="h-8 w-8 mx-auto mb-4" />
              <p>No users found</p>
            </div>
          </div>
        ) : (
          <div className="p-4 space-y-3">
            <AnimatePresence mode="popLayout">
              {filteredUsers.map((user, index) => {
                const StatusIcon = getStatusIcon(user);
                const isSelected = selectedUsers.includes(user.id);
                
                return (
                  <motion.div
                    key={user.id}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -20 }}
                    transition={{ delay: index * 0.05 }}
                    layout
                    className={cn(
                      'bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border transition-all duration-200 cursor-pointer',
                      isSelected 
                        ? 'border-blue-500 ring-2 ring-blue-200 dark:ring-blue-800' 
                        : 'border-gray-200 dark:border-gray-700 hover:shadow-md hover:border-gray-300 dark:hover:border-gray-600'
                    )}
                    onClick={() => toggleUserSelection(user.id)}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3 flex-1 min-w-0">
                        {/* User Avatar */}
                        <div className="w-12 h-12 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 flex items-center justify-center flex-shrink-0">
                          <span className="text-sm font-bold text-white">
                            {user.name?.charAt(0) || user.email.charAt(0).toUpperCase()}
                          </span>
                        </div>
                        
                        {/* User Info */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <h3 className="font-medium text-gray-900 dark:text-white truncate">
                              {user.name || user.email.split('@')[0]}
                            </h3>
                            <StatusIcon className={cn('h-4 w-4', getStatusColor(user))} />
                            {user.role === 'admin' && (
                              <Shield className="h-4 w-4 text-purple-500" />
                            )}
                          </div>
                          <p className="text-sm text-gray-600 dark:text-gray-400 truncate">
                            {user.email}
                          </p>
                          <div className="flex items-center gap-4 mt-1 text-xs text-gray-500">
                            <span className="flex items-center gap-1">
                              <Calendar className="h-3 w-3" />
                              {new Date(user.createdAt).toLocaleDateString()}
                            </span>
                            {user.lastLoginAt && (
                              <span className="flex items-center gap-1">
                                <Clock className="h-3 w-3" />
                                {new Date(user.lastLoginAt).toLocaleDateString()}
                              </span>
                            )}
                          </div>
                        </div>
                      </div>

                      {/* Actions */}
                      <div className="flex items-center gap-2">
                        <span className={cn(
                          'px-2 py-1 rounded-full text-xs font-medium',
                          user.packageTier === 'premium'
                            ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
                            : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
                        )}>
                          {user.packageTier}
                        </span>
                        
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            navigateToUser(user.id);
                          }}
                          className="p-1 rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        >
                          <Eye className="h-4 w-4" />
                        </button>
                        
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            setShowActions(!showActions);
                          }}
                          className="p-1 rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        >
                          <MoreVertical className="h-4 w-4" />
                        </button>
                      </div>
                    </div>
                  </motion.div>
                );
              })}
            </AnimatePresence>
          </div>
        )}
      </div>

      {/* Bulk Actions Bar */}
      <AnimatePresence>
        {selectedUsers.length > 0 && (
          <motion.div
            initial={{ y: 100, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 100, opacity: 0 }}
            className="p-4 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 shadow-lg"
          >
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {selectedUsers.length} user{selectedUsers.length > 1 ? 's' : ''} selected
              </span>
              
              <div className="flex items-center gap-2">
                <button
                  onClick={() => handleUserAction('edit')}
                  className="flex items-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
                >
                  <Edit className="h-4 w-4" />
                  Edit
                </button>
                
                <button
                  onClick={() => handleUserAction('delete')}
                  className="flex items-center gap-2 px-3 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm transition-colors"
                >
                  <Trash2 className="h-4 w-4" />
                  Delete
                </button>
                
                <button
                  onClick={() => setSelectedUsers([])}
                  className="px-3 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}