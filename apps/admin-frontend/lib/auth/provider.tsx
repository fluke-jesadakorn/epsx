/**
 * Authentication Provider
 * Manages authentication state across the application
 */
'use client';

import { createContext, useCallback, useEffect, useState, ReactNode } from 'react';

// Session data structure (matches server-side SessionData)
export interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  firebase_uid?: string;
  admin_modules: string[];
  permissions: string[];
  package_tier: string;
  hasAdminModule: (module: string) => boolean;
  isSystemAdmin: () => boolean;
}

export interface Session {
  user?: User;
  isLoggedIn: boolean;
  expiresAt?: number;
}

export interface AuthContextType {
  session: Session | null;
  isLoading: boolean;
  refreshSession: () => Promise<void>;
}

// Create authentication context
export const AuthContext = createContext<AuthContextType | null>(null);

interface AuthProviderProps {
  children: ReactNode;
}

/**
 * AuthProvider component - replaces SessionProvider from Auth.js
 */
export function AuthProvider({ children }: AuthProviderProps) {
  const [session, setSession] = useState<Session | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Fetch session from API
  const fetchSession = useCallback(async (): Promise<Session> => {
    try {
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
      });
      
      if (!response.ok) {
        throw new Error('Session fetch failed');
      }
      
      const sessionData = await response.json();
      
      // Add convenience methods to user object if present
      if (sessionData.user) {
        sessionData.user.hasAdminModule = (module: string) => 
          sessionData.user?.admin_modules?.includes(module) || false;
        sessionData.user.isSystemAdmin = () => 
          sessionData.user?.admin_modules?.includes('system_admin') || false;
      }
      
      return sessionData;
    } catch (error) {
      console.error('Session fetch error:', error);
      return { isLoggedIn: false };
    }
  }, []);

  // Refresh session data
  const refreshSession = useCallback(async () => {
    setIsLoading(true);
    try {
      const newSession = await fetchSession();
      setSession(newSession);
      console.log('🔄 Session refreshed:', {
        isLoggedIn: newSession.isLoggedIn,
        user: newSession.user?.email,
      });
    } catch (error) {
      console.error('Session refresh error:', error);
      setSession({ isLoggedIn: false });
    } finally {
      setIsLoading(false);
    }
  }, [fetchSession]);

  // Initialize session on mount
  useEffect(() => {
    refreshSession();
  }, [refreshSession]);

  // Auto-refresh session periodically (every 5 minutes)
  useEffect(() => {
    if (!session?.isLoggedIn) return;

    const interval = setInterval(() => {
      refreshSession();
    }, 5 * 60 * 1000); // 5 minutes

    return () => clearInterval(interval);
  }, [session?.isLoggedIn, refreshSession]);

  // Auto-refresh when tab becomes visible
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible' && session?.isLoggedIn) {
        refreshSession();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, [session?.isLoggedIn, refreshSession]);

  const contextValue: AuthContextType = {
    session,
    isLoading,
    refreshSession,
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
}