'use client';

import React, { createContext, useContext, useEffect, useState } from 'react';
import { iamService, type User, type UserRole, type UserPermission } from '@/services/iamService';

interface IAMContextType {
  user: User | null;
  role: UserRole | null;
  permissions: UserPermission[];
  loading: boolean;
  error: string | null;
  signIn: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  signUp: (email: string, password: string, displayName: string) => Promise<void>;
  hasPermission: (permissionId: string) => boolean;
  hasAnyPermission: (permissionIds: string[]) => boolean;
  hasAllPermissions: (permissionIds: string[]) => boolean;
  resetPassword: (email: string) => Promise<void>;
}

const IAMContext = createContext<IAMContextType | undefined>(undefined);

export function IAMProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [role, setRole] = useState<UserRole | null>(null);
  const [permissions, setPermissions] = useState<UserPermission[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Initialize IAM system
    iamService.initializeIAM().catch(console.error);

    // Set up auth state listener
    const unsubscribe = iamService.onAuthStateChange((user) => {
      setUser(user);
      if (user) {
        setRole(iamService.getUserRole());
        setPermissions(iamService.getUserPermissions());
      } else {
        setRole(null);
        setPermissions([]);
      }
      setLoading(false);
    });

    return () => unsubscribe();
  }, []);

  const signIn = async (email: string, password: string) => {
    try {
      setError(null);
      await iamService.signIn(email, password);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign in failed');
      throw err;
    }
  };

  const signOut = async () => {
    try {
      setError(null);
      await iamService.signOut();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign out failed');
      throw err;
    }
  };

  const signUp = async (email: string, password: string, displayName: string) => {
    try {
      setError(null);
      await iamService.signUp(email, password, displayName);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign up failed');
      throw err;
    }
  };

  const resetPassword = async (email: string) => {
    try {
      setError(null);
      await iamService.resetPassword(email);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Password reset failed');
      throw err;
    }
  };

  const hasPermission = (permissionId: string): boolean => {
    return iamService.hasPermission(permissionId);
  };

  const hasAnyPermission = (permissionIds: string[]): boolean => {
    return iamService.hasAnyPermission(permissionIds);
  };

  const hasAllPermissions = (permissionIds: string[]): boolean => {
    return iamService.hasAllPermissions(permissionIds);
  };

  return (
    <IAMContext.Provider
      value={{
        user,
        role,
        permissions,
        loading,
        error,
        signIn,
        signOut,
        signUp,
        hasPermission,
        hasAnyPermission,
        hasAllPermissions,
        resetPassword,
      }}
    >
      {children}
    </IAMContext.Provider>
  );
}

export function useIAM() {
  const context = useContext(IAMContext);
  if (context === undefined) {
    throw new Error('useIAM must be used within an IAMProvider');
  }
  return context;
}
