'use client';

import { useEffect, useState } from 'react';
import { AuthProvider } from '@/context/auth-context';
import { useAppState } from '@/context/app-state';

interface ClientAuthProviderProps {
  children: React.ReactNode;
  initialAuthState?: any;
}

/**
 * Client-side auth provider optimized for SSR hydration
 */
export function ClientAuthProvider({ children, initialAuthState }: ClientAuthProviderProps) {
  const [_isHydrated, setIsHydrated] = useState(false);
  const { actions } = useAppState();

  useEffect(() => {
    // Mark as hydrated and initialize from server state
    setIsHydrated(true);
    
    // Pre-populate app state if we have initial auth state
    if (initialAuthState?.user) {
      actions.user.setProfile(initialAuthState.user);
      actions.user.updatePermissions(initialAuthState.permissions || []);
      actions.user.setPackageTier(initialAuthState.packageTier || 'FREE');
    }
  }, [initialAuthState, actions]);

  // Prevent hydration issues by rendering with server state initially
  return (
    <AuthProvider 
      initialAuthState={initialAuthState}
    >
      {children}
    </AuthProvider>
  );
}