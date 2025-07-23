'use client';
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';

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
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token }),
      });
      
      if (res.ok) {
        const data = await res.json();
        setUser(data.user);
      } else {
        const errorData = await res.json();
        setError(errorData.message || 'Login failed');
      }
    } catch (error) {
      console.error('Admin login failed:', error);
      setError('Network error occurred');
    }
  };

  const signIn = async (email: string, password: string) => {
    try {
      setError(null);
      setLoading(true);
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });
      
      if (res.ok) {
        const data = await res.json();
        setUser(data.user);
      } else {
        const errorData = await res.json();
        setError(errorData.message || 'Invalid credentials');
        throw new Error(errorData.message || 'Invalid credentials');
      }
    } catch (error) {
      console.error('Admin sign in failed:', error);
      const errorMessage = error instanceof Error ? error.message : 'Network error occurred';
      setError(errorMessage);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const logout = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      setUser(null);
      setError(null);
    } catch (error) {
      console.error('Admin logout failed:', error);
    }
  };

  const signOut = logout; // Alias for logout

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const res = await fetch('/api/auth/me');
        if (res.ok) {
          const data = await res.json();
          setUser(data.user);
        }
      } catch (error) {
        console.error('Admin auth check failed:', error);
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