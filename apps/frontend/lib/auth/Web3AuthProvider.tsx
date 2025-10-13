'use client';

import React from 'react';
import { SharedOpenIDWeb3Provider, useSharedAuth } from '@/shared/components/auth/Provider';
import { createFrontendClient } from '@/shared/auth/client';

// Create frontend client instance
const frontendClient = createFrontendClient();

// Web3 Auth Provider Component
// Now uses the shared OpenID + Web3 authentication system
export function Web3AuthProvider({ children }: { children: React.ReactNode }) {
  return (
    <SharedOpenIDWeb3Provider 
      clientId="epsx-frontend"
      onAuthError={(error) => {
        console.error('Frontend auth error:', error);
      }}
    >
      {children}
    </SharedOpenIDWeb3Provider>
  );
}

// Direct re-exports - no legacy wrapper needed
export { useSharedAuth as useWeb3Auth, useSharedAuth as useAuth } from '@/shared/components/auth/Provider';

// Simplified hooks using shared auth directly
export function useWeb3Permission(permission: string): boolean {
  const { hasPermissionForDisplay } = useSharedAuth();
  return hasPermissionForDisplay(permission);
}

export function useWeb3Admin(): boolean {
  const { hasPermissionForDisplay } = useSharedAuth();
  return hasPermissionForDisplay('admin:*:*');
}