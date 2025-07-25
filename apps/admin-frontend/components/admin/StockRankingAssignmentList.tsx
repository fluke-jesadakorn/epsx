'use client';

import React, { useState, useEffect } from 'react';
import { 
  PackageTier, 
  StockRankingPackageAssignment,
  StockRankingUsageStats 
} from '@epsx/types/src/permission_profile';

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
  refreshTrigger 
}: StockRankingAssignmentListProps) {
  const [assignments, setAssignments] = useState<AssignmentWithUser[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [filter, setFilter] = useState({
    packageTier: '',
    status: '',
    search: ''
  });
  const [sortBy, setSortBy] = useState<'assignedAt' | 'expiresAt' | 'usageStats'>('assignedAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  useEffect(() => {
    loadAssignments();
  }, [refreshTrigger]);

  const loadAssignments = async () => {
    setIsLoading(true);
    try {
      const response = await fetch('/api/admin/stock-ranking/assignments');
      const data = await response.json();
      setAssignments(data.assignments || []);
    } catch (error) {
      console.error('Failed to load assignments:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRevokeAssignment = async (assignmentId: string) => {
    if (!confirm('Are you sure you want to revoke this assignment?')) {
      return;
    }

    try {
      const response = await fetch(`/api/admin/stock-ranking/assignments/${assignmentId}/revoke`, {
        method: 'POST',
      });

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

  const handleExtendAssignment = async (assignmentId: string, newExpirationDate: string) => {
    try {
      const response = await fetch(`/api/admin/stock-ranking/assignments/${assignmentId}/extend`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          expiresAt: new Date(newExpirationDate).toISOString(),
        }),
      });

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
    const matchesPackage = !filter.packageTier || assignment.packageTier === filter.packageTier;
    const matchesStatus = !filter.status || assignment.status === filter.status;
    const matchesSearch = !filter.search || 
      assignment.user?.email.toLowerCase().includes(filter.search.toLowerCase()) ||
      assignment.user?.name?.toLowerCase().includes(filter.search.toLowerCase()) ||
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
      active: 'bg-green-100 text-green-800',
      expired: 'bg-red-100 text-red-800',
      revoked: 'bg-gray-100 text-gray-800',
      suspended: 'bg-yellow-100 text-yellow-800'
    };

    return colors[status as keyof typeof colors] || 'bg-gray-100 text-gray-800';
  };

  const getTierBadge = (tier: PackageTier) => {
    const colors = {
      [PackageTier.FREE]: 'bg-gray-100 text-gray-800',
      [PackageTier.BRONZE]: 'bg-orange-100 text-orange-800',
      [PackageTier.SILVER]: 'bg-gray-300 text-gray-800',
      [PackageTier.GOLD]: 'bg-yellow-100 text-yellow-800',
      [PackageTier.PLATINUM]: 'bg-purple-100 text-purple-800',
      [PackageTier.ENTERPRISE]: 'bg-blue-100 text-blue-800'
    };

    return colors[tier] || 'bg-gray-100 text-gray-800';
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
        <div className="text-gray-500">Loading assignments...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium text-gray-900">
          Current Stock Ranking Assignments
        </h3>
        <button
          onClick={loadAssignments}
          className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200"
        >
          Refresh
        </button>
      </div>

      {/* Filters */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Package Tier
          </label>
          <select
            value={filter.packageTier}
            onChange={(e) => setFilter(prev => ({ ...prev, packageTier: e.target.value }))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
          >
            <option value="">All Tiers</option>
            {Object.values(PackageTier).map(tier => (
              <option key={tier} value={tier}>{tier}</option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Status
          </label>
          <select
            value={filter.status}
            onChange={(e) => setFilter(prev => ({ ...prev, status: e.target.value }))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
          >
            <option value="">All Statuses</option>
            <option value="active">Active</option>
            <option value="expired">Expired</option>
            <option value="revoked">Revoked</option>
            <option value="suspended">Suspended</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Sort By
          </label>
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
          >
            <option value="assignedAt">Assignment Date</option>
            <option value="expiresAt">Expiration Date</option>
            <option value="usageStats">Usage</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Search
          </label>
          <input
            type="text"
            placeholder="User email or reason..."
            value={filter.search}
            onChange={(e) => setFilter(prev => ({ ...prev, search: e.target.value }))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
          />
        </div>
      </div>

      {/* Statistics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-white p-4 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">{assignments.length}</div>
          <div className="text-sm text-gray-600">Total Assignments</div>
        </div>
        <div className="bg-white p-4 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-green-600">
            {assignments.filter(a => a.status === 'active').length}
          </div>
          <div className="text-sm text-gray-600">Active</div>
        </div>
        <div className="bg-white p-4 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-yellow-600">
            {assignments.filter(a => isExpiringSoon(a)).length}
          </div>
          <div className="text-sm text-gray-600">Expiring Soon</div>
        </div>
        <div className="bg-white p-4 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-red-600">
            {assignments.filter(a => a.status === 'expired').length}
          </div>
          <div className="text-sm text-gray-600">Expired</div>
        </div>
      </div>

      {/* Assignments Table */}
      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        <ul className="divide-y divide-gray-200">
          {sortedAssignments.length === 0 ? (
            <li className="px-6 py-8 text-center text-gray-500">
              No assignments found matching your criteria
            </li>
          ) : (
            sortedAssignments.map((assignment) => (
              <li key={assignment.id} className="px-6 py-4">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-3">
                      <div className="flex-shrink-0">
                        <span className={`px-2 py-1 text-xs font-medium rounded-full ${getTierBadge(assignment.packageTier)}`}>
                          {assignment.packageTier}
                        </span>
                      </div>
                      
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900 truncate">
                          {assignment.user?.name || assignment.user?.email || assignment.userId}
                        </p>
                        <p className="text-sm text-gray-500 truncate">
                          {assignment.reason}
                        </p>
                      </div>
                    </div>
                    
                    <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                      <span>
                        Assigned: {new Date(assignment.assignedAt).toLocaleDateString()}
                      </span>
                      {assignment.expiresAt && (
                        <span className={isExpiringSoon(assignment) ? 'text-yellow-600 font-medium' : ''}>
                          Expires: {new Date(assignment.expiresAt).toLocaleDateString()}
                        </span>
                      )}
                      <span>
                        Usage: {assignment.usageStats.apiCallsUsed} API calls, {assignment.usageStats.rankingsViewed} rankings viewed
                      </span>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-3">
                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusBadge(assignment.status)}`}>
                      {assignment.status}
                    </span>
                    
                    {assignment.status === 'active' && (
                      <div className="flex space-x-2">
                        <button
                          onClick={() => {
                            const newDate = prompt('Enter new expiration date (YYYY-MM-DD HH:MM):');
                            if (newDate) {
                              handleExtendAssignment(assignment.id, newDate);
                            }
                          }}
                          className="text-blue-600 hover:text-blue-800 text-sm"
                        >
                          Extend
                        </button>
                        <button
                          onClick={() => handleRevokeAssignment(assignment.id)}
                          className="text-red-600 hover:text-red-800 text-sm"
                        >
                          Revoke
                        </button>
                      </div>
                    )}
                  </div>
                </div>

                {/* Stock Ranking Config Preview */}
                <div className="mt-3 bg-gray-50 rounded p-3">
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs">
                    <div>
                      <span className="text-gray-600">Max Rankings:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.maxRankings === -1 
                          ? 'Unlimited' 
                          : assignment.stockRankingConfig.maxRankings}
                      </div>
                    </div>
                    <div>
                      <span className="text-gray-600">Rate Limit:</span>
                      <div className="font-medium">{assignment.stockRankingConfig.rateLimitPerMinute}/min</div>
                    </div>
                    <div>
                      <span className="text-gray-600">Real-time:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.realTimeUpdates ? 'Yes' : 'No'}
                      </div>
                    </div>
                    <div>
                      <span className="text-gray-600">Markets:</span>
                      <div className="font-medium">
                        {assignment.stockRankingConfig.allowedMarkets.includes('*') 
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