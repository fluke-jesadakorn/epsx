'use client';

import React, { createContext, useContext, useEffect, useState } from 'react';
import { onAuthStateChanged } from 'firebase/auth';
import type { User } from 'firebase/auth';
import { auth } from '@/lib/firebase-iam';
import { getUserPermissions } from '@/lib/firebase-iam-helpers';
import { templateEvaluationService, EffectivePermissions } from '@/lib/template-evaluation';
import { PackageTier } from '@epsx/types';
import { doc, getDoc } from 'firebase/firestore';
import { db } from '@/lib/firebase-iam';

interface AuthContextType {
  user: User | null;
  loading: boolean;
  permissions: string[];
  effectivePermissions: EffectivePermissions | null;
  packageTier: PackageTier;
  hasPermission: (permission: string) => boolean;
  refreshPermissions: () => Promise<void>;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, displayName?: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [permissions, setPermissions] = useState<string[]>([]);
  const [effectivePermissions, setEffectivePermissions] = useState<EffectivePermissions | null>(null);
  const [packageTier, setPackageTier] = useState<PackageTier>(PackageTier.FREE);

  const loadUserPermissions = async (user: User) => {
    try {
      if (!db) {
        console.warn('Firebase not initialized');
        setPermissions([]);
        setEffectivePermissions(null);
        return;
      }

      // Get user data
      const userDoc = await getDoc(doc(db, 'users', user.uid));
      if (!userDoc.exists()) {
        console.warn('User document not found');
        setPermissions([]);
        setEffectivePermissions(null);
        return;
      }

      const userData = userDoc.data();
      const userPackageTier = userData.packageTier || PackageTier.FREE;
      setPackageTier(userPackageTier);

      // Build user context for template evaluation
      const context = {
        userId: user.uid,
        packageTier: userPackageTier,
        staticPermissions: userData.permissions || [],
        roles: userData.roles || [],
      };

      // Evaluate templates and get effective permissions
      const effectivePerms = await templateEvaluationService.evaluateUserPermissions(context);
      
      setPermissions(effectivePerms.permissions);
      setEffectivePermissions(effectivePerms);

      // Log template sources for debugging
      if (effectivePerms.templateSources.length > 0) {
        console.log('Active templates contributing permissions:', effectivePerms.templateSources);
      }
      
      if (effectivePerms.conflicts.length > 0) {
        console.warn('Permission conflicts detected:', effectivePerms.conflicts);
      }
    } catch (error) {
      console.error('Error loading user permissions:', error);
      setPermissions([]);
      setEffectivePermissions(null);
    }
  };

  const refreshPermissions = async () => {
    if (user) {
      await loadUserPermissions(user);
    }
  };

  const hasPermission = (permission: string): boolean => {
    // Check exact match
    if (permissions.includes(permission)) {
      return true;
    }

    // Check wildcard permissions (e.g., "admin.*" covers "admin.users.create")
    return permissions.some(userPermission => {
      if (userPermission.endsWith('.*')) {
        const prefix = userPermission.slice(0, -2);
        return permission.startsWith(prefix + '.');
      }
      return false;
    });
  };

  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      setUser(user);
      if (user) {
        await loadUserPermissions(user);
      } else {
        setPermissions([]);
        setEffectivePermissions(null);
        setPackageTier(PackageTier.FREE);
      }
      setLoading(false);
    });

    return unsubscribe;
  }, []);

  const login = async (email: string, password: string) => {
    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Login failed');
      }

      const data = await response.json();
      return data;
    } catch (error) {
      throw error;
    }
  };

  const register = async (email: string, password: string, displayName?: string) => {
    try {
      const response = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, displayName }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Registration failed');
      }

      const data = await response.json();
      return data;
    } catch (error) {
      throw error;
    }
  };

  const logout = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      setUser(null);
      setPermissions([]);
    } catch (error) {
      console.error('Logout error:', error);
    }
  };

  return (
    <AuthContext.Provider value={{ 
      user, 
      loading, 
      permissions, 
      effectivePermissions,
      packageTier,
      hasPermission,
      refreshPermissions,
      login, 
      register, 
      logout 
    }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
