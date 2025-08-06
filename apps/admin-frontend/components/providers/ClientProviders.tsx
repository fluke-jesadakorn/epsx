'use client';

import { ToastProvider } from '@/components/ui/toast';
import { ThemeTransition } from '@/components/ui/theme-transition';
import { Navigation } from '@/components/layout/nav';
import { SessionProvider } from 'next-auth/react';
import { AdminAuthProvider } from '@/auth/ctx';
import { _ModuleAuthProvider } from '@/auth/module-ctx';
import { GlobalThemeProvider } from '@epsx/theme';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <GlobalThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <ThemeTransition />
      <SessionProvider>
        <AdminAuthProvider>
          {/* <ModuleAuthProvider> */}
            <ToastProvider>
              <Navigation />
              <div className="relative flex min-h-screen flex-col card">
                {children}
              </div>
            </ToastProvider>
          {/* </ModuleAuthProvider> */}
        </AdminAuthProvider>
      </SessionProvider>
    </GlobalThemeProvider>
  );
}