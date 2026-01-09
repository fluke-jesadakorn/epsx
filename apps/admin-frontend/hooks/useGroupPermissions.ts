/**
 * Permission Groups Management Hooks
 * Provides hooks for managing permission groups and available permissions
 */

import { useCallback, useEffect, useMemo, useState } from 'react';

import { AssignUserToGroupRequest, CreateGroupRequest, Group, GroupAssignmentHistory, groupMgmt, UpdateGroupRequest, UserGroupMembership } from '@/lib/api/group-management-client';

// ============================================================================
// PERMISSION GROUPS HOOK
// ============================================================================

interface UseGroupsReturn {
  groups: Group[];
  loading: boolean;
  error: string | null;
  createGroup: (request: CreateGroupRequest) => Promise<Group>;
  updateGroup: (groupId: string, request: UpdateGroupRequest) => Promise<Group>;
  deleteGroup: (groupId: string) => Promise<void>;
  refreshGroups: () => Promise<void>;
}

export type UsePermissionGroupsReturn = UseGroupsReturn;

export const useGroupPermissions = useGroups;

/**
 *
 */
export function useGroups(): UseGroupsReturn {
  const [groups, setGroups] = useState<Group[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // const client = new GroupManagementClient(createAdminApiClient()); // Removed

  const loadGroups = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const fetchedGroups = await groupMgmt.getGroups();
      setGroups(fetchedGroups);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load permission groups');
    } finally {
      setLoading(false);
    }
  }, []);

  const createGroup = useCallback(async (request: CreateGroupRequest): Promise<Group> => {
    try {
      setError(null);
      const newGroup = await groupMgmt.createGroup(request);
      // Refresh groups after creation
      await loadGroups();
      return newGroup;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create group';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadGroups]);

  const updateGroup = useCallback(async (groupId: string, request: UpdateGroupRequest): Promise<Group> => {
    try {
      setError(null);
      const updatedGroup = await groupMgmt.updateGroup(groupId, request);
      // Refresh groups after update
      await loadGroups();
      return updatedGroup;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update group';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadGroups]);

  const deleteGroup = useCallback(async (groupId: string): Promise<void> => {
    try {
      setError(null);
      await groupMgmt.deleteGroup(groupId);
      // Refresh groups after deletion
      await loadGroups();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete group';
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

/**
 *
 */
export function useAvailablePermissions(): UseAvailablePermissionsReturn {
  const [permissions, setPermissions] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // const client = new GroupManagementClient(createAdminApiClient()); // Removed

  const loadPermissions = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const fetchedPermissions = await groupMgmt.getAvailablePermissions();
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

/**
 *
 */
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

  // const client = new GroupManagementClient(createAdminApiClient()); // removed

  const loadStats = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // For now, provide basic analytics - can be enhanced later
      const groups = await groupMgmt.getGroups();
      const totalGroups = groups.length;
      const activeGroups = groups.filter((g: Group) => g.is_active).length; // DB schema has is_active
      const systemGroups = groups.filter((g: Group) => (g.group_type === 'system')).length;
      const totalPermissions = groups.reduce((sum: number, g: Group) => sum + g.permissions.length, 0);

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

/**
 *
 */
export function useWeb3AssignmentRules(): UseWeb3AssignmentRulesReturn {
  const [rules, setRules] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // const client = new GroupManagementClient(createAdminApiClient()); // removed

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

  const processWallet = useCallback(async (_walletAddress: string): Promise<void> => {
    try {
      setError(null);
      // For now, just log - would need specific API endpoint
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to process wallet');
      throw err;
    }
  }, []);

  const verifyWalletAssets = useCallback(async (_walletAddress: string): Promise<boolean> => {
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

// Backward compatibility exports
export const usePermissionGroups = useGroups;

/**
 *
 * @param userId
 */
export function useUserGroupMemberships(userId: string | null) {
  const [memberships, setMemberships] = useState<UserGroupMembership[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadMemberships = useCallback(async () => {
    if (!userId) {
      setMemberships([]);
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      const data = await groupMgmt.getUserGroups(userId);
      setMemberships(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load user memberships');
      // setMemberships([]); // Keep existing on error?
    } finally {
      setIsLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    loadMemberships();
  }, [loadMemberships]);

  const activeMemberships = useMemo(() =>
    memberships.filter(m => m.is_active),
    [memberships]
  );

  const expiringMemberships = useMemo(() => {
    const now = Date.now();
    const sevenDaysFromNow = now + (7 * 24 * 60 * 60 * 1000);
    return memberships.filter(m =>
      m.is_active &&
      m.expires_at &&
      new Date(m.expires_at).getTime() <= sevenDaysFromNow
    );
  }, [memberships]);

  const assignUserToGroup = useCallback(async (request: AssignUserToGroupRequest) => {
    await groupMgmt.assignUserToGroup(request);
    await loadMemberships();
  }, [loadMemberships]);

  const removeUserFromGroup = useCallback(async (groupId: string) => {
    if (!userId) {return;}
    await groupMgmt.removeUserFromGroup(userId, groupId);
    await loadMemberships();
  }, [userId, loadMemberships]);

  const refreshMemberships = loadMemberships;

  return {
    memberships,
    activeMemberships,
    expiringMemberships,
    isLoading,
    error,
    assignUserToGroup,
    removeUserFromGroup,
    refreshMemberships,
  };
}

// ============================================================================
// GROUP MEMBERS HOOK
// ============================================================================

/**
 *
 * @param groupId
 */
export function useGroupMembers(groupId: string | null) {
  const [members, setMembers] = useState<UserGroupMembership[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadMembers = useCallback(async () => {
    if (!groupId) {
      setMembers([]);
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      const data = await groupMgmt.getGroupMemberships(groupId);
      setMembers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load group members');
    } finally {
      setIsLoading(false);
    }
  }, [groupId]);

  useEffect(() => {
    loadMembers();
  }, [loadMembers]);

  return {
    members,
    isLoading,
    error,
    refreshMembers: loadMembers,
  };
}

// ============================================================================
// GROUP ASSIGNMENT HISTORY HOOK (STUB)
// ============================================================================

/**
 *
 */
export function useGroupAssignmentHistory() {
  return {
    history: [] as GroupAssignmentHistory[],
    isLoading: false,
    refreshHistory: async () => { },
  };
}

// ============================================================================
// COMBINED HOOKS EXPORT (BACKWARD COMPATIBILITY)
// ============================================================================

export const groupHooks = {
  useGroups,
  usePermissionGroups, // Alias
  useAvailablePermissions,
  useGroupAnalytics,
  // Removed: useAdminGroupPermissions - backend handles permission checking
  useWeb3AssignmentRules,
  useUserGroupMemberships,
  useGroupAssignmentHistory,
  useGroupMembers,
};

export default groupHooks;