'use client';

import { ReactNode } from 'react';
import { AuthProvider } from '@/lib/auth';

interface ClientProvidersProps {
  children: ReactNode;
  // session prop no longer needed since we manage state internally
}

export function ClientProviders({ children }: ClientProvidersProps) {
  return (
    <AuthProvider>
      {children}
    </AuthProvider>
  );
}