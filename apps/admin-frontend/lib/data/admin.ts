import { createApiClient, isApiError, type AdminUser, type PermissionProfile, type StockRankingAssignment, type AnalyticsStatistics, type StockRankingAnalytics, type AdminProfile } from '@epsx/api-client';
import { adminLogger } from '../logger';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || process.env.BACKEND_URL;

if (!BACKEND_URL) {
  throw new Error('BACKEND_URL or NEXT_PUBLIC_API_URL environment variable is required');
}

const apiClient = createApiClient(BACKEND_URL);

// User Management
export async function getUsers(searchParams?: URLSearchParams) {
  try {
    const url = `/admin/users${searchParams ? `?${searchParams.toString()}` : ''}`;
    
    adminLogger.info('Fetching users data', { url }, 'AdminDataLayer');
    
    const response = await apiClient.getAdminUsers(searchParams);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch users', { error: response.error, details: response.details }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch users');
    }

    adminLogger.debug('Users data fetched successfully', { userCount: response.data.users?.length || 0 }, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Users fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getUser(userId: string) {
  try {
    adminLogger.info('Fetching user data', { userId }, 'AdminDataLayer');

    const response = await apiClient.getAdminUser(userId);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch user', { error: response.error, details: response.details, userId }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch user');
    }

    adminLogger.debug('User data fetched successfully', { userId }, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('User fetch error', { 
      error: error instanceof Error ? error.message : String(error),
      userId 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Permission Profiles
export async function getPermissionProfiles(searchParams?: URLSearchParams) {
  try {
    adminLogger.info('Fetching permission profiles', { searchParams: searchParams?.toString() }, 'AdminDataLayer');
    
    const response = await apiClient.getAdminPermissionProfiles(searchParams);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch permission profiles', { error: response.error, details: response.details }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch permission profiles');
    }

    adminLogger.debug('Permission profiles fetched successfully', { profileCount: response.data.permission_profiles?.length || 0 }, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Permission profiles fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getPermissionProfile(profileId: string) {
  try {
    adminLogger.info('Fetching permission profile', { profileId }, 'AdminDataLayer');

    const response = await apiClient.getAdminPermissionProfile(profileId);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch permission profile', { error: response.error, details: response.details, profileId }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch permission profile');
    }

    adminLogger.debug('Permission profile fetched successfully', { profileId }, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Permission profile fetch error', { 
      error: error instanceof Error ? error.message : String(error),
      profileId 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Stock Ranking
export async function getStockRankingAssignments(searchParams?: URLSearchParams) {
  try {
    adminLogger.info('Fetching stock ranking assignments', { searchParams: searchParams?.toString() }, 'AdminDataLayer');
    
    const response = await apiClient.getStockRankingAssignments(searchParams);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch stock ranking assignments', { error: response.error, details: response.details }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch stock ranking assignments');
    }

    adminLogger.debug('Stock ranking assignments fetched successfully', { assignmentCount: response.data.assignments?.length || 0 }, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Stock ranking assignments fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getStockRankingAssignment(assignmentId: string) {
  try {
    adminLogger.info('Fetching stock ranking assignment', { assignmentId }, 'AdminDataLayer');

    const response = await apiClient.getStockRankingAssignment(assignmentId);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch stock ranking assignment', { error: response.error, details: response.details, assignmentId }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch stock ranking assignment');
    }

    adminLogger.debug('Stock ranking assignment fetched successfully', { assignmentId }, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Stock ranking assignment fetch error', { 
      error: error instanceof Error ? error.message : String(error),
      assignmentId 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Analytics
export async function getAnalyticsStatistics() {
  try {
    adminLogger.info('Fetching analytics statistics', {}, 'AdminDataLayer');

    const response = await apiClient.getAnalyticsStatistics();

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch analytics statistics', { error: response.error, details: response.details }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch analytics statistics');
    }

    adminLogger.debug('Analytics statistics fetched successfully', {}, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Analytics statistics fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

export async function getStockRankingAnalytics(searchParams?: URLSearchParams) {
  try {
    adminLogger.info('Fetching stock ranking analytics', { searchParams: searchParams?.toString() }, 'AdminDataLayer');
    
    const response = await apiClient.getStockRankingAnalytics(searchParams);

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch stock ranking analytics', { error: response.error, details: response.details }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch stock ranking analytics');
    }

    adminLogger.debug('Stock ranking analytics fetched successfully', {}, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Stock ranking analytics fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}

// Admin Profile
export async function getAdminProfile() {
  try {
    adminLogger.info('Fetching admin profile', {}, 'AdminDataLayer');

    const response = await apiClient.getAdminProfile();

    if (isApiError(response)) {
      adminLogger.error('Failed to fetch admin profile', { error: response.error, details: response.details }, 'AdminDataLayer');
      throw new Error(response.error || 'Failed to fetch admin profile');
    }

    adminLogger.debug('Admin profile fetched successfully', {}, 'AdminDataLayer');
    return response.data;
  } catch (error) {
    adminLogger.error('Admin profile fetch error', { 
      error: error instanceof Error ? error.message : String(error) 
    }, 'AdminDataLayer');
    throw error;
  }
}