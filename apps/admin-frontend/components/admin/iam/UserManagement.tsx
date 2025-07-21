'use client';

import React, { useState, useEffect } from 'react';
import { 
  Search, 
  Filter, 
  MoreHorizontal, 
  Edit, 
  Crown,
  Shield,
  Mail,
  Calendar,
  ChevronDown,
  UserCheck,
  UserX,
  Upload,
  Trash2
} from 'lucide-react';
import type { UserWithPermissions } from '../../../types/admin/iam-enhanced';
import { PackageTier, SubscriptionStatus } from '../../../types/admin/iam-enhanced';
import { iamService } from '../../../services/iamService';
import { UserDetailsModal } from './UserDetailsModal';
import { BulkActionsModal } from './BulkActionsModal';
import { useToast } from './useToast';
import { ToastContainer } from './ToastContainer';
import { useConfirmation } from './useConfirmation';
import { ConfirmationModal } from './ConfirmationModal';

interface UserManagementProps {
  onStatsUpdate?: () => void;
}

export const UserManagement: React.FC<UserManagementProps> = ({ onStatsUpdate }) => {
  const [users, setUsers] = useState<UserWithPermissions[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [selectedUser, setSelectedUser] = useState<UserWithPermissions | null>(null);
  const [showUserModal, setShowUserModal] = useState(false);
  const [showBulkModal, setShowBulkModal] = useState(false);
  const [openDropdown, setOpenDropdown] = useState<string | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState({
    packageTier: '',
    subscriptionStatus: '',
    hasCustomPermissions: false,
  });
  
  const toast = useToast();
  const confirmation = useConfirmation();

  useEffect(() => {
    loadUsers();
  }, [filters]);

  useEffect(() => {
    const handleClickOutside = () => {
      setOpenDropdown(null);
    };
    
    if (openDropdown) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [openDropdown]);

  const loadUsers = async () => {
    try {
      setLoading(true);
      const userData = await iamService.getUsers({
        ...(filters.packageTier && { packageTier: filters.packageTier as PackageTier }),
        ...(filters.subscriptionStatus && { subscriptionStatus: filters.subscriptionStatus }),
        ...(filters.hasCustomPermissions && { hasCustomPermissions: true }),
      });
      setUsers(userData);
      toast.success(`Loaded ${userData.length} users successfully`);
    } catch (error) {
      console.error('Failed to load users:', error);
      toast.error('Failed to load users. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const filteredUsers = users.filter(user => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return (
      user.email?.toLowerCase().includes(query) ||
      user.displayName?.toLowerCase().includes(query) ||
      user.name?.toLowerCase().includes(query)
    );
  });

  const handleUserSelect = (userId: string, checked: boolean) => {
    if (checked) {
      setSelectedUsers([...selectedUsers, userId]);
    } else {
      setSelectedUsers(selectedUsers.filter(id => id !== userId));
    }
  };

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedUsers(filteredUsers.map(user => user.id));
    } else {
      setSelectedUsers([]);
    }
  };

  const handleUserClick = (user: UserWithPermissions) => {
    setSelectedUser(user);
    setShowUserModal(true);
  };

  const handleUserStatusChange = async (userId: string, newStatus: SubscriptionStatus) => {
    try {
      // This would be implemented in the IAM service
      console.log('Changing user status:', userId, newStatus);
      await loadUsers();
      toast.success(`User status changed to ${newStatus.toUpperCase()}`);
    } catch (error) {
      console.error('Failed to change user status:', error);
      toast.error('Failed to change user status. Please try again.');
    }
  };

  const handleDeleteUser = async (userId: string, userEmail: string) => {
    const confirmed = await confirmation.confirm({
      title: 'Delete User',
      message: `Are you sure you want to delete the user "${userEmail}"? This action cannot be undone.`,
      confirmText: 'Delete',
      cancelText: 'Cancel',
      type: 'danger'
    });

    if (confirmed) {
      try {
        // This would be implemented in the IAM service
        console.log('Deleting user:', userId);
        // await iamService.deleteUser(userId);
        await loadUsers();
        onStatsUpdate?.();
        toast.success(`User "${userEmail}" has been deleted successfully`);
      } catch (error) {
        console.error('Failed to delete user:', error);
        toast.error('Failed to delete user. Please try again.');
      }
    }
  };

  // Re-enable package upgrade functionality
  const handlePackageUpgrade = async (userId: string, newTier: PackageTier) => {
    try {
      await iamService.updateUserPackageTier(userId, newTier, 'current-admin-id');
      await loadUsers();
      onStatsUpdate?.();
      toast.success(`Successfully upgraded user package to ${newTier.toUpperCase()}`);
    } catch (error) {
      console.error('Failed to upgrade package:', error);
      toast.error('Failed to upgrade package. Please try again.');
    }
  };

  const getTierColor = (tier: PackageTier) => {
    switch (tier) {
      case PackageTier.ENTERPRISE:
        return 'bg-purple-100 text-purple-800 border-purple-200';
      case PackageTier.PLATINUM:
        return 'bg-gray-100 text-gray-800 border-gray-200';
      case PackageTier.GOLD:
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case PackageTier.SILVER:
        return 'bg-blue-100 text-blue-800 border-blue-200';
      case PackageTier.BRONZE:
        return 'bg-orange-100 text-orange-800 border-orange-200';
      default:
        return 'bg-gray-100 text-gray-500 border-gray-200';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800';
      case 'trial':
        return 'bg-blue-100 text-blue-800';
      case 'expired':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const formatDate = (date: Date | string | undefined) => {
    if (!date) return 'Never';
    return new Date(date).toLocaleDateString();
  };

  if (loading) {
    return (
      <div className="space-y-4">
        <div className="animate-pulse">
          <div className="h-10 bg-gray-200 rounded mb-4"></div>
          <div className="space-y-3">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="h-16 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Toast Notifications */}
      <ToastContainer toasts={toast.toasts} onRemove={toast.removeToast} />
      
      {/* Confirmation Dialog */}
      {confirmation.dialog && <ConfirmationModal dialog={confirmation.dialog} />}
      
      {/* Search and Filter Header */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
          <input
            type="text"
            placeholder="Search users by name or email..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        
        <div className="flex gap-2">
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`inline-flex items-center px-4 py-2 border rounded-lg text-sm font-medium transition-colors ${
              showFilters 
                ? 'border-blue-300 bg-blue-50 text-blue-700' 
                : 'border-gray-300 bg-white text-gray-700 hover:bg-gray-50'
            }`}
          >
            <Filter className="h-4 w-4 mr-2" />
            Filters
            <ChevronDown className={`h-4 w-4 ml-2 transition-transform ${showFilters ? 'rotate-180' : ''}`} />
          </button>
          
          {selectedUsers.length > 0 && (
            <button
              onClick={() => setShowBulkModal(true)}
              className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
            >
              <Upload className="h-4 w-4 mr-2" />
              Bulk Actions ({selectedUsers.length})
            </button>
          )}
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Package Tier
              </label>
              <select
                value={filters.packageTier}
                onChange={(e) => setFilters({ ...filters, packageTier: e.target.value })}
                className="w-full border border-gray-300 rounded-lg px-3 py-2"
              >
                <option value="">All Tiers</option>
                {Object.values(PackageTier).map((tier) => (
                  <option key={tier} value={tier}>
                    {tier.charAt(0).toUpperCase() + tier.slice(1)}
                  </option>
                ))}
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Subscription Status
              </label>
              <select
                value={filters.subscriptionStatus}
                onChange={(e) => setFilters({ ...filters, subscriptionStatus: e.target.value })}
                className="w-full border border-gray-300 rounded-lg px-3 py-2"
              >
                <option value="">All Statuses</option>
                {Object.values(SubscriptionStatus).map((status) => (
                  <option key={status} value={status}>
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </option>
                ))}
              </select>
            </div>
            
            <div className="flex items-center">
              <input
                type="checkbox"
                id="hasCustomPermissions"
                checked={filters.hasCustomPermissions}
                onChange={(e) => setFilters({ ...filters, hasCustomPermissions: e.target.checked })}
                className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <label htmlFor="hasCustomPermissions" className="ml-2 text-sm text-gray-700">
                Has custom permissions
              </label>
            </div>
          </div>
        </div>
      )}

      {/* Users Table */}
      <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="w-12 px-6 py-3">
                  <input
                    type="checkbox"
                    checked={selectedUsers.length === filteredUsers.length && filteredUsers.length > 0}
                    onChange={(e) => handleSelectAll(e.target.checked)}
                    className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                  />
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  User
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Package
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Permissions
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Payment
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredUsers.map((user) => (
                <tr key={user.id} className="hover:bg-gray-50 cursor-pointer" onClick={() => handleUserClick(user)}>
                  <td className="px-6 py-4" onClick={(e) => e.stopPropagation()}>
                    <input
                      type="checkbox"
                      checked={selectedUsers.includes(user.id)}
                      onChange={(e) => handleUserSelect(user.id, e.target.checked)}
                      className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                    />
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className="h-10 w-10 rounded-full bg-blue-100 flex items-center justify-center">
                        <span className="text-sm font-medium text-blue-600">
                          {(user.displayName || user.name || user.email)?.charAt(0).toUpperCase()}
                        </span>
                      </div>
                      <div className="ml-4">
                        <div className="text-sm font-medium text-gray-900">
                          {user.displayName || user.name || user.email?.split('@')[0]}
                        </div>
                        <div className="text-sm text-gray-500 flex items-center">
                          <Mail className="h-3 w-3 mr-1" />
                          {user.email}
                        </div>
                      </div>
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getTierColor(user.packageTier)}`}>
                      <Crown className="h-3 w-3 mr-1" />
                      {user.packageTier.toUpperCase()}
                    </span>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(user.subscriptionStatus)}`}>
                      {user.subscriptionStatus === 'active' ? <UserCheck className="h-3 w-3 mr-1" /> : <UserX className="h-3 w-3 mr-1" />}
                      {user.subscriptionStatus}
                    </span>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center text-sm text-gray-900">
                      <Shield className="h-4 w-4 mr-1 text-gray-400" />
                      {user.customPermissions?.length || 0} custom
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    <div className="flex items-center">
                      <Calendar className="h-4 w-4 mr-1" />
                      {formatDate(user.lastPaymentDate)}
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium" onClick={(e) => e.stopPropagation()}>
                    <div className="relative inline-block text-left">
                      <button
                        onClick={() => setOpenDropdown(openDropdown === user.id ? null : user.id)}
                        className="text-gray-400 hover:text-gray-600 p-1 rounded-md hover:bg-gray-100"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </button>
                      
                      {openDropdown === user.id && (
                        <div className="absolute right-0 z-10 mt-2 w-48 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5">
                          <div className="py-1">
                            <button
                              onClick={() => {
                                handleUserClick(user);
                                setOpenDropdown(null);
                              }}
                              className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            >
                              <Edit className="h-4 w-4 mr-2" />
                              Edit User
                            </button>
                            
                            <button
                              onClick={() => {
                                handlePackageUpgrade(user.id, PackageTier.GOLD);
                                setOpenDropdown(null);
                              }}
                              className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            >
                              <Crown className="h-4 w-4 mr-2" />
                              Upgrade to Gold
                            </button>
                            
                            <button
                              onClick={() => {
                                handleUserStatusChange(user.id, SubscriptionStatus.ACTIVE);
                                setOpenDropdown(null);
                              }}
                              className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            >
                              <UserCheck className="h-4 w-4 mr-2" />
                              Activate User
                            </button>
                            
                            <button
                              onClick={() => {
                                console.log('View permissions for:', user.email);
                                setOpenDropdown(null);
                              }}
                              className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            >
                              <Shield className="h-4 w-4 mr-2" />
                              View Permissions
                            </button>
                            
                            <div className="border-t border-gray-100 my-1"></div>
                            
                            <button
                              onClick={() => {
                                handleDeleteUser(user.id, user.email);
                                setOpenDropdown(null);
                              }}
                              className="flex items-center w-full px-4 py-2 text-sm text-red-700 hover:bg-red-50"
                            >
                              <Trash2 className="h-4 w-4 mr-2" />
                              Delete User
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        
        {filteredUsers.length === 0 && (
          <div className="text-center py-12">
            <div className="text-gray-500">
              {searchQuery ? 'No users found matching your search.' : 'No users found.'}
            </div>
          </div>
        )}
      </div>

      {/* Pagination would go here */}
      
      {/* Modals */}
      {showUserModal && selectedUser && (
        <UserDetailsModal
          user={selectedUser}
          onClose={() => {
            setShowUserModal(false);
            setSelectedUser(null);
          }}
          onUpdate={loadUsers}
        />
      )}
      
      {showBulkModal && (
        <BulkActionsModal
          selectedUserIds={selectedUsers}
          onClose={() => setShowBulkModal(false)}
          onComplete={() => {
            setShowBulkModal(false);
            setSelectedUsers([]);
            loadUsers();
            onStatsUpdate?.();
          }}
        />
      )}
    </div>
  );
};
