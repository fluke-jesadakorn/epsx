'use client';

import { ThemeProvider } from 'next-themes';
import { ReactNode } from 'react';

import { Web3Provider } from '../../providers/AuthProvider';

import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/Provider';

interface ClientProvidersProps {
  children: ReactNode;
}

/**
 * Client providers with Web3 and auth support
 * Moved to client component to prevent server-side hydration issues
 * @param root0
 * @param root0.children
 */
export function ClientProviders({ children }: ClientProvidersProps) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
    >
      <Web3Provider>
        <SharedOpenIDWeb3Provider 
          clientId="epsx-admin"
          backendUrl={process.env.NEXT_PUBLIC_BACKEND_URL}
        >
          {children}
        </SharedOpenIDWeb3Provider>
      </Web3Provider>
    </ThemeProvider>
  );
}