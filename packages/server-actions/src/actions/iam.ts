'use server';

import { serverGet, serverPost, serverPut, serverDelete } from '../core/request';
import { withServerAction, type ServerActionResult } from '../core/error-handler';
import { validateSchema, updateUserTierSchema } from '../core/validation';

// IAM User Actions
export async function getIAMUsers(filters?: {
  packageTier?: string;
  subscriptionStatus?: string;
  hasCustomPermissions?: boolean;
  limit?: number;
  offset?: number;
}) {
  try {
    // TODO: Backend endpoint /api/v1/iam/users not implemented yet
    // Return empty array for now
    return [];
  } catch (error) {
    console.error('Error fetching IAM users:', error);
    return [];
  }
}

export async function getIAMUser(uid: string) {
  try {
    return await serverGet(`/api/v1/iam/users/${uid}`);
  } catch (error) {
    console.error('Error fetching IAM user:', error);
    throw error;
  }
}

export async function getUserEffectivePermissions(uid: string) {
  try {
    return await serverGet(`/api/v1/iam/users/${uid}/effective-permissions`);
  } catch (error) {
    console.error('Error fetching effective permissions:', error);
    return [];
  }
}

export async function updateUserTier(data: {
  userId: string;
  newTier: string;
  updatedBy: string;
  reason?: string;
}): Promise<ServerActionResult<any>> {
  return withServerAction(async () => {
    const validatedData = validateSchema(updateUserTierSchema, data, 'updateUserTier');
    return await serverPut(`/api/v1/iam/users/${validatedData.userId}/tier`, validatedData);
  }, 'updateUserTier');
}

export async function bulkUserUpdate(data: {
  userIds: string[];
  updates: {
    packageTier?: string;
    status?: string;
    roles?: string[];
  };
  updatedBy: string;
  reason?: string;
}) {
  try {
    return await serverPost('/api/v1/iam/users/bulk-update', data);
  } catch (error) {
    console.error('Error bulk updating users:', error);
    throw error;
  }
}

export async function updateUserStatus(data: {
  userId: string;
  status: 'active' | 'suspended' | 'deactivated';
  updatedBy: string;
  reason?: string;
}) {
  try {
    return await serverPut(`/api/v1/iam/users/${data.userId}/status`, data);
  } catch (error) {
    console.error('Error updating user status:', error);
    throw error;
  }
}

// IAM Role Actions
export async function getIAMRoles() {
  try {
    // TODO: Backend endpoint /api/v1/iam/roles exists but may not return expected format
    // Return empty array for now
    return [];
  } catch (error) {
    console.error('Error fetching IAM roles:', error);
    return [];
  }
}

export async function createIAMRole(data: {
  name: string;
  description?: string;
  packageTier: string;
  policies: string[];
  inlinePermissions: any[];
  assignable: boolean;
}) {
  try {
    return await serverPost('/api/v1/iam/roles', data);
  } catch (error) {
    console.error('Error creating IAM role:', error);
    throw error;
  }
}

export async function updateIAMRole(roleId: string, data: any) {
  try {
    return await serverPut(`/api/v1/iam/roles/${roleId}`, data);
  } catch (error) {
    console.error('Error updating IAM role:', error);
    throw error;
  }
}

export async function deleteIAMRole(roleId: string) {
  try {
    return await serverDelete(`/api/v1/iam/roles/${roleId}`);
  } catch (error) {
    console.error('Error deleting IAM role:', error);
    throw error;
  }
}

// IAM Policy Actions
export async function getIAMPolicies() {
  try {
    // TODO: Backend endpoint /api/v1/iam/policies exists but may not return expected format
    // Return empty array for now
    return [];
  } catch (error) {
    console.error('Error fetching IAM policies:', error);
    return [];
  }
}

export async function createIAMPolicy(data: {
  name: string;
  description?: string;
  document: any;
}) {
  try {
    return await serverPost('/api/v1/iam/policies', data);
  } catch (error) {
    console.error('Error creating IAM policy:', error);
    throw error;
  }
}

// Permission Actions
export async function getCustomPermissions() {
  try {
    // For now, return mock data since backend endpoint doesn't exist yet
    return [];
  } catch (error) {
    console.error('Error fetching custom permissions:', error);
    return [];
  }
}

export async function grantCustomPermission(data: {
  userId: string;
  featureId: string;
  permission: string;
  grantedBy: string;
  expiresAt?: Date;
  reason?: string;
}) {
  try {
    return await serverPost('/api/v1/iam/permissions/grant', data);
  } catch (error) {
    console.error('Error granting custom permission:', error);
    throw error;
  }
}

export async function revokeCustomPermission(data: {
  permissionId: string;
  revokedBy: string;
  reason?: string;
}) {
  try {
    return await serverPost('/api/v1/iam/permissions/revoke', data);
  } catch (error) {
    console.error('Error revoking custom permission:', error);
    throw error;
  }
}

export async function evaluatePermission(data: {
  userId: string;
  action: string;
  resource: string;
}) {
  try {
    return await serverPost('/api/v1/iam/permissions/evaluate', data);
  } catch (error) {
    console.error('Error evaluating permission:', error);
    return { allowed: false };
  }
}

// Package Management Actions
export async function applyPackagePermissions(data: {
  userId: string;
  packageTier: string;
}) {
  try {
    return await serverPost('/api/v1/iam/packages/apply', data);
  } catch (error) {
    console.error('Error applying package permissions:', error);
    throw error;
  }
}

export async function previewPackageUpgrade(data: {
  userId: string;
  targetTier: string;
}) {
  try {
    return await serverPost('/api/v1/iam/packages/preview', data);
  } catch (error) {
    console.error('Error previewing package upgrade:', error);
    return {
      currentPermissions: [],
      newPermissions: [],
      addedPermissions: [],
      removedPermissions: []
    };
  }
}

// Bulk Operations
export async function bulkApplyPermissionProfile(data: {
  userIds: string[];
  profileId: string;
  appliedBy: string;
}) {
  try {
    return await serverPost('/api/v1/iam/bulk/apply-profile', data);
  } catch (error) {
    console.error('Error bulk applying permission profile:', error);
    throw error;
  }
}

// Audit Actions
export async function getUserAuditLogs(uid: string, limit: number = 50) {
  try {
    return await serverGet(`/api/v1/iam/audit/users/${uid}`, { limit });
  } catch (error) {
    console.error('Error fetching user audit logs:', error);
    return [];
  }
}

export async function getAllAuditLogs(limit: number = 100) {
  try {
    return await serverGet('/api/v1/iam/audit/logs', { limit });
  } catch (error) {
    console.error('Error fetching audit logs:', error);
    return [];
  }
}

// Maintenance Actions
export async function cleanupExpiredPermissions() {
  try {
    return await serverPost('/api/v1/iam/maintenance/cleanup-expired');
  } catch (error) {
    console.error('Error cleaning up expired permissions:', error);
    throw error;
  }
}