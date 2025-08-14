/**
 * Minimal Frontend Authentication Hooks
 */
'use client';

import { useState } from 'react';

export interface FrontendUser {
  id: string;
  email: string;
  name: string;
  role: string;
  permissions: string[];
  package_tier: string;
  firebase_uid: string;
  hasPermission: (permission: string) => boolean;
  hasPackageTier: (tier: string) => boolean;
}

export interface AuthState {
  user: FrontendUser | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  accessToken: string | null;
}

// Simple exports - no complex JSX components for now
export function AuthProvider({ children }: { children: any }) {
  return children;
}

export function useAuth() {
  const [state] = useState<AuthState>({
    user: null,
    isLoading: false,
    isAuthenticated: false,
    accessToken: null,
  });

  return {
    ...state,
    signIn: () => { if (typeof window !== 'undefined') window.location.href = '/api/auth/signin'; },
    signOut: async () => {
      if (typeof window !== 'undefined') {
        await fetch('/api/auth/signout', { method: 'POST' });
        window.location.href = '/login';
      }
    },
    refreshSession: () => {},
  };
}

export function useSignIn() {
  return { signIn: () => { if (typeof window !== 'undefined') window.location.href = '/api/auth/signin'; } };
}

export function useSignOut() {
  return { signOut: async () => {
    if (typeof window !== 'undefined') {
      await fetch('/api/auth/signout', { method: 'POST' });
      window.location.href = '/login';
    }
  }};
}

export function useUser() {
  return { user: null, isLoading: false };
}

export function useSession() {
  return {
    data: null,
    status: 'unauthenticated' as const,
    update: () => {},
    accessToken: null,
  };
}

export function usePermissions() {
  return {
    permissions: [],
    hasPermission: () => false,
    hasAnyPermission: () => false,
    hasAllPermissions: () => false,
  };
}

export function usePackageTier() {
  return {
    package_tier: 'FREE' as const,
    hasPackageTier: () => false,
    hasMinimumTier: () => false,
  };
}