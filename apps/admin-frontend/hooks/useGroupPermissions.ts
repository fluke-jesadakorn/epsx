/**
 * Group-Based Permission Management Hooks
 * New Web3 group-based permission system replacing individual permission management
 * 
 * This file provides hooks for:
 * - Group permission management
 * - User group membership operations
 * - Web3 auto-assignment rules
 * - Group analytics and insights
 * - Permission validation through groups
 */

'use client';

import { useState, useCallback, useMemo, useEffect } from 'react';
import useSWR, { mutate } from 'swr';
import { useAuth } from '@/lib/auth';
import { 
  groupManagementClient,
  PermissionGroup,
  UserGroupMembership,
  Web3AssignmentRule,
  GroupAssignmentHistory,
  CreateGroupRequest,
  UpdateGroupRequest,
  AssignUserToGroupRequest,
  CreateWeb3RuleRequest,
  ProcessWalletRequest,
  BulkProcessRequest,
  GroupAnalytics
} from '@/lib/api/group-management-client';

// ============================================================================
// GROUP PERMISSION TYPES
// ============================================================================

export interface GroupPermissionError {
  message: string;
  code: string;
  details?: any;
}

export interface UserGroupPermissions {
  user_id: string;
  groups: UserGroupMembership[];
  effective_permissions: string[];
  expiring_memberships: UserGroupMembership[];
  total_groups: number;
}

export interface GroupPermissionStats {
  total_groups: number;
  total_active_memberships: number;
  system_groups: number;
  custom_groups: number;
  expiring_soon: number;
  web3_assignments: number;
  recent_activity: number;
  health_score: number;
}

// ============================================================================
// CORE GROUP MANAGEMENT HOOKS
// ============================================================================

/**
 * Hook for managing permission groups
 */
export function usePermissionGroups() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<GroupPermissionError | null>(null);

  const { 
    data: groups, 
    error: fetchError, 
    isLoading: isLoadingGroups, 
    mutate: refreshGroups 
  } = useSWR<PermissionGroup[]>('permission-groups', () => 
    groupManagementClient.getPermissionGroups()
  );

  const systemGroups = useMemo(() => 
    groups?.filter(g => g.is_system_group) || [], [groups]);
  
  const customGroups = useMemo(() => 
    groups?.filter(g => !g.is_system_group) || [], [groups]);

  const createGroup = useCallback(async (request: CreateGroupRequest): Promise<PermissionGroup> => {
    setLoading(true);
    setError(null);
    
    try {
      const newGroup = await groupManagementClient.createPermissionGroup(request);
      await refreshGroups();
      await mutate('group-analytics'); // Refresh analytics
      return newGroup;
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to create group',
        code: 'CREATE_GROUP_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [refreshGroups]);

  const updateGroup = useCallback(async (groupId: string, request: UpdateGroupRequest): Promise<PermissionGroup> => {
    setLoading(true);
    setError(null);
    
    try {
      const updatedGroup = await groupManagementClient.updatePermissionGroup(groupId, request);
      await refreshGroups();
      return updatedGroup;
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to update group',
        code: 'UPDATE_GROUP_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [refreshGroups]);

  const deleteGroup = useCallback(async (groupId: string): Promise<void> => {
    setLoading(true);
    setError(null);
    
    try {
      await groupManagementClient.deletePermissionGroup(groupId);
      await refreshGroups();
      await mutate('group-analytics'); // Refresh analytics
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to delete group',
        code: 'DELETE_GROUP_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [refreshGroups]);

  return {
    groups: groups || [],
    systemGroups,
    customGroups,
    isLoading: isLoadingGroups || loading,
    error: fetchError || error,
    createGroup,
    updateGroup,
    deleteGroup,
    refreshGroups
  };
}

/**
 * Hook for managing user group memberships
 */
export function useUserGroupMemberships(userId: string | null) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<GroupPermissionError | null>(null);

  const { 
    data: memberships, 
    error: fetchError, 
    isLoading: isLoadingMemberships, 
    mutate: refreshMemberships 
  } = useSWR<UserGroupMembership[]>(
    userId ? `user-groups-${userId}` : null,
    () => userId ? groupManagementClient.getUserGroups(userId) : []
  );

  const { 
    data: permissions, 
    error: permissionsError, 
    isLoading: isLoadingPermissions, 
    mutate: refreshPermissions 
  } = useSWR<string[]>(
    userId ? `user-permissions-${userId}` : null,
    () => userId ? groupManagementClient.getUserPermissions(userId) : []
  );

  const activeMemberships = useMemo(() => 
    memberships?.filter(m => m.is_active) || [], [memberships]);
  
  const expiringMemberships = useMemo(() => {
    const now = Date.now();
    const sevenDaysFromNow = now + (7 * 24 * 60 * 60 * 1000);
    return activeMemberships.filter(m => 
      m.expires_at && new Date(m.expires_at).getTime() <= sevenDaysFromNow
    );
  }, [activeMemberships]);

  const assignUserToGroup = useCallback(async (request: AssignUserToGroupRequest): Promise<void> => {
    setLoading(true);
    setError(null);
    
    try {
      await groupManagementClient.assignUserToGroup(request);
      await refreshMemberships();
      await refreshPermissions();
      await mutate('group-analytics'); // Refresh analytics
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to assign user to group',
        code: 'ASSIGN_GROUP_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [refreshMemberships, refreshPermissions]);

  const removeUserFromGroup = useCallback(async (groupId: string): Promise<void> => {
    if (!userId) throw new Error('User ID is required');
    
    setLoading(true);
    setError(null);
    
    try {
      await groupManagementClient.removeUserFromGroup(userId, groupId);
      await refreshMemberships();
      await refreshPermissions();
      await mutate('group-analytics'); // Refresh analytics
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to remove user from group',
        code: 'REMOVE_GROUP_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [userId, refreshMemberships, refreshPermissions]);

  const hasPermission = useCallback((permission: string): boolean => {
    if (!permissions) return false;
    
    // Support wildcard matching
    return permissions.some(p => {
      if (p === permission) return true;
      if (p.endsWith('*')) {
        const prefix = p.slice(0, -1);
        return permission.startsWith(prefix);
      }
      return false;
    });
  }, [permissions]);

  const hasAnyPermission = useCallback((permissionList: string[]): boolean => {
    return permissionList.some(permission => hasPermission(permission));
  }, [hasPermission]);

  const hasAllPermissions = useCallback((permissionList: string[]): boolean => {
    return permissionList.every(permission => hasPermission(permission));
  }, [hasPermission]);

  return {
    memberships: memberships || [],
    activeMemberships,
    expiringMemberships,
    permissions: permissions || [],
    isLoading: isLoadingMemberships || isLoadingPermissions || loading,
    error: fetchError || permissionsError || error,
    assignUserToGroup,
    removeUserFromGroup,
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    refreshMemberships,
    refreshPermissions
  };
}

/**
 * Hook for managing Web3 assignment rules
 */
export function useWeb3AssignmentRules() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<GroupPermissionError | null>(null);

  const { 
    data: rules, 
    error: fetchError, 
    isLoading: isLoadingRules, 
    mutate: refreshRules 
  } = useSWR<Web3AssignmentRule[]>('web3-assignment-rules', () => 
    groupManagementClient.getWeb3AssignmentRules()
  );

  const activeRules = useMemo(() => 
    rules?.filter(r => r.is_active) || [], [rules]);

  const createRule = useCallback(async (request: CreateWeb3RuleRequest): Promise<Web3AssignmentRule> => {
    setLoading(true);
    setError(null);
    
    try {
      const newRule = await groupManagementClient.createWeb3AssignmentRule(request);
      await refreshRules();
      return newRule;
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to create Web3 assignment rule',
        code: 'CREATE_WEB3_RULE_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [refreshRules]);

  const deleteRule = useCallback(async (ruleId: string): Promise<void> => {
    setLoading(true);
    setError(null);
    
    try {
      await groupManagementClient.deleteWeb3AssignmentRule(ruleId);
      await refreshRules();
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to delete Web3 assignment rule',
        code: 'DELETE_WEB3_RULE_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [refreshRules]);

  const processWallet = useCallback(async (walletAddress: string): Promise<string[]> => {
    setLoading(true);
    setError(null);
    
    try {
      const assignedGroups = await groupManagementClient.processWalletAssignment({ wallet_address: walletAddress });
      await mutate('group-analytics'); // Refresh analytics
      return assignedGroups;
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to process wallet assignment',
        code: 'PROCESS_WALLET_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  const verifyWalletAssets = useCallback(async (walletAddress: string): Promise<any> => {
    setLoading(true);
    setError(null);
    
    try {
      const assets = await groupManagementClient.verifyWalletAssets(walletAddress);
      return assets;
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to verify wallet assets',
        code: 'VERIFY_WALLET_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  const bulkProcessWallets = useCallback(async (walletAddresses: string[]): Promise<any> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await groupManagementClient.bulkProcessWallets({ wallet_addresses: walletAddresses });
      await mutate('group-analytics'); // Refresh analytics
      return result;
    } catch (err) {
      const error = { 
        message: err instanceof Error ? err.message : 'Failed to bulk process wallets',
        code: 'BULK_PROCESS_FAILED' 
      };
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    rules: rules || [],
    activeRules,
    isLoading: isLoadingRules || loading,
    error: fetchError || error,
    createRule,
    deleteRule,
    processWallet,
    verifyWalletAssets,
    bulkProcessWallets,
    refreshRules
  };
}

/**
 * Hook for group analytics and insights
 */
export function useGroupAnalytics(refreshInterval: number = 30000) {
  const { 
    data: analytics, 
    error, 
    isLoading, 
    mutate: refreshAnalytics 
  } = useSWR<GroupAnalytics>('group-analytics', () => 
    groupManagementClient.getGroupAnalytics(),
    { refreshInterval }
  );

  const stats: GroupPermissionStats = useMemo(() => {
    if (!analytics) {
      return {
        total_groups: 0,
        total_active_memberships: 0,
        system_groups: 0,
        custom_groups: 0,
        expiring_soon: 0,
        web3_assignments: 0,
        recent_activity: 0,
        health_score: 0
      };
    }

    return {
      total_groups: analytics.total_groups,
      total_active_memberships: analytics.total_active_memberships,
      system_groups: 7, // Fixed number of system groups
      custom_groups: analytics.total_groups - 7,
      expiring_soon: analytics.expiring_soon_count,
      web3_assignments: 0, // TODO: Add to analytics
      recent_activity: 0, // TODO: Add to analytics
      health_score: Math.min(100, Math.floor((analytics.total_active_memberships / Math.max(1, analytics.total_groups)) * 10))
    };
  }, [analytics]);

  const { 
    data: expiringMemberships, 
    mutate: refreshExpiring 
  } = useSWR<UserGroupMembership[]>('expiring-memberships', () => 
    groupManagementClient.getExpiringMemberships(7)
  );

  const cleanupExpiredMemberships = useCallback(async (): Promise<{ removed_count: number }> => {
    const result = await groupManagementClient.cleanupExpiredMemberships();
    await refreshAnalytics();
    await refreshExpiring();
    return result;
  }, [refreshAnalytics, refreshExpiring]);

  return {
    analytics,
    stats,
    expiringMemberships: expiringMemberships || [],
    isLoading,
    error,
    cleanupExpiredMemberships,
    refreshAnalytics,
    refreshExpiring
  };
}


// ============================================================================
// PERMISSION VALIDATION HOOKS
// ============================================================================

/**
 * Hook for checking user permissions through group system
 */
export function useGroupPermissionCheck(userId: string | null, permission: string) {
  const { 
    data: hasPermission, 
    error, 
    isLoading 
  } = useSWR<boolean>(
    userId && permission ? `check-permission-${userId}-${permission}` : null,
    () => userId ? groupManagementClient.checkUserPermission(userId, permission) : false
  );

  return {
    hasPermission: hasPermission || false,
    isLoading,
    error
  };
}

/**
 * Hook for admin group permission management
 */
export function useAdminGroupPermissions() {
  const { user } = useAuth();
  const userId = user?.wallet_address;

  const { hasPermission, hasAnyPermission, hasAllPermissions } = useUserGroupMemberships(userId || null);

  const isAdmin = useMemo(() => 
    hasAnyPermission(['admin:*:*', 'admin:users:manage', 'admin:system:manage']), 
    [hasAnyPermission]);

  const isSuperAdmin = useMemo(() => 
    hasPermission('admin:*:*'), 
    [hasPermission]);

  const canManageUsers = useMemo(() => 
    hasAnyPermission(['admin:users:manage', 'admin:*:*']), 
    [hasAnyPermission]);

  const canManageGroups = useMemo(() => 
    hasAnyPermission(['admin:permissions:manage', 'admin:groups:manage', 'admin:*:*']), 
    [hasAnyPermission]);

  const canManageWeb3Rules = useMemo(() => 
    hasAnyPermission(['admin:web3:manage', 'admin:*:*']), 
    [hasAnyPermission]);

  const canViewAuditTrail = useMemo(() => 
    hasAnyPermission(['admin:audit:view', 'admin:users:manage', 'admin:*:*']), 
    [hasAnyPermission]);

  return {
    isAdmin,
    isSuperAdmin,
    canManageUsers,
    canManageGroups,
    canManageWeb3Rules,
    canViewAuditTrail,
    hasPermission,
    hasAnyPermission,
    hasAllPermissions
  };
}

// ============================================================================
// UTILITY HOOKS
// ============================================================================

/**
 * Hook for group assignment history with filtering and pagination
 */
export function useGroupAssignmentHistory(filters?: {
  operation_type?: string;
  operation_source?: string;
  group_id?: string;
  user_search?: string;
  date_from?: string;
  date_to?: string;
  limit?: number;
  offset?: number;
}) {
  const { 
    data, 
    error, 
    isLoading, 
    mutate: refetchHistory 
  } = useSWR(
    filters ? `group-assignment-history-${JSON.stringify(filters)}` : 'group-assignment-history',
    () => groupManagementClient.getGroupAssignmentHistory(filters)
  );

  const history = useMemo(() => data?.data?.history || [], [data]);
  const totalCount = useMemo(() => data?.data?.total || 0, [data]);

  const refresh = useCallback(() => {
    refetchHistory();
  }, [refetchHistory]);

  return {
    history,
    totalCount,
    isLoading,
    error,
    refresh
  };
}

/**
 * Hook for available permissions list
 */
export function useAvailablePermissions() {
  const { 
    data: permissions, 
    error, 
    isLoading 
  } = useSWR<string[]>('available-permissions', () => 
    groupManagementClient.getAvailablePermissions()
  );

  return {
    permissions: permissions || [],
    isLoading,
    error
  };
}

/**
 * Main group permissions hook - combines all functionality
 */
export function useGroupPermissionManagement() {
  const groups = usePermissionGroups();
  const web3Rules = useWeb3AssignmentRules();
  const analytics = useGroupAnalytics();
  const adminPermissions = useAdminGroupPermissions();

  return {
    ...groups,
    ...web3Rules,
    ...analytics,
    ...adminPermissions,
    // Prefixed to avoid conflicts
    groupData: groups,
    web3Data: web3Rules,
    analyticsData: analytics
  };
}

// Default export for main usage
export default useGroupPermissionManagement;