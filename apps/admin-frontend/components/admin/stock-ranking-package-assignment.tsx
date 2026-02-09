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

type StockRankingType = string;
interface BulkStockRankingAssignmentResult {
  summary: { successful: number };
  failed?: Array<{ id: string; error: string }>;
  message?: string;
}

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

const StockRankingPackageConfigs = {
  getConfigForTier: (tier: PackageTier) => ({
    maxRankings: tier === PackageTier.ENTERPRISE ? -1 : 50,
    rateLimitPerMinute: 60,
    realTimeUpdates: true,
    allowedMarkets: ['*'],
    allowedRankingTypes: ['basic', 'advanced'] as string[],
    advancedFeatures: {
      customFilters: tier !== PackageTier.FREE,
      exportData: tier !== PackageTier.FREE
    },
    exportOptions: {
      maxExportsPerDay: tier === PackageTier.ENTERPRISE ? -1 : 10
    }
  })
};

function PackageSelector({
  selected,
  onChange
}: {
  selected: PackageTier;
  onChange: (tier: PackageTier) => void;
}) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium text-foreground">Select Package Tier</h3>
      <div className="grid grid-cols-2 gap-3">
        {Object.values(PackageTier).map((tier) => (
          <button
            key={tier}
            onClick={() => onChange(tier)}
            className={`p-4 rounded-lg border-2 text-left transition-all ${
              selected === tier
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
    </div>
  );
}

function PackagePreview({
  tier,
  show
}: {
  tier: PackageTier;
  show: boolean;
}) {
  const config = StockRankingPackageConfigs.getConfigForTier(tier);

  if (!show) {
    return null;
  }

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
    <div className="bg-muted rounded-lg p-4">
      <h4 className="font-medium text-foreground mb-3">
        {tier} Package Features
      </h4>
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-muted-foreground">Max Rankings:</span>
          <span className="font-medium">
            {config.maxRankings === -1 ? 'Unlimited' : config.maxRankings}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Rate Limit:</span>
          <span className="font-medium">{config.rateLimitPerMinute}/min</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Real-time Updates:</span>
          <span className="font-medium">{config.realTimeUpdates ? 'Yes' : 'No'}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Markets:</span>
          <span className="font-medium">{formatMarkets(config.allowedMarkets)}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Ranking Types:</span>
          <div className="mt-1 text-xs">{formatRankingTypes(config.allowedRankingTypes)}</div>
        </div>
        <div>
          <span className="text-muted-foreground">Advanced Features:</span>
          <div className="mt-1 text-xs">
            {Object.entries(config.advancedFeatures)
              .filter(([_, enabled]) => enabled)
              .map(([feature]) => feature.replace(/([A-Z])/g, ' $1').trim())
               
              .join(', ') ?? 'None'}
          </div>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Exports/Day:</span>
          <span className="font-medium">
            {config.exportOptions.maxExportsPerDay === -1 ? 'Unlimited' : config.exportOptions.maxExportsPerDay}
          </span>
        </div>
      </div>
    </div>
  );
}

function UserSelector({
  users,
  selectedIds,
  searchQuery,
  isLoading,
  onSearchChange,
  onUserToggle,
  onSelectAll,
  onClearAll
}: {
  users: User[];
  selectedIds: string[];
  searchQuery: string;
  isLoading: boolean;
  onSearchChange: (q: string) => void;
  onUserToggle: (id: string, selected: boolean) => void;
  onSelectAll: () => void;
  onClearAll: () => void;
}) {
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium text-foreground">Select Users</h3>
        <div className="text-sm text-muted-foreground">{selectedIds.length} selected</div>
      </div>
      <div className="relative">
        <input
          type="text"
          placeholder="Search users by email, name, or ID..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="w-full px-4 py-2 border border-input rounded-lg bg-background focus:ring-2 focus:ring-blue-500 focus:border-transparent text-foreground placeholder:text-muted-foreground"
        />
      </div>
      <div className="flex space-x-2">
        <button
          onClick={onSelectAll}
          className="px-3 py-1 text-sm bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded hover:bg-blue-200 dark:hover:bg-blue-900/50"
        >
          Select All
        </button>
        <button
          onClick={onClearAll}
          className="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80"
        >
          Clear All
        </button>
      </div>
      <div className="border border-border rounded-lg max-h-64 overflow-y-auto bg-background">
        {isLoading ? (
          <div className="p-4 text-center text-muted-foreground">Loading users...</div>
        ) : users.length === 0 ? (
          <div className="p-4 text-center text-muted-foreground">No users found</div>
        ) : (
          users.map((user) => (
            <div
              key={user.id}
              className="flex items-center p-3 border-b border-border/50 last:border-b-0 hover:bg-muted/50"
            >
              <input
                type="checkbox"
                checked={selectedIds.includes(user.id)}
                onChange={(e) => onUserToggle(user.id, e.target.checked)}
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-input rounded"
              />
              <div className="ml-3 flex-1">
                <div className="text-sm font-medium text-foreground">{user.name ?? user.email}</div>
                <div className="text-xs text-muted-foreground">
                  {user.email} • Current: {user.currentPackage ?? 'None'}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function AssignmentForm({
  reason,
  expirationDate,
  notifyUsers,
  onReasonChange,
  onExpirationChange,
  onNotifyChange
}: {
  reason: string;
  expirationDate: string;
  notifyUsers: boolean;
  onReasonChange: (r: string) => void;
  onExpirationChange: (d: string) => void;
  onNotifyChange: (n: boolean) => void;
}) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium text-foreground">Assignment Details</h3>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-foreground mb-2">
            Reason for Assignment *
          </label>
          <textarea
            value={reason}
            onChange={(e) => onReasonChange(e.target.value)}
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
            onChange={(e) => onExpirationChange(e.target.value)}
            className="w-full px-3 py-2 border border-input rounded-md bg-background focus:ring-2 focus:ring-blue-500 focus:border-transparent text-foreground"
          />
        </div>
      </div>
      <div className="flex items-center space-x-4">
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={notifyUsers}
            onChange={(e) => onNotifyChange(e.target.checked)}
            className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-input rounded"
          />
          <span className="ml-2 text-sm text-foreground">Notify users via email</span>
        </label>
      </div>
    </div>
  );
}

interface AssignmentPayload {
  selectedUsers: string[];
  selectedPackage: PackageTier;
  assignmentReason: string;
  expirationDate: string;
}

function useLoadUsers() {
  const [users, setUsers] = React.useState<User[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setIsLoading(true);
    try {
      const response = await fetch('/api/admin/users');
      if (response.ok) {
        const contentType = response.headers.get('content-type');
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        if (contentType?.includes('application/json')) {
          const data = await response.json() as { users?: User[] };
          setUsers(data.users ?? []);
        }
      }
    } catch {
      // silent fail
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { users, isLoading, load };
}

function useAssignmentSubmit(
  onComplete?: (result: BulkStockRankingAssignmentResult) => void
) {
  const [isLoading, setIsLoading] = React.useState(false);

  const submit = React.useCallback(
    async (payload: AssignmentPayload) => {
      if (payload.selectedUsers.length === 0) {
        return;
      }
      if (!payload.assignmentReason.trim()) {
        return;
      }

      setIsLoading(true);
      try {
        const assignmentData = {
          user_ids: payload.selectedUsers,
          assignments: [
            {
              module_id: 'stock-ranking-module-id',
              access_level: payload.selectedPackage.toLowerCase(),
              custom_quotas: null,
              restrictions: null,
              // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
              expires_at: payload.expirationDate !== null ? new Date(payload.expirationDate).toISOString() : null
            }
          ],
          reason: payload.assignmentReason
        };

        const response = await fetch('/api/admin/users/bulk/assign-modules', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(assignmentData)
        });

        if (!response.ok) {
          const errorText = await response.text();
          throw new Error(`Assignment failed: ${response.status} ${errorText}`);
        }

        const contentType = response.headers.get('content-type');
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        if (!contentType?.includes('application/json')) {
          throw new Error('Invalid response format from assignment API');
        }

        const result = await response.json() as BulkStockRankingAssignmentResult;
        onComplete?.(result);
        return result;
      } finally {
        setIsLoading(false);
      }
    },
    [onComplete]
  );

  return { submit, isLoading };
}

function AssignmentSummary({
  show,
  selectedCount,
  packageTier,
  expirationDate
}: {
  show: boolean;
  selectedCount: number;
  packageTier: PackageTier;
  expirationDate: string;
}) {
  if (!show) {
    return null;
  }

  const config = StockRankingPackageConfigs.getConfigForTier(packageTier);

  return (
    <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800/50 rounded-lg p-4">
      <h4 className="font-medium text-blue-900 dark:text-blue-300 mb-2">Assignment Summary</h4>
      <div className="text-sm text-blue-800 dark:text-blue-200">
        <div>
          Package: <strong>{packageTier}</strong>
        </div>
        <div>
          Users: <strong>{selectedCount} selected</strong>
        </div>
        <div>
          Max Rankings:{' '}
          <strong>{config.maxRankings === -1 ? 'Unlimited' : config.maxRankings}</strong>
        </div>
        <div>
          Rate Limit: <strong>{config.rateLimitPerMinute} requests/minute</strong>
        </div>
        {expirationDate && (
          <div>
            Expires: <strong>{new Date(expirationDate).toLocaleString()}</strong>
          </div>
        )}
      </div>
    </div>
  );
}

export default function StockRankingPackageAssignment({
  onAssignmentComplete
}: StockRankingPackageAssignmentProps) {
  const [selectedPackage, setSelectedPackage] = useState<PackageTier>(PackageTier.BRONZE);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [assignmentReason, setAssignmentReason] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [expirationDate, setExpirationDate] = useState('');
  const [showPreview, setShowPreview] = useState(false);

  const { users, isLoading: isLoadingUsers, load: loadUsers } = useLoadUsers();

  const { submit: submitAssignment, isLoading: isSubmitting } = useAssignmentSubmit((result) => {
    setSelectedUsers([]);
    setAssignmentReason('');
    setExpirationDate('');
    setShowPreview(false);
    onAssignmentComplete?.(result);
  });

  const filteredUsers = useMemo(
    () =>
      users.filter(
        (user) =>
          user.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          (user.name !== null && user.name.toLowerCase().includes(searchQuery.toLowerCase())) ||
          user.id.toLowerCase().includes(searchQuery.toLowerCase())
      ),
    [users, searchQuery]
  );

  useEffect(() => {
    void loadUsers();
  }, [loadUsers]);

  const handleAssignment = async () => {
    await submitAssignment({
      selectedUsers,
      selectedPackage,
      assignmentReason,
      expirationDate
    });
  };

  const resetForm = () => {
    setSelectedUsers([]);
    setAssignmentReason('');
    setExpirationDate('');
    setShowPreview(false);
  };

  return (
    <div className="space-y-6">
      <div className="border-b border-border pb-4">
        <h2 className="text-2xl font-bold text-foreground">Stock Ranking Package Assignment</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Assign stock ranking access packages to users with specific permissions and limits
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div>
          <PackageSelector selected={selectedPackage} onChange={(tier) => {
            setSelectedPackage(tier);
            setShowPreview(true);
          }} />
          <div className="mt-4">
            <PackagePreview tier={selectedPackage} show={showPreview} />
          </div>
        </div>

        <UserSelector
          users={filteredUsers}
          selectedIds={selectedUsers}
          searchQuery={searchQuery}
          isLoading={isLoadingUsers}
          onSearchChange={setSearchQuery}
          onUserToggle={(id, selected) => {
            setSelectedUsers((prev) =>
              selected ? [...prev, id] : prev.filter((uid) => uid !== id)
            );
          }}
          onSelectAll={() => setSelectedUsers(filteredUsers.map((u) => u.id))}
          onClearAll={() => setSelectedUsers([])}
        />
      </div>

      { }
      <AssignmentForm
        reason={assignmentReason}
        expirationDate={expirationDate}
        notifyUsers={notifyUsers}
        onReasonChange={setAssignmentReason}
        onExpirationChange={setExpirationDate}
        onNotifyChange={setNotifyUsers}
      />

      <AssignmentSummary
        show={selectedUsers.length > 0}
        selectedCount={selectedUsers.length}
        packageTier={selectedPackage}
        expirationDate={expirationDate}
      />

      <div className="flex justify-end space-x-3 pt-4 border-t border-border">
        <button
          type="button"
          onClick={resetForm}
          className="px-4 py-2 text-sm font-medium text-foreground bg-background border border-input rounded-md hover:bg-muted focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Reset
        </button>
        <button
          type="button"
          onClick={() => void handleAssignment()}
          disabled={selectedUsers.length === 0 || !assignmentReason.trim() || isSubmitting}
          className="px-6 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSubmitting ? 'Assigning...' : `Assign ${selectedPackage} Package`}
        </button>
      </div>
    </div>
  );
}