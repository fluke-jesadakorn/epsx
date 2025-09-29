'use client';

import { ReactNode } from 'react';
import { ThemeProvider } from 'next-themes';
import { Web3Provider } from '../../providers/Web3Provider';
import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/SharedOpenIDWeb3Provider';

interface ClientProvidersProps {
  children: ReactNode;
}

/**
 * Client providers with Web3 and auth support
 * Moved to client component to prevent server-side hydration issues
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