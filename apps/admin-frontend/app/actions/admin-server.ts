'use server';

// Import and re-export server actions from the shared package
import { 
  createUser as _createUser,
  updateUser as _updateUser,
  deleteUser as _deleteUser
} from '@epsx/server-actions';

export { 
  _createUser as createUser,
  _updateUser as updateUser,
  _deleteUser as deleteUser
};

import type { TokenFeature } from '@/types/auth/features';
import { Permission } from '@/types/auth/features';
import type { UserRole } from '@/types/auth/roles';
import type { AdminUser } from '@epsx/api-client';
import { isApiError, serverGetAdminUsers, serverSetUserRole, serverGetUserStats as _serverGetUserStats } from '@epsx/api-client';

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
    // Use direct fetch approach for debugging
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();
    const allCookies = cookieStore.getAll();
    
    // Build cookie header manually
    const cookieHeader = allCookies.map(c => `${c.name}=${c.value}`).join('; ');
    
    // Direct fetch to backend
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const url = `${backendUrl}/api/admin/analytics/user-statistics?include_roles=true&include_tiers=true`;
    
    
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
      },
    });
    
    
    if (!response.ok) {
      const errorText = await response.text();
      console.error('[getUserStats] Error response:', errorText);
      throw new Error(`HTTP ${response.status}: ${errorText || 'Request failed'}`);
    }
    
    const data = await response.json();
    return data;
    
  } catch (error) {
    console.error('[getUserStats] Exception:', error);
    throw error;
  }
}

// IAM Server Actions
async function makeIAMRequest(endpoint: string, options: RequestInit = {}) {
  const { cookies } = await import('next/headers');
  const cookieStore = await cookies();
  const allCookies = cookieStore.getAll();
  const cookieHeader = allCookies.map(c => `${c.name}=${c.value}`).join('; ');
  
  const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
  const url = `${backendUrl}/api/v1/iam/${endpoint}`;
  
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'Cookie': cookieHeader,
      ...options.headers,
    },
  });
  
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errorText || 'Request failed'}`);
  }
  
  return response.json();
}

export async function getIAMUsers(filters?: any) {
  try {
    const queryParams = new URLSearchParams();
    if (filters) {
      Object.keys(filters).forEach(key => {
        if (filters[key] !== undefined) {
          queryParams.append(key, filters[key]);
        }
      });
    }
    
    const endpoint = `users${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return await makeIAMRequest(endpoint);
  } catch (error) {
    console.error('Error fetching IAM users:', error);
    return [];
  }
}

export async function getIAMRoles() {
  try {
    return await makeIAMRequest('roles');
  } catch (error) {
    console.error('Error fetching IAM roles:', error);
    return [];
  }
}

export async function getIAMPolicies() {
  try {
    return await makeIAMRequest('policies');
  } catch (error) {
    console.error('Error fetching IAM policies:', error);
    return [];
  }
}
