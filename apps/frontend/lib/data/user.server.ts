import { createApiClient, isApiError } from '@/lib/api-client';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const apiClient = createApiClient(BACKEND_URL);

export async function getCurrentUser() {
  try {
    const response = await apiClient.serverGetCurrentUser();

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch current user');
    }

    return response.data;
  } catch (error) {
    console.error('Current user fetch error:', error);
    throw error;
  }
}

export async function getUserProfile() {
  try {
    const response = await apiClient.getCurrentUser();

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch user profile');
    }

    return response.data;
  } catch (error) {
    console.error('User profile fetch error:', error);
    throw error;
  }
}

export async function getAuditLogs(searchParams?: URLSearchParams) {
  try {
    const queryString = searchParams?.toString() || '';
    const endpoint = `/audit/logs${queryString ? `?${queryString}` : ''}`;
    
    const response = await apiClient.serverGetAuditLogs(endpoint);

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch audit logs');
    }

    return response.data;
  } catch (error) {
    console.error('Audit logs fetch error:', error);
    throw error;
  }
}

export async function getPremiumRankings() {
  try {
    const response = await apiClient.serverGetPremiumRankings();

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch premium rankings');
    }

    return response.data;
  } catch (error) {
    console.error('Premium rankings fetch error:', error);
    throw error;
  }
}