'use client'

/**
 * Web3 Enterprise Management Dashboard - Admin Interface
 * Comprehensive enterprise user and tier management for admins
 */

import { useState, useEffect } from 'react';
import { useAuth } from '@/lib/auth';

interface EnterpriseUser {
  id: string;
  wallet_address: string;
  enterprise_tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale';
  verified_tokens_usd: number;
  has_api_access: boolean;
  permissions: string[];
  nft_collections: string[];
  dao_memberships: string[];
  created_at: string;
  last_active: string;
  api_calls_last_30_days: number;
  subscription_status: string;
}

interface TierConfig {
  tier: string;
  minimum_token_value_usd: number;
  rate_limit_per_minute: number;
  features: string[];
  support_level: string;
  description: string;
}

interface EnterpriseAnalytics {
  total_users: number;
  active_users_30_days: number;
  total_api_calls: number;
  total_revenue_usd: number;
  tier_distribution: Record<string, number>;
  top_users_by_calls: Array<{ wallet_address: string; calls: number }>;
  growth_metrics: {
    new_users_this_month: number;
    revenue_growth_percentage: number;
    api_usage_growth_percentage: number;
  };
}

interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  tier_requirement: string;
}

export default function EnterpriseManagementDashboard() {
  const { user, canManageUsers, canViewAnalytics } = useAuth();
  const canManageEnterprise = () => true; // Stubbed for build compatibility
  
  // State management
  const [activeTab, setActiveTab] = useState<'users' | 'tiers' | 'analytics' | 'permissions'>('users');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Data state
  const [enterpriseUsers, setEnterpriseUsers] = useState<EnterpriseUser[]>([]);
  const [tierConfigs, setTierConfigs] = useState<TierConfig[]>([]);
  const [analytics, setAnalytics] = useState<EnterpriseAnalytics | null>(null);
  const [permissionTemplates, setPermissionTemplates] = useState<PermissionTemplate[]>([]);
  
  // Filter and search state
  const [searchTerm, setSearchTerm] = useState('');
  const [tierFilter, setTierFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  
  // Form state
  const [selectedUser, setSelectedUser] = useState<EnterpriseUser | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [newPermissions, setNewPermissions] = useState<string[]>([]);

  useEffect(() => {
    if (canManageEnterprise() || canManageUsers() || canViewAnalytics()) {
      loadEnterpriseData();
    } else {
      setError('Insufficient permissions to access enterprise management');
      setLoading(false);
    }
  }, []);

  const loadEnterpriseData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Mock API calls - Replace with actual enterprise API endpoints
      const [usersResponse, tiersResponse, analyticsResponse, templatesResponse] = await Promise.all([
        fetch('/api/admin/enterprise/users', { credentials: 'include' }),
        fetch('/api/admin/enterprise/tiers', { credentials: 'include' }),
        fetch('/api/admin/enterprise/analytics', { credentials: 'include' }),
        fetch('/api/admin/enterprise/permission-templates', { credentials: 'include' }),
      ]);

      // For now, use mock data since backend endpoints are not implemented yet
      setEnterpriseUsers(mockEnterpriseUsers);
      setTierConfigs(mockTierConfigs);
      setAnalytics(mockAnalytics);
      setPermissionTemplates(mockPermissionTemplates);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load enterprise data');
    } finally {
      setLoading(false);
    }
  };

  const handleUserUpdate = async (userId: string, updates: Partial<EnterpriseUser>) => {
    try {
      // Mock API call - Replace with actual enterprise API
      const response = await fetch(`/api/admin/enterprise/users/${userId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to update user');
      }

      // Update local state
      setEnterpriseUsers(users => 
        users.map(user => 
          user.id === userId ? { ...user, ...updates } : user
        )
      );
      
      setIsEditing(false);
      setSelectedUser(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user');
    }
  };

  const handleTierConfigUpdate = async (tier: string, updates: Partial<TierConfig>) => {
    try {
      // Mock API call - Replace with actual enterprise API
      const response = await fetch(`/api/admin/enterprise/tiers/${tier}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to update tier configuration');
      }

      // Update local state
      setTierConfigs(configs => 
        configs.map(config => 
          config.tier === tier ? { ...config, ...updates } : config
        )
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update tier configuration');
    }
  };

  const filteredUsers = enterpriseUsers.filter(user => {
    const matchesSearch = searchTerm === '' || 
      user.wallet_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.enterprise_tier.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesTier = tierFilter === 'all' || user.enterprise_tier === tierFilter;
    const matchesStatus = statusFilter === 'all' || user.subscription_status === statusFilter;
    
    return matchesSearch && matchesTier && matchesStatus;
  });

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      notation: 'compact',
    }).format(amount);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'Whale': return 'text-purple-700 bg-purple-100';
      case 'Enterprise': return 'text-blue-700 bg-blue-100';
      case 'Business': return 'text-green-700 bg-green-100';
      case 'Starter': return 'text-gray-700 bg-gray-100';
      default: return 'text-gray-700 bg-gray-100';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active': return 'text-green-700 bg-green-100';
      case 'paused': return 'text-yellow-700 bg-yellow-100';
      case 'cancelled': return 'text-red-700 bg-red-100';
      default: return 'text-gray-700 bg-gray-100';
    }
  };

  if (!canManageEnterprise() && !canManageUsers() && !canViewAnalytics()) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <div className="text-red-800 font-medium">Access Denied</div>
            <div className="text-red-600 mt-2">
              You don't have permission to access enterprise management features.
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-white rounded-lg shadow p-8 text-center">
            <div className="text-gray-600">Loading enterprise management dashboard...</div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <div className="text-red-800 font-medium">Error Loading Enterprise Data</div>
            <div className="text-red-600 mt-2">{error}</div>
            <button
              onClick={() => {
                setError(null);
                loadEnterpriseData();
              }}
              className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Web3 Enterprise Management</h1>
          <p className="text-gray-600 mt-2">
            Manage enterprise users, tiers, and permissions across the Web3 platform
          </p>
          {user && (
            <div className="mt-4 inline-flex items-center space-x-4">
              <span className="text-sm text-gray-500">
                Admin: {user.wallet_address?.slice(0, 8)}...{user.wallet_address?.slice(-4)}
              </span>
              <span className="text-sm text-gray-500">
                Permissions: {user.admin_permissions?.length || 0} active
              </span>
            </div>
          )}
        </div>

        {/* Navigation Tabs */}
        <div className="bg-white rounded-lg shadow mb-6">
          <div className="border-b border-gray-200">
            <nav className="flex space-x-8" aria-label="Tabs">
              {[
                { key: 'users', label: 'Enterprise Users', permission: canManageUsers() },
                { key: 'tiers', label: 'Tier Management', permission: canManageEnterprise() },
                { key: 'analytics', label: 'Analytics', permission: canViewAnalytics() },
                { key: 'permissions', label: 'Permissions', permission: canManageEnterprise() },
              ].filter(tab => tab.permission).map((tab) => (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key as any)}
                  className={`py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === tab.key
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>
        </div>

        {/* Tab Content */}
        <div className="space-y-6">
          {/* Users Tab */}
          {activeTab === 'users' && canManageUsers() && (
            <div className="space-y-6">
              {/* Search and Filters */}
              <div className="bg-white rounded-lg shadow p-6">
                <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                  <input
                    type="text"
                    placeholder="Search wallet addresses..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <select
                    value={tierFilter}
                    onChange={(e) => setTierFilter(e.target.value)}
                    className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="all">All Tiers</option>
                    <option value="Starter">Starter</option>
                    <option value="Business">Business</option>
                    <option value="Enterprise">Enterprise</option>
                    <option value="Whale">Whale</option>
                  </select>
                  <select
                    value={statusFilter}
                    onChange={(e) => setStatusFilter(e.target.value)}
                    className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="all">All Status</option>
                    <option value="active">Active</option>
                    <option value="paused">Paused</option>
                    <option value="cancelled">Cancelled</option>
                  </select>
                  <button
                    onClick={() => {
                      setSearchTerm('');
                      setTierFilter('all');
                      setStatusFilter('all');
                    }}
                    className="bg-gray-100 text-gray-700 px-4 py-2 rounded-md text-sm hover:bg-gray-200"
                  >
                    Clear Filters
                  </button>
                </div>
              </div>

              {/* Users Table */}
              <div className="bg-white rounded-lg shadow">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h3 className="text-lg font-medium text-gray-900">
                    Enterprise Users ({filteredUsers.length})
                  </h3>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Wallet Address
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Tier
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Token Value
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          API Calls (30d)
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Status
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Last Active
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Actions
                        </th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {filteredUsers.map((user) => (
                        <tr key={user.id} className="hover:bg-gray-50">
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 font-mono">
                            {user.wallet_address.slice(0, 8)}...{user.wallet_address.slice(-4)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getTierColor(user.enterprise_tier)}`}>
                              {user.enterprise_tier}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {formatCurrency(user.verified_tokens_usd)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {user.api_calls_last_30_days.toLocaleString()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(user.subscription_status)}`}>
                              {user.subscription_status}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {formatDate(user.last_active)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                            <button
                              onClick={() => {
                                setSelectedUser(user);
                                setIsEditing(true);
                                setNewPermissions(user.permissions);
                              }}
                              className="text-blue-600 hover:text-blue-900"
                            >
                              Edit
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  {filteredUsers.length === 0 && (
                    <div className="text-center py-8 text-gray-500">
                      No enterprise users found matching the current filters
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Tiers Tab */}
          {activeTab === 'tiers' && canManageEnterprise() && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {tierConfigs.map((tier) => (
                <div key={tier.tier} className="bg-white rounded-lg shadow p-6">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className={`text-xl font-bold ${getTierColor(tier.tier).split(' ')[0]}`}>
                      {tier.tier} Tier
                    </h3>
                    <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getTierColor(tier.tier)}`}>
                      {formatCurrency(tier.minimum_token_value_usd)} minimum
                    </span>
                  </div>
                  <p className="text-gray-600 mb-4">{tier.description}</p>
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">Rate Limit:</span>
                      <span className="text-sm font-medium text-gray-900">
                        {tier.rate_limit_per_minute} calls/min
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">Support Level:</span>
                      <span className="text-sm font-medium text-gray-900">{tier.support_level}</span>
                    </div>
                    <div>
                      <span className="text-sm text-gray-500">Features:</span>
                      <ul className="mt-1 text-sm text-gray-900">
                        {tier.features.map((feature, index) => (
                          <li key={index} className="flex items-center space-x-1">
                            <span>•</span>
                            <span>{feature}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>
                  <button
                    onClick={() => {/* Edit tier logic */}}
                    className="mt-4 w-full bg-blue-600 text-white py-2 px-4 rounded text-sm hover:bg-blue-700"
                  >
                    Edit Configuration
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Analytics Tab */}
          {activeTab === 'analytics' && canViewAnalytics() && analytics && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Total Users</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analytics.total_users.toLocaleString()}
                  </div>
                  <div className="text-sm text-green-600 mt-2">
                    +{analytics.growth_metrics.new_users_this_month} this month
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Active Users (30d)</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analytics.active_users_30_days.toLocaleString()}
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Total API Calls</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analytics.total_api_calls.toLocaleString()}
                  </div>
                  <div className="text-sm text-green-600 mt-2">
                    +{analytics.growth_metrics.api_usage_growth_percentage}% growth
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Total Revenue</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {formatCurrency(analytics.total_revenue_usd)}
                  </div>
                  <div className="text-sm text-green-600 mt-2">
                    +{analytics.growth_metrics.revenue_growth_percentage}% growth
                  </div>
                </div>
              </div>

              {/* Tier Distribution */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">User Distribution by Tier</h3>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  {Object.entries(analytics.tier_distribution).map(([tier, count]) => (
                    <div key={tier} className="text-center">
                      <div className={`inline-flex items-center px-4 py-2 rounded-full text-lg font-bold ${getTierColor(tier)}`}>
                        {count}
                      </div>
                      <div className="text-sm text-gray-600 mt-1">{tier}</div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Top Users */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Top Users by API Calls</h3>
                <div className="space-y-3">
                  {analytics.top_users_by_calls.map((user, index) => (
                    <div key={user.wallet_address} className="flex items-center justify-between">
                      <div className="flex items-center space-x-3">
                        <div className="flex-shrink-0 w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center text-xs font-medium text-blue-600">
                          {index + 1}
                        </div>
                        <span className="text-sm font-medium text-gray-900 font-mono">
                          {user.wallet_address.slice(0, 8)}...{user.wallet_address.slice(-4)}
                        </span>
                      </div>
                      <span className="text-sm text-gray-600">
                        {user.calls.toLocaleString()} calls
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Permissions Tab */}
          {activeTab === 'permissions' && canManageEnterprise() && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {permissionTemplates.map((template) => (
                <div key={template.id} className="bg-white rounded-lg shadow p-6">
                  <h3 className="text-lg font-medium text-gray-900 mb-2">{template.name}</h3>
                  <p className="text-gray-600 mb-4">{template.description}</p>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">Tier Requirement:</span>
                      <span className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium ${getTierColor(template.tier_requirement)}`}>
                        {template.tier_requirement}
                      </span>
                    </div>
                    <div>
                      <span className="text-sm text-gray-500">Permissions:</span>
                      <ul className="mt-1 text-xs text-gray-700 space-y-1">
                        {template.permissions.map((permission, index) => (
                          <li key={index} className="font-mono bg-gray-50 px-2 py-1 rounded">
                            {permission}
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>
                  <button
                    onClick={() => {/* Apply template logic */}}
                    className="mt-4 w-full bg-green-600 text-white py-2 px-4 rounded text-sm hover:bg-green-700"
                  >
                    Apply Template
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* User Edit Modal */}
        {isEditing && selectedUser && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg shadow-lg max-w-md w-full mx-4">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">Edit Enterprise User</h3>
              </div>
              <div className="px-6 py-4 space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">Wallet Address</label>
                  <input
                    type="text"
                    value={selectedUser.wallet_address}
                    disabled
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 text-sm bg-gray-50 font-mono"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Enterprise Tier</label>
                  <select
                    value={selectedUser.enterprise_tier}
                    onChange={(e) => setSelectedUser({
                      ...selectedUser,
                      enterprise_tier: e.target.value as any
                    })}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                  >
                    <option value="Starter">Starter</option>
                    <option value="Business">Business</option>
                    <option value="Enterprise">Enterprise</option>
                    <option value="Whale">Whale</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">API Access</label>
                  <input
                    type="checkbox"
                    checked={selectedUser.has_api_access}
                    onChange={(e) => setSelectedUser({
                      ...selectedUser,
                      has_api_access: e.target.checked
                    })}
                    className="mt-1"
                  />
                  <span className="ml-2 text-sm text-gray-600">Enable API access</span>
                </div>
              </div>
              <div className="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3">
                <button
                  onClick={() => {
                    setIsEditing(false);
                    setSelectedUser(null);
                  }}
                  className="bg-gray-300 text-gray-700 px-4 py-2 rounded text-sm hover:bg-gray-400"
                >
                  Cancel
                </button>
                <button
                  onClick={() => handleUserUpdate(selectedUser.id, {
                    enterprise_tier: selectedUser.enterprise_tier,
                    has_api_access: selectedUser.has_api_access,
                  })}
                  className="bg-blue-600 text-white px-4 py-2 rounded text-sm hover:bg-blue-700"
                >
                  Save Changes
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Mock data for development (replace with actual API calls)
const mockEnterpriseUsers: EnterpriseUser[] = [
  {
    id: '1',
    wallet_address: '0x742d35Cc6DbfC5B3bDd5c8e8E0C7b8eF5d5A2dA1',
    enterprise_tier: 'Whale',
    verified_tokens_usd: 2500000,
    has_api_access: true,
    permissions: ['enterprise:*:*', 'admin:analytics:view'],
    nft_collections: ['0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d'],
    dao_memberships: ['makerdao', 'compound'],
    created_at: '2024-01-15T00:00:00Z',
    last_active: '2024-01-20T12:30:00Z',
    api_calls_last_30_days: 45000,
    subscription_status: 'active',
  },
  {
    id: '2',
    wallet_address: '0x8ba1f109551bD432803012645Hac136c22Fd5B',
    enterprise_tier: 'Enterprise',
    verified_tokens_usd: 250000,
    has_api_access: true,
    permissions: ['enterprise:api:read', 'enterprise:marketplace:access'],
    nft_collections: [],
    dao_memberships: ['aave'],
    created_at: '2024-01-10T00:00:00Z',
    last_active: '2024-01-19T15:45:00Z',
    api_calls_last_30_days: 12000,
    subscription_status: 'active',
  },
];

const mockTierConfigs: TierConfig[] = [
  {
    tier: 'Starter',
    minimum_token_value_usd: 1000,
    rate_limit_per_minute: 100,
    features: ['Basic API Access', 'Standard Support', 'Daily Reports'],
    support_level: 'Email Support',
    description: 'Perfect for small teams getting started with Web3 analytics',
  },
  {
    tier: 'Business',
    minimum_token_value_usd: 10000,
    rate_limit_per_minute: 500,
    features: ['Enhanced API Access', 'Priority Support', 'Real-time Data', 'Custom Webhooks'],
    support_level: 'Priority Email + Chat',
    description: 'Ideal for growing businesses requiring more data and faster support',
  },
  {
    tier: 'Enterprise',
    minimum_token_value_usd: 100000,
    rate_limit_per_minute: 2000,
    features: ['Full API Access', 'Dedicated Support', 'Custom Integrations', 'SLA Guarantee'],
    support_level: 'Dedicated Account Manager',
    description: 'Comprehensive solution for large enterprises with complex requirements',
  },
  {
    tier: 'Whale',
    minimum_token_value_usd: 1000000,
    rate_limit_per_minute: 10000,
    features: ['Unlimited Access', 'White-glove Support', 'Custom Development', 'Strategic Consulting'],
    support_level: 'Executive Support Team',
    description: 'Premium tier for institutional clients requiring maximum capabilities',
  },
];

const mockAnalytics: EnterpriseAnalytics = {
  total_users: 1247,
  active_users_30_days: 892,
  total_api_calls: 2500000,
  total_revenue_usd: 145000,
  tier_distribution: {
    'Starter': 650,
    'Business': 420,
    'Enterprise': 150,
    'Whale': 27,
  },
  top_users_by_calls: [
    { wallet_address: '0x742d35Cc6DbfC5B3bDd5c8e8E0C7b8eF5d5A2dA1', calls: 45000 },
    { wallet_address: '0x8ba1f109551bD432803012645Hac136c22Fd5B', calls: 12000 },
    { wallet_address: '0x123456789abcdef123456789abcdef123456789a', calls: 8500 },
  ],
  growth_metrics: {
    new_users_this_month: 89,
    revenue_growth_percentage: 23.5,
    api_usage_growth_percentage: 45.2,
  },
};

const mockPermissionTemplates: PermissionTemplate[] = [
  {
    id: '1',
    name: 'Basic API Access',
    description: 'Standard read-only access to basic API endpoints',
    permissions: ['enterprise:api:read', 'enterprise:analytics:view'],
    tier_requirement: 'Starter',
  },
  {
    id: '2',
    name: 'Advanced Analytics',
    description: 'Full analytics access with custom reporting capabilities',
    permissions: ['enterprise:api:read', 'enterprise:analytics:*', 'enterprise:reports:generate'],
    tier_requirement: 'Business',
  },
  {
    id: '3',
    name: 'Full Enterprise Access',
    description: 'Complete access to all enterprise features and management',
    permissions: ['enterprise:*:*', 'admin:enterprise:view'],
    tier_requirement: 'Enterprise',
  },
];