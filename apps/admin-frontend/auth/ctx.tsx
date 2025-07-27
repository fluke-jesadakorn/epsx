'use client';
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { adminLogger } from '@/lib/logger';
import { apiClient, isApiError } from '@epsx/api-client';

interface AdminUser {
  uid: string;
  email: string;
  name?: string;
  roles: string[];
  isAdmin: boolean;
  verified?: boolean;
  disabled?: boolean;
  claims?: {
    role?: string;
  };
  meta?: {
    created?: string;
    lastLogin?: string;
  };
}

interface AdminAuthCtx {
  user: AdminUser | null;
  loading: boolean;
  init: boolean;
  isAdmin: boolean;
  error: string | null;
  login: (token: string) => Promise<void>;
  logout: () => Promise<void>;
  signOut: () => Promise<void>;
  signIn: (email: string, password: string) => Promise<void>;
}

const AuthCtx = createContext<AdminAuthCtx | null>(null);

export function AdminAuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AdminUser | null>(null);
  const [loading, setLoading] = useState(true);
  const [init, setInit] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const login = async (token: string) => {
    try {
      setError(null);
      const response = await apiClient.serverLogin({ token });
      if (isApiError(response)) {
        setError(response.error || 'Login failed');
        return;
      }
      const data = response.data;
      setUser({
        uid: data.user_id,
        email: data.email,
        name: data.displayName,
        roles: [data.role], // Convert single role to array
        isAdmin: data.role === 'admin' || data.role === 'ADMIN',
        verified: true, // Assume verified if profile is accessible
        disabled: false,
        claims: { role: data.role },
        meta: undefined
      });
    } catch (error) {
      adminLogger.error('Admin login failed', { error: error instanceof Error ? error.message : error }, 'AdminAuthProvider.login');
      setError('Network error occurred');
    }
  };

  const signIn = async (email: string, password: string) => {
    try {
      setError(null);
      setLoading(true);
      
      // Use server action for admin login
      const formData = new FormData();
      formData.append('email', email);
      formData.append('password', password);
      
      const { adminLoginAction } = await import('@/lib/actions/auth');
      const result = await adminLoginAction(formData);
      
      if (!result.success) {
        const errorMessage = result.error || 'Invalid credentials';
        setError(errorMessage);
        throw new Error(errorMessage);
      }
      
      const data = result.user;
      if (data) {
        setUser({
          uid: data.uid,
          email: data.email,
          name: data.email, // Use email as display name since displayName doesn't exist
          roles: data.roles || [data.customClaims?.role || 'admin'],
          isAdmin: data.isAdmin,
          verified: true,
          disabled: false,
          claims: data.customClaims,
          meta: undefined // Set to undefined since it doesn't exist in the user object
        });
      }
    } catch (error) {
      adminLogger.error('Admin sign in failed', { error: error instanceof Error ? error.message : error, email }, 'AdminAuthProvider.signIn');
      const errorMessage = error instanceof Error ? error.message : 'Network error occurred';
      setError(errorMessage);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const logout = async () => {
    try {
      const response = await apiClient.logout();
      if (isApiError(response)) {
        adminLogger.error('Admin logout failed', { error: response.error }, 'AdminAuthProvider.logout');
      }
      setUser(null);
      setError(null);
    } catch (error) {
      adminLogger.error('Admin logout failed', { error: error instanceof Error ? error.message : error }, 'AdminAuthProvider.logout');
    }
  };

  const signOut = logout; // Alias for logout

  useEffect(() => {
    const checkExistingSession = async () => {
      try {
        adminLogger.info('Checking existing session...', {}, 'AdminAuthProvider.checkExistingSession');
        
        // Check if admin session cookie exists
        const cookies = document.cookie;
        const hasAdminSession = cookies.includes('admin_sess_id=');
        
        adminLogger.info('Cookie check result', { hasAdminSession, cookies }, 'AdminAuthProvider.checkExistingSession');
        
        if (!hasAdminSession) {
          adminLogger.info('No admin session cookie found', {}, 'AdminAuthProvider.checkExistingSession');
          setLoading(false);
          setInit(true);
          return;
        }
        
        // Try to get admin profile to validate session
        adminLogger.info('Attempting to validate session...', {}, 'AdminAuthProvider.checkExistingSession');
        const { serverGetAdminProfile } = await import('@epsx/api-client');
        const response = await serverGetAdminProfile();
        
        adminLogger.info('Session validation response', { 
          hasData: !!response.data, 
          hasError: !!response.error,
          error: response.error 
        }, 'AdminAuthProvider.checkExistingSession');
        
        if (response.data && !response.error) {
          const data = response.data;
          const userData = {
            uid: data.user_id || data.id,
            email: data.email,
            name: data.name || data.email,
            roles: [data.role || 'admin'],
            isAdmin: true,
            verified: true,
            disabled: false,
            claims: { role: data.role },
            meta: undefined
          };
          
          adminLogger.info('Setting user data', { userData }, 'AdminAuthProvider.checkExistingSession');
          setUser(userData);
        } else {
          adminLogger.warn('Session validation failed', { 
            error: response.error 
          }, 'AdminAuthProvider.checkExistingSession');
        }
      } catch (error) {
        adminLogger.error('Session check failed', { error: error instanceof Error ? error.message : error }, 'AdminAuthProvider.checkExistingSession');
        // Continue with no user - let middleware handle redirects
      } finally {
        setLoading(false);
        setInit(true);
        adminLogger.info('Session check complete', { loading: false, init: true }, 'AdminAuthProvider.checkExistingSession');
      }
    };
    
    checkExistingSession();
  }, []);

  const isAdmin = user?.isAdmin || user?.claims?.role === 'ADMIN' || user?.claims?.role === 'SUPER_ADMIN' || false;

  return (
    <AuthCtx.Provider value={{ 
      user, 
      loading, 
      init, 
      isAdmin, 
      error,
      login, 
      logout, 
      signOut,
      signIn
    }}>
      {children}
    </AuthCtx.Provider>
  );
}

export const useAdminAuth = () => {
  const ctx = useContext(AuthCtx);
  if (!ctx) throw new Error('useAdminAuth must be used within AdminAuthProvider');
  return ctx;
};

// Export the provider with the expected name
export const AppAdminAuthProvider = AdminAuthProvider;