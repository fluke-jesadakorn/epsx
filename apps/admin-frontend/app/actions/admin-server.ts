'use server';

import { createApiClient, isApiError, type AdminUser } from '@epsx/api-client';
import { config } from '@/lib/config';
import type { TokenFeature, Permission } from '@/types/auth/features';
import type { UserRole } from '@/types/auth/roles';

interface User {
  userId: string;
  email?: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
}

// Get API client server-side only
const getApi = () => {
  const url = config.getBackendUrl();
  return createApiClient(url);
};

export async function fetchUserDetails() {
  try {
    const res = await getApi().getAdminUsers();

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
      permissions: (u.permissions || []).map(p => ({ id: p, name: p, description: '', resource: '', action: '', risk: 'low' as const })),
    }));

    return users;
  } catch (error) {
    throw error;
  }
}

export async function updateUserRole(uid: string, role: string) {
  try {
    const res = await getApi().setUserRole(uid, role);

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
    const res = await getApi().getUserStats();

    if (isApiError(res)) {
      throw new Error(`Failed to fetch user statistics: ${res.error}`);
    }

    return res.data;
  } catch (error) {
    throw error;
  }
}
