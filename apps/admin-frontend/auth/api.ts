import { createApiClient, isApiError } from '@epsx/api-client';

interface AdminLoginReq {
  token: string;
}

interface AdminUsrRes {
  user: {
    id: string;
    email: string;
    roles: string[];
    isAdmin: boolean;
  };
}

// Get API client - will automatically use backend URL
const getApi = () => {
  return createApiClient();
};

export const adminAuthApi = {
  login: async (req: AdminLoginReq): Promise<AdminUsrRes> => {
    try {
      const api = getApi();
      const response = await api.loginAdmin({ token: req.token });
      
      if (isApiError(response)) {
        throw new Error(response.error || 'Admin login failed');
      }
      
      // Transform response to match expected format
      return {
        user: {
          id: response.data?.user_id || '',
          email: response.data?.email || '',
          roles: [response.data?.role || 'user'],
          isAdmin: ['admin', 'super_admin', 'SuperAdmin'].includes(response.data?.role || '')
        }
      };
    } catch (error) {
      throw new Error('Admin login failed');
    }
  },

  logout: async (): Promise<void> => {
    try {
      const api = getApi();
      const response = await api.logoutAdmin();
      
      if (isApiError(response)) {
        throw new Error(response.error || 'Admin logout failed');
      }
    } catch (error) {
      throw new Error('Admin logout failed');
    }
  },

  me: async (): Promise<AdminUsrRes> => {
    try {
      const api = getApi();
      const response = await api.getAdminProfile();
      
      if (isApiError(response)) {
        throw new Error(response.error || 'Admin auth check failed');
      }
      
      // Transform response to match expected format
      return {
        user: {
          id: response.data?.uid || '',
          email: response.data?.email || '',
          roles: [response.data?.role || 'user'],
          isAdmin: ['admin', 'super_admin', 'SuperAdmin'].includes(response.data?.role || '')
        }
      };
    } catch (error) {
      throw new Error('Admin auth check failed');
    }
  },
};