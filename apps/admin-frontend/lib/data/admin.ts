import { createApiClient } from '@/lib/api-client';
import { env } from '@/config/env';

// Types for API responses
interface AdminUser {
  id: string;
  email: string;
  name?: string;
  roles: string[];
  admin_modules: string[];
  status: 'active' | 'inactive' | 'disabled';
  created_at: string;
  updated_at: string;
}

interface AdminUsersResponse {
  users: AdminUser[];
  total: number;
  page: number;
  page_size: number;
}

interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  is_system: boolean;
  created_at: string;
}

interface AnalyticsStats {
  total_users: number;
  active_users: number;
  user_growth: number;
  package_distribution: Record<string, number>;
}

interface StockRankingAssignment {
  id: string;
  user_id: string;
  package_tier_id: string;
  assignment_type: string;
  status: 'active' | 'expired' | 'pending';
  assigned_at: string;
  expires_at?: string;
}

interface StockRankingAnalytics {
  total_assignments: number;
  active_assignments: number;
  assignment_growth: number;
  tier_distribution: Record<string, number>;
}

// Get authentication token for admin API calls
function getAuthToken(): string | null {
  if (typeof window === 'undefined') {
    return null; // Server-side - token should be passed differently
  }
  return localStorage.getItem('admin_jwt') || sessionStorage.getItem('admin_jwt');
}

// Create admin API client with backend URL and auth
const getApiClient = () => {
  const token = getAuthToken();
  return createApiClient(env.NEXT_PUBLIC_BACKEND_URL, token || undefined);
};

const isApiError = (response: any) => !response.success && response.error;

// User Management
export async function getUsers(searchParams?: URLSearchParams): Promise<AdminUsersResponse> {
  try {
    const params = searchParams ? `?${searchParams.toString()}` : '';
    const response = await getApiClient().get<AdminUsersResponse>(`/api/v1/admin/users${params}`);

    if (isApiError(response)) {
      console.error('Failed to fetch users', { error: response.error }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch users');
    }

    return response.data;
  } catch (error) {
    console.error('Users fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getUser(userId: string): Promise<AdminUser> {
  try {
    const response = await getApiClient().get<AdminUser>(`/api/v1/admin/users/${userId}`);

    if (isApiError(response)) {
      console.error('Failed to fetch user', { error: response.error, userId }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch user');
    }

    return response.data;
  } catch (error) {
    console.error('User fetch error', { 
      error: error instanceof Error ? error.message : String(error),
      userId 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Permission Profiles
export async function getPermissionProfiles(searchParams?: URLSearchParams): Promise<PermissionProfile[]> {
  try {
    const params = searchParams ? `?${searchParams.toString()}` : '';
    const response = await getApiClient().get<PermissionProfile[]>(`/api/v1/admin/permission-profiles${params}`);

    if (isApiError(response)) {
      console.error('Failed to fetch permission profiles', { error: response.error }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch permission profiles');
    }

    return response.data;
  } catch (error) {
    console.error('Permission profiles fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getPermissionProfile(profileId: string): Promise<PermissionProfile> {
  try {
    const response = await getApiClient().get<PermissionProfile>(`/api/v1/admin/permission-profiles/${profileId}`);

    if (isApiError(response)) {
      console.error('Failed to fetch permission profile', { error: response.error, profileId }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch permission profile');
    }

    return response.data;
  } catch (error) {
    console.error('Permission profile fetch error', { 
      error: error instanceof Error ? error.message : String(error),
      profileId 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Stock Ranking
export async function getStockRankingAssignments(searchParams?: URLSearchParams): Promise<StockRankingAssignment[]> {
  try {
    const params = searchParams ? `?${searchParams.toString()}` : '';
    const response = await getApiClient().get<StockRankingAssignment[]>(`/api/v1/admin/stock-ranking/assignments${params}`);

    if (isApiError(response)) {
      console.error('Failed to fetch stock ranking assignments', { error: response.error }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch stock ranking assignments');
    }

    return response.data;
  } catch (error) {
    console.error('Stock ranking assignments fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getStockRankingAssignment(assignmentId: string): Promise<StockRankingAssignment> {
  try {
    const response = await getApiClient().get<StockRankingAssignment>(`/api/v1/admin/stock-ranking/assignments/${assignmentId}`);

    if (isApiError(response)) {
      console.error('Failed to fetch stock ranking assignment', { error: response.error, assignmentId }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch stock ranking assignment');
    }

    return response.data;
  } catch (error) {
    console.error('Stock ranking assignment fetch error', { 
      error: error instanceof Error ? error.message : String(error),
      assignmentId 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Analytics
export async function getAnalyticsStatistics(): Promise<AnalyticsStats> {
  try {
    const response = await getApiClient().get<AnalyticsStats>('/api/v1/admin/analytics');

    if (isApiError(response)) {
      console.error('Failed to fetch analytics statistics', { error: response.error }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch analytics statistics');
    }

    return response.data;
  } catch (error) {
    console.error('Analytics statistics fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getStockRankingAnalytics(searchParams?: URLSearchParams): Promise<StockRankingAnalytics> {
  try {
    const params = searchParams ? `?${searchParams.toString()}` : '';
    const response = await getApiClient().get<StockRankingAnalytics>(`/api/v1/admin/stock-ranking/analytics${params}`);

    if (isApiError(response)) {
      console.error('Failed to fetch stock ranking analytics', { error: response.error }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch stock ranking analytics');
    }

    return response.data;
  } catch (error) {
    console.error('Stock ranking analytics fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Admin Profile
export async function getAdminProfile(): Promise<AdminUser> {
  try {
    const response = await getApiClient().get<AdminUser>('/api/v1/admin/profile');

    if (isApiError(response)) {
      console.error('Failed to fetch admin profile', { error: response.error }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch admin profile');
    }

    return response.data;
  } catch (error) {
    console.error('Admin profile fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}