'use client';

import { ToastProvider } from '@/components/ui/toast';
import { ThemeTransition } from '@/components/ui/theme-transition';
import { SessionProvider } from 'next-auth/react';
import { AdminAuthProvider } from '@/lib/auth/ctx';
import { ModuleAuthProvider } from '@/auth/module-ctx';
import { GlobalThemeProvider } from '@epsx/theme';
import { AdminAuthWrapper } from './AdminAuthWrapper';
import { ErrorBoundary } from '@epsx/ui';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <GlobalThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <ErrorBoundary context="admin" enableRetry maxRetries={2} title="Admin Panel Error">
        <ThemeTransition />
        <SessionProvider>
          <AdminAuthProvider>
            <ModuleAuthProvider>
              <ToastProvider>
                <AdminAuthWrapper>
                  {children}
                </AdminAuthWrapper>
              </ToastProvider>
            </ModuleAuthProvider>
          </AdminAuthProvider>
        </SessionProvider>
      </ErrorBoundary>
    </GlobalThemeProvider>
  );
}