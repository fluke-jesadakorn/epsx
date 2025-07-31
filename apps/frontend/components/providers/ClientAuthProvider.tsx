'use client';

import { UnifiedAuthProvider } from '@epsx/auth-shared/client';
import { ReactNode } from 'react';

interface ClientAuthProviderProps {
  children: ReactNode;
  isAdminContext?: boolean;
}

export function ClientAuthProvider({ 
  children, 
  isAdminContext = false 
}: ClientAuthProviderProps) {
  return (
    <UnifiedAuthProvider isAdminContext={isAdminContext}>
      {children}
    </UnifiedAuthProvider>
  );
}