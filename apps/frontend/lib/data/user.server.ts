import { createApiClient, isApiError } from '@/lib/api-client';
import { getBackendUrl } from '../../../../shared/utils/url-resolver';

const BACKEND_URL = getBackendUrl('server');
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