'use client';
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { logger } from '@/lib/logger';
import { apiClient, isApiError } from '@epsx/api-client';

interface Usr {
  id: string;
  email: string;
  roles: string[];
}

interface AuthCtx {
  usr: Usr | null;
  loading: boolean;
  login: (token: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthCtx | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [usr, setUsr] = useState<Usr | null>(null);
  const [loading, setLoading] = useState(true);

  const login = async (token: string) => {
    try {
      const response = await apiClient.post('/api/v1/authentication/login', { token });
      if (isApiError(response)) {
        logger.error('Login failed', { error: response.error });
        return;
      }
      const userData = response.data;
      setUsr({
        id: userData.user_id,
        email: userData.email,
        roles: [userData.role], // Convert single role to array
      });
    } catch (error) {
      logger.error('Login failed', { error: error instanceof Error ? error.message : error });
    }
  };

  const logout = async () => {
    try {
      const response = await apiClient.logout();
      if (isApiError(response)) {
        logger.error('Logout failed', { error: response.error });
      }
      setUsr(null);
    } catch (error) {
      logger.error('Logout failed', { error: error instanceof Error ? error.message : error });
    }
  };

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const response = await apiClient.getCurrentUser();
        if (isApiError(response)) {
          logger.error('Auth check failed', { error: response.error });
        } else {
          const userData = response.data;
          setUsr({
            id: userData.user_id,
            email: userData.email,
            roles: [userData.role], // Convert single role to array
          });
        }
      } catch (error) {
        logger.error('Auth check failed', { error: error instanceof Error ? error.message : error });
      } finally {
        setLoading(false);
      }
    };
    
    checkAuth();
  }, []);

  return (
    <AuthContext.Provider value={{ usr, loading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
};