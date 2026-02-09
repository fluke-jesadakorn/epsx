'use client';

import { createFrontendClient } from '@/shared/auth/client';
import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/Provider';
import React from 'react';

// Create frontend client instance
const frontendClient = createFrontendClient();

// Web3 Auth Provider Component
// Now uses the shared OpenID + Web3 authentication system
export function Web3AuthProvider({ children }: { children: React.ReactNode }) {
  return (
    <SharedOpenIDWeb3Provider
      clientId="epsx-frontend"
      onAuthError={(error) => {
      }}
    >
      {children}
    </SharedOpenIDWeb3Provider>
  );
}

// Direct re-exports - no legacy wrapper needed
export { useSharedAuth as useAuth, useSharedAuth as useWeb3Auth } from '@/shared/components/auth/Provider';

// Simplified hooks using shared auth directly
export function useWeb3Permission(_permission: string): boolean {
  // PERMISSION REFACTOR: All client-side checks are now permissive.
  return true;
}

export function useWeb3Admin(): boolean {
  // PERMISSION REFACTOR: All client-side checks are now permissive.
  return true;
}