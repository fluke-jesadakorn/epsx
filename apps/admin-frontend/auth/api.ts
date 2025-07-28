import { adminLogin, logout, getCurrentUser } from '@epsx/server-actions';

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

export const adminAuthApi = {
  login: async (req: AdminLoginReq): Promise<AdminUsrRes> => {
    try {
      // The token would be credentials like { email, password }
      // For now, we'll need to adapt this based on the actual auth flow
      const response = await adminLogin({ email: '', password: req.token });
      
      // Transform response to match expected format
      return {
        user: {
          id: response.user_id || '',
          email: response.email || '',
          roles: [response.role || 'user'],
          isAdmin: ['admin', 'super_admin', 'SuperAdmin'].includes(response.role || '')
        }
      };
    } catch (error) {
      throw new Error('Admin login failed');
    }
  },

  logout: async (): Promise<void> => {
    try {
      await logout();
    } catch (error) {
      throw new Error('Admin logout failed');
    }
  },

  me: async (): Promise<AdminUsrRes> => {
    try {
      const response = await getCurrentUser();
      
      if (!response) {
        throw new Error('No user data');
      }
      
      // Transform response to match expected format
      return {
        user: {
          id: response.id || response.user_id || '',
          email: response.email || '',
          roles: [response.role || 'user'],
          isAdmin: ['admin', 'super_admin', 'SuperAdmin'].includes(response.role || '')
        }
      };
    } catch (error) {
      throw new Error('Admin auth check failed');
    }
  },
};