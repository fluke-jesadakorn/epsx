'use client';

import { ToastProvider } from '@/components/ui/toast';
import { ThemeTransition } from '@/components/ui/theme-transition';
import { SessionProvider } from 'next-auth/react';
import { ServerSideAuthProvider } from '@/context/server-side-auth';
import { GlobalThemeProvider } from '@epsx/theme';
import { AdminAuthWrapper } from './AdminAuthWrapper';
import { ErrorBoundary } from '@epsx/ui';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <GlobalThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <ErrorBoundary context="admin" enableRetry maxRetries={2} title="Admin Panel Error">
        <ThemeTransition />
        <SessionProvider>
          <ServerSideAuthProvider>
            <ToastProvider>
              <AdminAuthWrapper>
                {children}
              </AdminAuthWrapper>
            </ToastProvider>
          </ServerSideAuthProvider>
        </SessionProvider>
      </ErrorBoundary>
    </GlobalThemeProvider>
  );
}