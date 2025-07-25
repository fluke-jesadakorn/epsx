'use client';
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { adminLogger } from '@/lib/logger';
import { apiClient, isApiError } from '@epsx/api-client';

interface AdminUser {
  uid: string;
  email: string;
  displayName?: string;
  roles: string[];
  isAdmin: boolean;
  emailVerified?: boolean;
  disabled?: boolean;
  customClaims?: {
    role?: string;
  };
  metadata?: {
    creationTime?: string;
    lastSignInTime?: string;
  };
}

interface AdminAuthCtx {
  user: AdminUser | null;
  loading: boolean;
  isInitialized: boolean;
  isAdmin: boolean;
  error: string | null;
  login: (token: string) => Promise<void>;
  logout: () => Promise<void>;
  signOut: () => Promise<void>;
  signIn: (email: string, password: string) => Promise<void>;
}

const AdminAuthContext = createContext<AdminAuthCtx | null>(null);

export function AdminAuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AdminUser | null>(null);
  const [loading, setLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const login = async (token: string) => {
    try {
      setError(null);
      const response = await apiClient.serverLogin({ token });
      if (isApiError(response)) {
        setError(response.error || 'Login failed');
        return;
      }
      const userData = response.data;
      setUser({
        uid: userData.user_id,
        email: userData.email,
        displayName: userData.displayName,
        roles: [userData.role], // Convert single role to array
        isAdmin: userData.role === 'admin' || userData.role === 'ADMIN',
        emailVerified: true, // Assume verified if profile is accessible
        disabled: false,
        customClaims: { role: userData.role },
        metadata: undefined
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
      const response = await apiClient.login({ type: 'credentials', email, password });
      if (isApiError(response)) {
        const errorMessage = response.error || 'Invalid credentials';
        setError(errorMessage);
        throw new Error(errorMessage);
      }
      const userData = response.data;
      setUser({
        uid: userData.user_id,
        email: userData.email,
        displayName: userData.displayName,
        roles: [userData.role], // Convert single role to array
        isAdmin: userData.role === 'admin' || userData.role === 'ADMIN',
        emailVerified: true, // Assume verified if profile is accessible
        disabled: false,
        customClaims: { role: userData.role },
        metadata: undefined
      });
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
    const checkAuth = async () => {
      try {
        const response = await apiClient.getCurrentUser();
        if (isApiError(response)) {
          adminLogger.error('Admin auth check failed', { error: response.error }, 'AdminAuthProvider.checkAuth');
        } else {
          const userData = response.data;
          setUser({
            uid: userData.user_id,
            email: userData.email,
            displayName: userData.displayName,
            roles: [userData.role], // Convert single role to array
            isAdmin: userData.role === 'admin' || userData.role === 'ADMIN',
            emailVerified: true, // Assume verified if profile is accessible
            disabled: false,
            customClaims: { role: userData.role },
            metadata: undefined
          });
        }
      } catch (error) {
        adminLogger.error('Admin auth check failed', { error: error instanceof Error ? error.message : error }, 'AdminAuthProvider.checkAuth');
      } finally {
        setLoading(false);
        setIsInitialized(true);
      }
    };
    
    checkAuth();
  }, []);

  const isAdmin = user?.isAdmin || user?.customClaims?.role === 'ADMIN' || false;

  return (
    <AdminAuthContext.Provider value={{ 
      user, 
      loading, 
      isInitialized, 
      isAdmin, 
      error,
      login, 
      logout, 
      signOut,
      signIn
    }}>
      {children}
    </AdminAuthContext.Provider>
  );
}

export const useAdminAuth = () => {
  const ctx = useContext(AdminAuthContext);
  if (!ctx) throw new Error('useAdminAuth must be used within AdminAuthProvider');
  return ctx;
};

// Export the provider with the expected name
export const AppAdminAuthProvider = AdminAuthProvider;