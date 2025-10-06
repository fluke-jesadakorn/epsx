'use client';

import React, { useState, useEffect, useMemo } from 'react';

enum PackageTier {
  FREE = 'free',
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  ENTERPRISE = 'enterprise',
  PREMIUM = 'premium'
}
type StockRankingType = any;
type BulkStockRankingAssignment = any;
type BulkStockRankingAssignmentResult = any;
type StockRankingPackageData = any;

const StockRankingPackageConfigs = {
  getConfigForTier: (tier: PackageTier) => ({
    maxRankings: tier === PackageTier.ENTERPRISE ? -1 : 50,
    rateLimitPerMinute: 60,
    realTimeUpdates: true,
    allowedMarkets: ['*'],
    allowedRankingTypes: ['basic', 'advanced'] as any[],
    advancedFeatures: {
      customFilters: tier !== PackageTier.FREE,
      exportData: tier !== PackageTier.FREE
    },
    exportOptions: {
      maxExportsPerDay: tier === PackageTier.ENTERPRISE ? -1 : 10
    }
  })
};

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

/**
 *
 * @param root0
 * @param root0.onAssignmentComplete
 */
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
  const [assignments, setAssignments] = useState<StockRankingPackageData[]>([]);
  const _assignments = assignments;
  const _setAssignments = setAssignments;
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
      const response = await fetch('/api/v1/admin/users');
      
      if (!response.ok) {
        // eslint-disable-next-line no-console
        console.error('Users API error:', response.status, response.statusText);
        setUsers([]);
        return;
      }
      
      const contentType = response.headers.get('content-type');
      if (!contentType?.includes('application/json')) {
        // eslint-disable-next-line no-console
        console.error('Invalid users response type:', contentType);
        setUsers([]);
        return;
      }
      
      const data = await response.json();
      setUsers(data.users || []);
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load users:', _error);
      setUsers([]);
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
      const assignmentData = {
        user_ids: selectedUsers,
        assignments: [{
          module_id: 'stock-ranking-module-id', // This should come from a module registry
          access_level: selectedPackage.toLowerCase(),
          custom_quotas: null,
          restrictions: null,
          expires_at: expirationDate ? new Date(expirationDate).toISOString() : null
        }],
        reason: assignmentReason
      };

      const response = await fetch('/api/v1/admin/users/bulk/assign-modules', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(assignmentData),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Assignment failed: ${response.status} ${errorText}`);
      }

      const contentType = response.headers.get('content-type');
      if (!contentType?.includes('application/json')) {
        throw new Error('Invalid response format from assignment API');
      }

      const result = await response.json();

      if (response.ok) {
        // Create a compatible result object for the callback
        const compatibleResult = {
          summary: result.summary,
          failed: result.failed || [],
          message: result.message
        };
        onAssignmentComplete?.(compatibleResult);
        
        // Reset form
        setSelectedUsers([]);
        setAssignmentReason('');
        setExpirationDate('');
        setShowPreview(false);
        
        // Show success message
        alert(`Successfully assigned ${result.summary.successful} users to ${selectedPackage} package`);
        
        if (result.failed && result.failed.length > 0) {
          // eslint-disable-next-line no-console
          console.warn('Some assignments failed:', result.failed);
        }
      } else {
        throw new Error(result.message || 'Assignment failed');
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Assignment error:', _error);
      alert('Failed to assign packages. Please try again.');
    } finally {
      setIsLoading(prev => ({ ...prev, assignment: false }));
    }
  };

  const formatRankingTypes = (types: StockRankingType[]): string => {
    return types.map(type => 
      type.replace(/_/g, ' ').replace(/\b\w/g, (l: string) => l.toUpperCase())
    ).join(', ');
  };

  const formatMarkets = (markets: string[]): string => {
    if (markets.includes('*')) {return 'All Markets';}
    return markets.join(', ');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="border-b border-border pb-4">
        <h2 className="text-2xl font-bold text-foreground">
          Stock Ranking Package Assignment
        </h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Assign stock ranking access packages to users with specific permissions and limits
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Package Selection */}
        <div className="space-y-4">
          <h3 className="text-lg font-medium text-foreground">Select Package Tier</h3>
          
          <div className="grid grid-cols-2 gap-3">
            {Object.values(PackageTier).map((tier) => (
              <button
                key={tier}
                onClick={() => handlePackageChange(tier)}
                className={`p-4 rounded-lg border-2 text-left transition-all ${
                  selectedPackage === tier
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-900 dark:text-blue-300'
                    : 'border-border hover:border-gray-300 dark:hover:border-gray-600 text-foreground'
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
            <div className="bg-muted rounded-lg p-4">
              <h4 className="font-medium text-foreground mb-3">
                {selectedPackage} Package Features
              </h4>
              
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Max Rankings:</span>
                  <span className="font-medium">
                    {packageConfig.maxRankings === -1 ? 'Unlimited' : packageConfig.maxRankings}
                  </span>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Rate Limit:</span>
                  <span className="font-medium">{packageConfig.rateLimitPerMinute}/min</span>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Real-time Updates:</span>
                  <span className="font-medium">
                    {packageConfig.realTimeUpdates ? 'Yes' : 'No'}
                  </span>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Markets:</span>
                  <span className="font-medium">{formatMarkets(packageConfig.allowedMarkets)}</span>
                </div>
                
                <div>
                  <span className="text-muted-foreground">Ranking Types:</span>
                  <div className="mt-1 text-xs">
                    {formatRankingTypes(packageConfig.allowedRankingTypes)}
                  </div>
                </div>
                
                <div>
                  <span className="text-muted-foreground">Advanced Features:</span>
                  <div className="mt-1 text-xs">
                    {Object.entries(packageConfig.advancedFeatures)
                      .filter(([_, enabled]) => enabled)
                      .map(([feature, _]) => feature.replace(/([A-Z])/g, ' $1').trim())
                      .join(', ') || 'None'}
                  </div>
                </div>
                
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Exports/Day:</span>
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
            <h3 className="text-lg font-medium text-foreground">Select Users</h3>
            <div className="text-sm text-muted-foreground">
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
              className="w-full px-4 py-2 border border-input rounded-lg bg-background focus:ring-2 focus:ring-blue-500 focus:border-transparent text-foreground placeholder:text-muted-foreground"
            />
          </div>

          {/* Bulk Selection */}
          <div className="flex space-x-2">
            <button
              onClick={() => handleBulkUserSelection(true)}
              className="px-3 py-1 text-sm bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded hover:bg-blue-200 dark:hover:bg-blue-900/50"
            >
              Select All
            </button>
            <button
              onClick={() => handleBulkUserSelection(false)}
              className="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80"
            >
              Clear All
            </button>
          </div>

          {/* Users List */}
          <div className="border border-border rounded-lg max-h-64 overflow-y-auto bg-background">
            {isLoading.users ? (
              <div className="p-4 text-center text-muted-foreground">Loading users...</div>
            ) : filteredUsers.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground">No users found</div>
            ) : (
              filteredUsers.map((user) => (
                <div key={user.id} className="flex items-center p-3 border-b border-border/50 last:border-b-0 hover:bg-muted/50">
                  <input
                    type="checkbox"
                    checked={selectedUsers.includes(user.id)}
                    onChange={(e) => handleUserSelection(user.id, e.target.checked)}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-input rounded"
                  />
                  <div className="ml-3 flex-1">
                    <div className="text-sm font-medium text-foreground">
                      {user.name || user.email}
                    </div>
                    <div className="text-xs text-muted-foreground">
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
        <h3 className="text-lg font-medium text-foreground">Assignment Details</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Reason for Assignment *
            </label>
            <textarea
              value={assignmentReason}
              onChange={(e) => setAssignmentReason(e.target.value)}
              placeholder="e.g., Promotional upgrade, Trial access, Customer request..."
              className="w-full px-3 py-2 border border-input rounded-md bg-background focus:ring-2 focus:ring-blue-500 focus:border-transparent text-foreground placeholder:text-muted-foreground"
              rows={3}
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Expiration Date (Optional)
            </label>
            <input
              type="datetime-local"
              value={expirationDate}
              onChange={(e) => setExpirationDate(e.target.value)}
              className="w-full px-3 py-2 border border-input rounded-md bg-background focus:ring-2 focus:ring-blue-500 focus:border-transparent text-foreground"
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
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-input rounded"
            />
            <span className="ml-2 text-sm text-foreground">Notify users via email</span>
          </label>
        </div>
      </div>

      {/* Assignment Summary */}
      {selectedUsers.length > 0 && (
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800/50 rounded-lg p-4">
          <h4 className="font-medium text-blue-900 dark:text-blue-300 mb-2">Assignment Summary</h4>
          <div className="text-sm text-blue-800 dark:text-blue-200">
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
      <div className="flex justify-end space-x-3 pt-4 border-t border-border">
        <button
          type="button"
          onClick={() => {
            setSelectedUsers([]);
            setAssignmentReason('');
            setExpirationDate('');
            setShowPreview(false);
          }}
          className="px-4 py-2 text-sm font-medium text-foreground bg-background border border-input rounded-md hover:bg-muted focus:outline-none focus:ring-2 focus:ring-blue-500"
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