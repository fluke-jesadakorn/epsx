'use client';
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';

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
      const res = await fetch('/api/v1/authentication/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token }),
      });
      
      if (res.ok) {
        const data = await res.json();
        setUsr(data.user);
      }
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  const logout = async () => {
    try {
      await fetch('/api/v1/authentication/logout', { method: 'POST' });
      setUsr(null);
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const res = await fetch('/api/v1/authentication/profile');
        if (res.ok) {
          const data = await res.json();
          setUsr(data.user);
        }
      } catch (error) {
        console.error('Auth check failed:', error);
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