/**
 * Permission Groups Management Hooks
 * Provides hooks for managing permission groups and available permissions
 */

import { useState, useEffect, useCallback } from 'react';
import { GroupManagementClient, PermissionGroup, CreateGroupRequest, UpdateGroupRequest } from '@/lib/api/group-management-client';
import { createAdminApiClient } from '@/shared/utils/api-client';

// ============================================================================
// PERMISSION GROUPS HOOK
// ============================================================================

interface UsePermissionGroupsReturn {
  groups: PermissionGroup[];
  loading: boolean;
  error: string | null;
  createGroup: (request: CreateGroupRequest) => Promise<PermissionGroup>;
  updateGroup: (groupId: string, request: UpdateGroupRequest) => Promise<PermissionGroup>;
  deleteGroup: (groupId: string) => Promise<void>;
  refreshGroups: () => Promise<void>;
}

export function usePermissionGroups(): UsePermissionGroupsReturn {
  const [groups, setGroups] = useState<PermissionGroup[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = new GroupManagementClient(createAdminApiClient());

  const loadGroups = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const fetchedGroups = await client.getPermissionGroups();
      setGroups(fetchedGroups);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load permission groups');
    } finally {
      setLoading(false);
    }
  }, []);

  const createGroup = useCallback(async (request: CreateGroupRequest): Promise<PermissionGroup> => {
    try {
      setError(null);
      const newGroup = await client.createPermissionGroup(request);
      // Refresh groups after creation
      await loadGroups();
      return newGroup;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create permission group';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadGroups]);

  const updateGroup = useCallback(async (groupId: string, request: UpdateGroupRequest): Promise<PermissionGroup> => {
    try {
      setError(null);
      const updatedGroup = await client.updatePermissionGroup(groupId, request);
      // Refresh groups after update
      await loadGroups();
      return updatedGroup;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update permission group';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadGroups]);

  const deleteGroup = useCallback(async (groupId: string): Promise<void> => {
    try {
      setError(null);
      await client.deletePermissionGroup(groupId);
      // Refresh groups after deletion
      await loadGroups();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete permission group';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadGroups]);

  // Load groups on mount
  useEffect(() => {
    loadGroups();
  }, [loadGroups]);

  return {
    groups,
    loading,
    error,
    createGroup,
    updateGroup,
    deleteGroup,
    refreshGroups: loadGroups,
  };
}

// ============================================================================
// AVAILABLE PERMISSIONS HOOK
// ============================================================================

interface UseAvailablePermissionsReturn {
  permissions: string[];
  isLoading: boolean;
  error: string | null;
  refreshPermissions: () => Promise<void>;
}

export function useAvailablePermissions(): UseAvailablePermissionsReturn {
  const [permissions, setPermissions] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = new GroupManagementClient(createAdminApiClient());

  const loadPermissions = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const fetchedPermissions = await client.getAvailablePermissions();
      setPermissions(fetchedPermissions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load available permissions');
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load permissions on mount
  useEffect(() => {
    loadPermissions();
  }, [loadPermissions]);

  return {
    permissions,
    isLoading,
    error,
    refreshPermissions: loadPermissions,
  };
}

// ============================================================================
// GROUP ANALYTICS HOOK
// ============================================================================

interface GroupAnalyticsStats {
  totalGroups: number;
  activeGroups: number;
  systemGroups: number;
  totalUsers: number;
  avgPermissionsPerGroup: number;
  recentActivity: number;
  health_score: number;
}

interface UseGroupAnalyticsReturn {
  stats: GroupAnalyticsStats;
  loading: boolean;
  error: string | null;
  refreshStats: () => Promise<void>;
}

export function useGroupAnalytics(): UseGroupAnalyticsReturn {
  const [stats, setStats] = useState<GroupAnalyticsStats>({
    totalGroups: 0,
    activeGroups: 0,
    systemGroups: 0,
    totalUsers: 0,
    avgPermissionsPerGroup: 0,
    recentActivity: 0,
    health_score: 0,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = new GroupManagementClient(createAdminApiClient());

  const loadStats = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      
      // For now, provide basic analytics - can be enhanced later
      const groups = await client.getPermissionGroups();
      const totalGroups = groups.length;
      const activeGroups = groups.filter(g => g.is_system_group === false).length;
      const systemGroups = groups.filter(g => g.is_system_group === true).length;
      const totalPermissions = groups.reduce((sum, g) => sum + g.permissions.length, 0);
      
      setStats({
        totalGroups,
        activeGroups,
        systemGroups,
        totalUsers: 0, // Would need separate API call
        avgPermissionsPerGroup: totalGroups > 0 ? Math.round(totalPermissions / totalGroups) : 0,
        recentActivity: 0, // Would need separate API call
        health_score: totalGroups > 0 ? Math.round((activeGroups / totalGroups) * 100) : 100,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load group analytics');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  return {
    stats,
    loading,
    error,
    refreshStats: loadStats,
  };
}

// ============================================================================
// ADMIN GROUP PERMISSIONS HOOK (REMOVED)
// Backend handles all permission checking - no client-side validation
// Admin UI components rely on backend errors for permission enforcement
// ============================================================================

// ============================================================================
// WEB3 ASSIGNMENT RULES HOOK
// ============================================================================

interface UseWeb3AssignmentRulesReturn {
  rules: any[]; // Would be Web3AssignmentRule[] in full implementation
  loading: boolean;
  error: string | null;
  processWallet: (walletAddress: string) => Promise<void>;
  verifyWalletAssets: (walletAddress: string) => Promise<boolean>;
  refreshRules: () => Promise<void>;
}

export function useWeb3AssignmentRules(): UseWeb3AssignmentRulesReturn {
  const [rules, setRules] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = new GroupManagementClient(createAdminApiClient());

  const loadRules = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      // For now, return empty rules - would need specific API endpoint
      setRules([]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load Web3 assignment rules');
    } finally {
      setLoading(false);
    }
  }, []);

  const processWallet = useCallback(async (walletAddress: string): Promise<void> => {
    try {
      setError(null);
      // For now, just log - would need specific API endpoint
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to process wallet');
      throw err;
    }
  }, []);

  const verifyWalletAssets = useCallback(async (walletAddress: string): Promise<boolean> => {
    try {
      setError(null);
      // For now, return true - would need actual verification
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to verify wallet assets');
      return false;
    }
  }, []);

  useEffect(() => {
    loadRules();
  }, [loadRules]);

  return {
    rules,
    loading,
    error,
    processWallet,
    verifyWalletAssets,
    refreshRules: loadRules,
  };
}

// ============================================================================
// USER GROUP MEMBERSHIPS HOOK (STUB)
// ============================================================================

import type { UserGroupMembership } from '@/lib/api/group-management-client';

interface AssignmentHistoryItem {
  id: string;
  user_id: string;
  group_id: string;
  action: 'assigned' | 'removed';
  performed_by: string;
  performed_at: string;
  reason?: string;
}

export function useUserGroupMemberships(userId: string | null) {
  return {
    memberships: [] as UserGroupMembership[],
    activeMemberships: [] as UserGroupMembership[],
    expiringMemberships: [] as UserGroupMembership[],
    isLoading: false,
    assignUserToGroup: async (request: any) => {},
    removeUserFromGroup: async (membershipId: string) => {},
    refreshMemberships: async () => {},
  };
}

// ============================================================================
// GROUP ASSIGNMENT HISTORY HOOK (STUB)
// ============================================================================

export function useGroupAssignmentHistory() {
  return {
    history: [] as AssignmentHistoryItem[],
    isLoading: false,
    refreshHistory: async () => {},
  };
}

// ============================================================================
// COMBINED HOOKS EXPORT (BACKWARD COMPATIBILITY)
// ============================================================================

export const useGroupPermissions = {
  usePermissionGroups,
  useAvailablePermissions,
  useGroupAnalytics,
  // Removed: useAdminGroupPermissions - backend handles permission checking
  useWeb3AssignmentRules,
  useUserGroupMemberships,
  useGroupAssignmentHistory,
};

export default useGroupPermissions;