'use client';

import React, { createContext, useContext, useEffect, useState } from 'react';
import { PackageTier } from '@epsx/types';

interface BackendUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
  // Firebase compatibility properties
  uid?: string; // Alias for user_id
  emailVerified?: boolean;
  displayName?: string;
  photoURL?: string;
  providerData?: any[];
  isAnonymous?: boolean;
  metadata?: any;
  refreshToken?: string;
  tenantId?: string | null;
  // Additional Firebase User properties for compatibility
  delete?: () => Promise<void>;
  getIdToken?: (forceRefresh?: boolean) => Promise<string>;
  getIdTokenResult?: (forceRefresh?: boolean) => Promise<any>;
  reload?: () => Promise<void>;
  toJSON?: () => object;
}

interface AuthContextType {
  user: BackendUser | null;
  loading: boolean;
  isInitialized: boolean;
  permissions: string[];
  packageTier: PackageTier;
  hasPermission: (permission: string) => boolean;
  refreshPermissions: () => Promise<void>;
  login: (email: string, password: string) => Promise<void>;
  loginWithFirebaseToken: (token: string) => Promise<void>;
  register: (email: string, password: string, displayName?: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<BackendUser | null>(null);
  const [loading, setLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [permissions, setPermissions] = useState<string[]>([]);
  const [packageTier, setPackageTier] = useState<PackageTier>(PackageTier.FREE);

  const loadUserSession = async () => {
    try {
      const response = await fetch('/api/auth/me', {
        method: 'GET',
        credentials: 'include',
      });

      if (!response.ok) {
        if (response.status === 401) {
          // Session expired or invalid
          setUser(null);
          setPermissions([]);
          setPackageTier(PackageTier.FREE);
          return;
        }
        throw new Error('Failed to load session');
      }

      const userData = await response.json();
      // Add Firebase compatibility properties
      const enhancedUserData = {
        ...userData,
        uid: userData.user_id, // Alias for Firebase compatibility
        emailVerified: true, // Backend users are considered verified
        displayName: userData.display_name || userData.email?.split('@')[0] || null,
        photoURL: userData.photo_url || null,
        providerData: userData.provider_data || [],
      };
      setUser(enhancedUserData);
      setPermissions(userData.permissions || []);
      setPackageTier(userData.package_tier as PackageTier || PackageTier.FREE);
    } catch (error) {
      console.error('Error loading user session:', error);
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
    }
  };

  const refreshPermissions = async () => {
    await loadUserSession();
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
    const initAuth = async () => {
      try {
        await loadUserSession();
      } finally {
        setLoading(false);
        setIsInitialized(true);
      }
    };

    initAuth();
  }, []);

  const login = async (email: string, password: string) => {
    try {
      const response = await fetch('/api/auth/enhanced-login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ 
          type: 'credentials', 
          email, 
          password 
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Login failed');
      }

      const userData = await response.json();
      // Add Firebase compatibility properties
      const enhancedUserData = {
        ...userData,
        uid: userData.user_id, // Alias for Firebase compatibility
        emailVerified: true, // Backend users are considered verified
        displayName: userData.display_name || userData.email?.split('@')[0] || null,
        photoURL: userData.photo_url || null,
        providerData: userData.provider_data || [],
      };
      setUser(enhancedUserData);
      setPermissions(userData.permissions || []);
      setPackageTier(userData.package_tier as PackageTier || PackageTier.FREE);
      
      return enhancedUserData;
    } catch (error) {
      throw error;
    }
  };

  const loginWithFirebaseToken = async (token: string) => {
    try {
      const response = await fetch('/api/auth/enhanced-login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ 
          type: 'firebase', 
          firebase_token: token 
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Login failed');
      }

      const userData = await response.json();
      // Add Firebase compatibility properties
      const enhancedUserData = {
        ...userData,
        uid: userData.user_id, // Alias for Firebase compatibility
        emailVerified: true, // Backend users are considered verified
        displayName: userData.display_name || userData.email?.split('@')[0] || null,
        photoURL: userData.photo_url || null,
        providerData: userData.provider_data || [],
      };
      setUser(enhancedUserData);
      setPermissions(userData.permissions || []);
      setPackageTier(userData.package_tier as PackageTier || PackageTier.FREE);
      
      return enhancedUserData;
    } catch (error) {
      throw error;
    }
  };

  const register = async (email: string, password: string, displayName?: string) => {
    try {
      const response = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ 
          email, 
          password, 
          name: displayName,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Registration failed');
      }

      const data = await response.json();
      return data;
    } catch (error) {
      throw error;
    }
  };

  const logout = async () => {
    try {
      await fetch('/api/auth/logout', { 
        method: 'POST',
        credentials: 'include',
      });
      setUser(null);
      setPermissions([]);
      setPackageTier(PackageTier.FREE);
    } catch (error) {
      console.error('Logout error:', error);
    }
  };

  return (
    <AuthContext.Provider value={{ 
      user, 
      loading, 
      isInitialized,
      permissions, 
      packageTier,
      hasPermission,
      refreshPermissions,
      login,
      loginWithFirebaseToken,
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
