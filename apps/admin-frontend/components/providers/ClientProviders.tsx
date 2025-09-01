'use client';

import { ReactNode } from 'react';
import { ThemeProvider } from 'next-themes';

interface ClientProvidersProps {
  children: ReactNode;
}

/**
 * Client providers with theme support
 * Auth state is managed by Zustand store, direct next-themes integration
 */
export function ClientProviders({ children }: ClientProvidersProps) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
    >
      {children}
    </ThemeProvider>
  );
}