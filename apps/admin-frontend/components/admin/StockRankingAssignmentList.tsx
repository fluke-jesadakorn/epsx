'use client';

import {
  PackageTier,
  StockRankingPackageAssignment,
} from '@/types';
import { useEffect, useState } from 'react';

interface User {
  id: string;
  email: string;
  name?: string;
}

interface AssignmentWithUser extends StockRankingPackageAssignment {
  user?: User;
}

interface StockRankingAssignmentListProps {
  refreshTrigger?: number;
}

export default function StockRankingAssignmentList({
  refreshTrigger,
}: StockRankingAssignmentListProps) {
  const [assignments, setAssignments] = useState<AssignmentWithUser[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [filter, setFilter] = useState({
    packageTier: '',
    status: '',
    search: '',
  });
  const [sortBy, setSortBy] = useState<
    'assignedAt' | 'expiresAt' | 'usageStats'
  >('assignedAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const _setSortOrder = setSortOrder;

  useEffect(() => {
    loadAssignments();
  }, [refreshTrigger]);

  const loadAssignments = async () => {
    setIsLoading(true);
    try {
      const response = await fetch('/api/v1/admin/stock-ranking/assignments');
      
      if (!response.ok) {
        console.error('Assignment API error:', response.status, response.statusText);
        setAssignments([]);
        return;
      }
      
      const contentType = response.headers.get('content-type');
      if (!contentType?.includes('application/json')) {
        console.error('Invalid response type:', contentType);
        setAssignments([]);
        return;
      }
      
      const data = await response.json();
      setAssignments(data.assignments || []);
    } catch (error) {
      console.error('Failed to load assignments:', error);
      setAssignments([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRevokeAssignment = async (assignmentId: string) => {
    if (!confirm('Are you sure you want to revoke this assignment?')) {
      return;
    }

    try {
      const response = await fetch(
        `/api/v1/admin/stock-ranking/assignments/${assignmentId}/revoke`,
        {
          method: 'POST',
        }
      );

      if (response.ok) {
        await loadAssignments();
        alert('Assignment revoked successfully');
      } else {
        throw new Error('Failed to revoke assignment');
      }
    } catch (error) {
      console.error('Error revoking assignment:', error);
      alert('Failed to revoke assignment');
    }
  };

  const handleExtendAssignment = async (
    assignmentId: string,
    newExpirationDate: string
  ) => {
    try {
      const response = await fetch(
        `/api/v1/admin/stock-ranking/assignments/${assignmentId}/extend`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            expiresAt: new Date(newExpirationDate).toISOString(),
          }),
        }
      );

      if (response.ok) {
        await loadAssignments();
        alert('Assignment extended successfully');
      } else {
        throw new Error('Failed to extend assignment');
      }
    } catch (error) {
      console.error('Error extending assignment:', error);
      alert('Failed to extend assignment');
    }
  };

  const filteredAssignments = assignments.filter(assignment => {
    const matchesPackage =
      !filter.packageTier || assignment.packageTier === filter.packageTier;
    const matchesStatus = !filter.status || assignment.status === filter.status;
    const matchesSearch =
      !filter.search ||
      assignment.user?.email
        .toLowerCase()
        .includes(filter.search.toLowerCase()) ||
      assignment.user?.name
        ?.toLowerCase()
        .includes(filter.search.toLowerCase()) ||
      assignment.reason.toLowerCase().includes(filter.search.toLowerCase());

    return matchesPackage && matchesStatus && matchesSearch;
  });

  const sortedAssignments = [...filteredAssignments].sort((a, b) => {
    let aValue: any, bValue: any;

    switch (sortBy) {
      case 'assignedAt':
        aValue = new Date(a.assignedAt).getTime();
        bValue = new Date(b.assignedAt).getTime();
        break;
      case 'expiresAt':
        aValue = a.expiresAt ? new Date(a.expiresAt).getTime() : 0;
        bValue = b.expiresAt ? new Date(b.expiresAt).getTime() : 0;
        break;
      case 'usageStats':
        aValue = a.usageStats.apiCallsUsed;
        bValue = b.usageStats.apiCallsUsed;
        break;
      default:
        return 0;
    }

    if (sortOrder === 'asc') {
      return aValue > bValue ? 1 : -1;
    } else {
      return aValue < bValue ? 1 : -1;
    }
  });

  const getStatusBadge = (status: string) => {
    const colors = {
      active: 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300',
      expired: 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300',
      revoked: 'bg-muted text-muted-foreground',
      suspended: 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300',
    };

    return colors[status as keyof typeof colors] || 'bg-muted text-muted-foreground';
  };

  const getTierBadge = (tier: PackageTier) => {
    const colors = {
      [PackageTier.FREE]: 'bg-muted text-muted-foreground',
      [PackageTier.BRONZE]: 'bg-orange-100 dark:bg-orange-900/30 text-orange-800 dark:text-orange-300',
      [PackageTier.SILVER]: 'bg-gray-300 dark:bg-gray-700 text-gray-800 dark:text-gray-300',
      [PackageTier.GOLD]: 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300',
      [PackageTier.PLATINUM]: 'bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300',
      [PackageTier.ENTERPRISE]: 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300',
    };

    return colors[tier] || 'bg-muted text-muted-foreground';
  };

  const isExpiringSoon = (assignment: AssignmentWithUser) => {
    if (!assignment.expiresAt) return false;
    const expirationDate = new Date(assignment.expiresAt);
    const now = new Date();
    const sevenDaysFromNow = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000);
    return expirationDate <= sevenDaysFromNow && expirationDate > now;
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center py-12">
        <div className="text-muted-foreground">Loading assignments...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium text-foreground">
          Current Stock Ranking Assignments
        </h3>
        <button
          onClick={loadAssignments}
          className="px-3 py-1 text-sm bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded hover:bg-blue-200 dark:hover:bg-blue-900/50"
        >
          Refresh
        </button>
      </div>

      {/* Filters */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div>
          <label className="block text-sm font-medium text-foreground mb-1">
            Package Tier
          </label>
          <select
            value={filter.packageTier}
            onChange={e =>
              setFilter(prev => ({ ...prev, packageTier: e.target.value }))
            }
            className="w-full px-3 py-2 border border-input rounded-md text-sm bg-background text-foreground"
          >
            <option value="">All Tiers</option>
            {Object.values(PackageTier).map(tier => (
              <option key={tier} value={tier}>
                {tier}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-foreground mb-1">
            Status
          </label>
          <select
            value={filter.status}
            onChange={e =>
              setFilter(prev => ({ ...prev, status: e.target.value }))
            }
            className="w-full px-3 py-2 border border-input rounded-md text-sm bg-background text-foreground"
          >
            <option value="">All Statuses</option>
            <option value="active">Active</option>
            <option value="expired">Expired</option>
            <option value="revoked">Revoked</option>
            <option value="suspended">Suspended</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-foreground mb-1">
            Sort By
          </label>
          <select
            value={sortBy}
            onChange={e => setSortBy(e.target.value as any)}
            className="w-full px-3 py-2 border border-input rounded-md text-sm bg-background text-foreground"
          >
            <option value="assignedAt">Assignment Date</option>
            <option value="expiresAt">Expiration Date</option>
            <option value="usageStats">Usage</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-foreground mb-1">
            Search
          </label>
          <input
            type="text"
            placeholder="User email or reason..."
            value={filter.search}
            onChange={e =>
              setFilter(prev => ({ ...prev, search: e.target.value }))
            }
            className="w-full px-3 py-2 border border-input rounded-md text-sm bg-background text-foreground placeholder:text-muted-foreground"
          />
        </div>
      </div>

      {/* Statistics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="pancake-card p-4">
          <div className="text-2xl font-bold text-foreground">
            {assignments.length}
          </div>
          <div className="text-sm text-muted-foreground">Total Assignments</div>
        </div>
        <div className="pancake-card p-4">
          <div className="text-2xl font-bold text-green-600 dark:text-green-400">
            {assignments.filter(a => a.status === 'active').length}
          </div>
          <div className="text-sm text-muted-foreground">Active</div>
        </div>
        <div className="pancake-card p-4">
          <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
            {assignments.filter(a => isExpiringSoon(a)).length}
          </div>
          <div className="text-sm text-muted-foreground">Expiring Soon</div>
        </div>
        <div className="pancake-card p-4">
          <div className="text-2xl font-bold text-red-600 dark:text-red-400">
            {assignments.filter(a => a.status === 'expired').length}
          </div>
          <div className="text-sm text-muted-foreground">Expired</div>
        </div>
      </div>

      {/* Assignments Table */}
      <div className="pancake-card overflow-hidden">
        <ul className="divide-y divide-border">
          {sortedAssignments.length === 0 ? (
            <li className="px-6 py-8 text-center text-muted-foreground">
              No assignments found matching your criteria
            </li>
          ) : (
            sortedAssignments.map(assignment => (
              <li key={assignment.id} className="px-6 py-4 hover:bg-muted/50">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-3">
                      <div className="flex-shrink-0">
                        <span
                          className={`px-2 py-1 text-xs font-medium rounded-full ${getTierBadge(assignment.packageTier)}`}
                        >
                          {assignment.packageTier}
                        </span>
                      </div>

                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-foreground truncate">
                          {assignment.user?.name ||
                            assignment.user?.email ||
                            assignment.userId}
                        </p>
                        <p className="text-sm text-muted-foreground truncate">
                          {assignment.reason}
                        </p>
                      </div>
                    </div>

                    <div className="mt-2 flex items-center space-x-4 text-xs text-muted-foreground">
                      <span>
                        Assigned:{' '}
                        {new Date(assignment.assignedAt).toLocaleDateString()}
                      </span>
                      {assignment.expiresAt && (
                        <span
                          className={
                            isExpiringSoon(assignment)
                              ? 'text-yellow-600 dark:text-yellow-400 font-medium'
                              : ''
                          }
                        >
                          Expires:{' '}
                          {new Date(assignment.expiresAt).toLocaleDateString()}
                        </span>
                      )}
                      <span>
                        Usage: {assignment.usageStats.apiCallsUsed} API calls,{' '}
                        {assignment.usageStats.rankingsViewed} rankings viewed
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center space-x-3">
                    <span
                      className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusBadge(assignment.status)}`}
                    >
                      {assignment.status}
                    </span>

                    {assignment.status === 'active' && (
                      <div className="flex space-x-2">
                        <button
                          onClick={() => {
                            const newDate = prompt(
                              'Enter new expiration date (YYYY-MM-DD HH:MM):'
                            );
                            if (newDate) {
                              handleExtendAssignment(assignment.id, newDate);
                            }
                          }}
                          className="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 text-sm"
                        >
                          Extend
                        </button>
                        <button
                          onClick={() => handleRevokeAssignment(assignment.id)}
                          className="text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300 text-sm"
                        >
                          Revoke
                        </button>
                      </div>
                    )}
                  </div>
                </div>

                {/* Stock Ranking Config Preview */}
                <div className="mt-3 bg-muted rounded p-3">
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs">
                    <div>
                      <span className="text-muted-foreground">Max Rankings:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.maxRankings === -1
                          ? 'Unlimited'
                          : assignment.stockRankingConfig.maxRankings}
                      </div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Rate Limit:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.rateLimitPerMinute}/min
                      </div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Real-time:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.realTimeUpdates
                          ? 'Yes'
                          : 'No'}
                      </div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Markets:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.allowedMarkets.includes(
                          '*'
                        )
                          ? 'All'
                          : assignment.stockRankingConfig.allowedMarkets.length}
                      </div>
                    </div>
                  </div>
                </div>
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  );
}
