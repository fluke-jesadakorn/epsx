'use client';

import React, { useState, useEffect, useMemo } from 'react';
import { 
  PackageTier, 
  StockRankingConfig, 
  StockRankingPackageConfigs,
  StockRankingType,
  BulkStockRankingAssignment,
  BulkStockRankingAssignmentResult,
  StockRankingPackageAssignment
} from '@epsx/types/src/permission_profile';

interface User {
  id: string;
  email: string;
  name?: string;
  currentPackage?: PackageTier;
  createdAt: Date;
}

interface StockRankingPackageAssignmentProps {
  onAssignmentComplete?: (result: BulkStockRankingAssignmentResult) => void;
}

export default function StockRankingPackageAssignment({ 
  onAssignmentComplete 
}: StockRankingPackageAssignmentProps) {
  // State management
  const [selectedPackage, setSelectedPackage] = useState<PackageTier>(PackageTier.BRONZE);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [assignmentReason, setAssignmentReason] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [users, setUsers] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState({
    users: false,
    assignment: false
  });
  const [assignments, setAssignments] = useState<StockRankingPackageAssignment[]>([]);
  const [expirationDate, setExpirationDate] = useState<string>('');
  const [notifyUsers, setNotifyUsers] = useState(true);
  const [showPreview, setShowPreview] = useState(false);

  // Get configuration for selected package
  const packageConfig = useMemo(() => 
    StockRankingPackageConfigs.getConfigForTier(selectedPackage), 
    [selectedPackage]
  );

  // Filter users based on search query
  const filteredUsers = useMemo(() => 
    users.filter(user => 
      user.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.id.toLowerCase().includes(searchQuery.toLowerCase())
    ), 
    [users, searchQuery]
  );

  // Load users
  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    setIsLoading(prev => ({ ...prev, users: true }));
    try {
      const response = await fetch('/api/admin/user-management/users');
      const data = await response.json();
      setUsers(data.users || []);
    } catch (error) {
      console.error('Failed to load users:', error);
    } finally {
      setIsLoading(prev => ({ ...prev, users: false }));
    }
  };

  const handlePackageChange = (packageTier: PackageTier) => {
    setSelectedPackage(packageTier);
    setShowPreview(true);
  };

  const handleUserSelection = (userId: string, selected: boolean) => {
    if (selected) {
      setSelectedUsers(prev => [...prev, userId]);
    } else {
      setSelectedUsers(prev => prev.filter(id => id !== userId));
    }
  };

  const handleBulkUserSelection = (selectAll: boolean) => {
    if (selectAll) {
      setSelectedUsers(filteredUsers.map(user => user.id));
    } else {
      setSelectedUsers([]);
    }
  };

  const handleAssignment = async () => {
    if (selectedUsers.length === 0) {
      alert('Please select at least one user');
      return;
    }

    if (!assignmentReason.trim()) {
      alert('Please provide a reason for the assignment');
      return;
    }

    setIsLoading(prev => ({ ...prev, assignment: true }));

    try {
      const assignmentData: BulkStockRankingAssignment = {
        userIds: selectedUsers,
        packageTier: selectedPackage,
        permissionProfileId: `${selectedPackage.toLowerCase()}_stock_ranking`,
        reason: assignmentReason,
        expiresAt: expirationDate ? new Date(expirationDate) : undefined,
        assignedBy: 'current_admin', // This should come from auth context
        notifyUsers
      };

      const response = await fetch('/api/admin/stock-ranking/assign-bulk', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(assignmentData),
      });

      const result: BulkStockRankingAssignmentResult = await response.json();

      if (response.ok) {
        onAssignmentComplete?.(result);
        
        // Reset form
        setSelectedUsers([]);
        setAssignmentReason('');
        setExpirationDate('');
        setShowPreview(false);
        
        // Show success message
        alert(`Successfully assigned ${result.summary.successful} users to ${selectedPackage} package`);
        
        if (result.failed.length > 0) {
          console.warn('Some assignments failed:', result.failed);
        }
      } else {
        throw new Error(result.message || 'Assignment failed');
      }
    } catch (error) {
      console.error('Assignment error:', error);
      alert('Failed to assign packages. Please try again.');
    } finally {
      setIsLoading(prev => ({ ...prev, assignment: false }));
    }
  };

  const formatRankingTypes = (types: StockRankingType[]): string => {
    return types.map(type => 
      type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
    ).join(', ');
  };

  const formatMarkets = (markets: string[]): string => {
    if (markets.includes('*')) return 'All Markets';
    return markets.join(', ');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="border-b border-gray-200 pb-4">
        <h2 className="text-2xl font-bold text-gray-900">
          Stock Ranking Package Assignment
        </h2>
        <p className="mt-2 text-sm text-gray-600">
          Assign stock ranking access packages to users with specific permissions and limits
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Package Selection */}
        <div className="space-y-4">
          <h3 className="text-lg font-medium text-gray-900">Select Package Tier</h3>
          
          <div className="grid grid-cols-2 gap-3">
            {Object.values(PackageTier).map((tier) => (
              <button
                key={tier}
                onClick={() => handlePackageChange(tier)}
                className={`p-4 rounded-lg border-2 text-left transition-all ${
                  selectedPackage === tier
                    ? 'border-blue-500 bg-blue-50 text-blue-900'
                    : 'border-gray-200 hover:border-gray-300 text-gray-700'
                }`}
              >
                <div className="font-medium">{tier}</div>
                <div className="text-sm opacity-75">
                  {StockRankingPackageConfigs.getConfigForTier(tier).maxRankings === -1 
                    ? 'Unlimited' 
                    : `${StockRankingPackageConfigs.getConfigForTier(tier).maxRankings} rankings`}
                </div>
              </button>
            ))}
          </div>

          {/* Package Configuration Preview */}
          {showPreview && (
            <div className="bg-gray-50 rounded-lg p-4">
              <h4 className="font-medium text-gray-900 mb-3">
                {selectedPackage} Package Features
              </h4>
              
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">Max Rankings:</span>
                  <span className="font-medium">
                    {packageConfig.maxRankings === -1 ? 'Unlimited' : packageConfig.maxRankings}
                  </span>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-gray-600">Rate Limit:</span>
                  <span className="font-medium">{packageConfig.rateLimitPerMinute}/min</span>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-gray-600">Real-time Updates:</span>
                  <span className="font-medium">
                    {packageConfig.realTimeUpdates ? 'Yes' : 'No'}
                  </span>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-gray-600">Markets:</span>
                  <span className="font-medium">{formatMarkets(packageConfig.allowedMarkets)}</span>
                </div>
                
                <div>
                  <span className="text-gray-600">Ranking Types:</span>
                  <div className="mt-1 text-xs">
                    {formatRankingTypes(packageConfig.allowedRankingTypes)}
                  </div>
                </div>
                
                <div>
                  <span className="text-gray-600">Advanced Features:</span>
                  <div className="mt-1 text-xs">
                    {Object.entries(packageConfig.advancedFeatures)
                      .filter(([_, enabled]) => enabled)
                      .map(([feature, _]) => feature.replace(/([A-Z])/g, ' $1').trim())
                      .join(', ') || 'None'}
                  </div>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-gray-600">Exports/Day:</span>
                  <span className="font-medium">
                    {packageConfig.exportOptions.maxExportsPerDay === -1 
                      ? 'Unlimited' 
                      : packageConfig.exportOptions.maxExportsPerDay}
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* User Selection */}
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="text-lg font-medium text-gray-900">Select Users</h3>
            <div className="text-sm text-gray-500">
              {selectedUsers.length} selected
            </div>
          </div>

          {/* Search */}
          <div className="relative">
            <input
              type="text"
              placeholder="Search users by email, name, or ID..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Bulk Selection */}
          <div className="flex space-x-2">
            <button
              onClick={() => handleBulkUserSelection(true)}
              className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200"
            >
              Select All
            </button>
            <button
              onClick={() => handleBulkUserSelection(false)}
              className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
            >
              Clear All
            </button>
          </div>

          {/* Users List */}
          <div className="border border-gray-200 rounded-lg max-h-64 overflow-y-auto">
            {isLoading.users ? (
              <div className="p-4 text-center text-gray-500">Loading users...</div>
            ) : filteredUsers.length === 0 ? (
              <div className="p-4 text-center text-gray-500">No users found</div>
            ) : (
              filteredUsers.map((user) => (
                <div key={user.id} className="flex items-center p-3 border-b border-gray-100 last:border-b-0">
                  <input
                    type="checkbox"
                    checked={selectedUsers.includes(user.id)}
                    onChange={(e) => handleUserSelection(user.id, e.target.checked)}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <div className="ml-3 flex-1">
                    <div className="text-sm font-medium text-gray-900">
                      {user.name || user.email}
                    </div>
                    <div className="text-xs text-gray-500">
                      {user.email} • Current: {user.currentPackage || 'None'}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Assignment Details */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Assignment Details</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Reason for Assignment *
            </label>
            <textarea
              value={assignmentReason}
              onChange={(e) => setAssignmentReason(e.target.value)}
              placeholder="e.g., Promotional upgrade, Trial access, Customer request..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              rows={3}
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Expiration Date (Optional)
            </label>
            <input
              type="datetime-local"
              value={expirationDate}
              onChange={(e) => setExpirationDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
        </div>

        {/* Options */}
        <div className="flex items-center space-x-4">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={notifyUsers}
              onChange={(e) => setNotifyUsers(e.target.checked)}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <span className="ml-2 text-sm text-gray-700">Notify users via email</span>
          </label>
        </div>
      </div>

      {/* Assignment Summary */}
      {selectedUsers.length > 0 && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h4 className="font-medium text-blue-900 mb-2">Assignment Summary</h4>
          <div className="text-sm text-blue-800">
            <div>Package: <strong>{selectedPackage}</strong></div>
            <div>Users: <strong>{selectedUsers.length} selected</strong></div>
            <div>Max Rankings: <strong>
              {packageConfig.maxRankings === -1 ? 'Unlimited' : packageConfig.maxRankings}
            </strong></div>
            <div>Rate Limit: <strong>{packageConfig.rateLimitPerMinute} requests/minute</strong></div>
            {expirationDate && (
              <div>Expires: <strong>{new Date(expirationDate).toLocaleString()}</strong></div>
            )}
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="flex justify-end space-x-3 pt-4 border-t border-gray-200">
        <button
          type="button"
          onClick={() => {
            setSelectedUsers([]);
            setAssignmentReason('');
            setExpirationDate('');
            setShowPreview(false);
          }}
          className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Reset
        </button>
        
        <button
          type="button"
          onClick={handleAssignment}
          disabled={selectedUsers.length === 0 || !assignmentReason.trim() || isLoading.assignment}
          className="px-6 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading.assignment ? 'Assigning...' : `Assign ${selectedPackage} Package`}
        </button>
      </div>
    </div>
  );
}