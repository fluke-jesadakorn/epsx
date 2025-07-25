import { createApiClient, isApiError } from '@epsx/api-client';

interface LoginReq {
  token: string;
}

interface UsrRes {
  user: {
    id: string;
    email: string;
    roles: string[];
  };
}

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';
const apiClient = createApiClient(BACKEND_URL);

export const authApi = {
  login: async (req: LoginReq): Promise<UsrRes> => {
    const response = await apiClient.login({ type: 'token', token: req.token });
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Login failed');
    }
    
    // Transform response to match expected interface
    return {
      user: {
        id: response.data.user_id,
        email: response.data.email,
        roles: response.data.role ? [response.data.role] : [],
      }
    };
  },

  logout: async (): Promise<void> => {
    const response = await apiClient.logout();
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Logout failed');
    }
  },

  me: async (): Promise<UsrRes> => {
    const response = await apiClient.getCurrentUser();
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Auth check failed');
    }
    
    // Transform response to match expected interface
    return {
      user: {
        id: response.data.user_id,
        email: response.data.email,
        roles: response.data.role ? [response.data.role] : [],
      }
    };
  },
};