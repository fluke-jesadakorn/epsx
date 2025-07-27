'use server';

import type { TokenFeature } from '@/types/auth/features';
import { Permission } from '@/types/auth/features';
import type { UserRole } from '@/types/auth/roles';
import { isApiError, type AdminUser } from '@epsx/api-client';
import { serverGetAdminUsers, serverSetUserRole, serverGetUserStats } from '@epsx/api-client';

interface User {
  userId: string;
  email?: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
}

export async function fetchUserDetails() {
  try {
    const res = await serverGetAdminUsers();

    if (isApiError(res)) {
      throw new Error(`Failed to fetch users: ${res.error}`);
    }

    if (!res.data) {
      throw new Error('No user data received');
    }

    // Transform AdminUser[] to User[] format for backward compatibility
    const users: User[] = res.data.users.map((u: AdminUser) => ({
      userId: u.uid,
      email: u.email,
      role: (u.role || 'user') as UserRole,
      tokenBalance: 0, // Not available in AdminUser, set default
      features: [], // Not available in AdminUser, set default
      permissions: (u.permissions || [])
        .map(p => p as Permission)
        .filter(p => Object.values(Permission).includes(p)),
    }));

    return users;
  } catch (error) {
    throw error;
  }
}

export async function updateUserRole(uid: string, role: string) {
  try {
    const res = await serverSetUserRole(uid, role);

    if (isApiError(res)) {
      throw new Error(`Failed to update user role: ${res.error}`);
    }

    return { success: true, message: 'User role updated successfully' };
  } catch (error) {
    throw error;
  }
}

export async function getUserStats() {
  try {
    const res = await serverGetUserStats();

    if (isApiError(res)) {
      throw new Error(`Failed to fetch user statistics: ${res.error}`);
    }

    return res.data;
  } catch (error) {
    throw error;
  }
}
