'use client';

import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { BackgroundDecorationsClient } from '@/components/layout/BackgroundDecorations.client';
import { ToastProvider } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { SessionProvider } from 'next-auth/react';
import { UIProvider } from '@/context/ui-context';
import { GlobalThemeProvider } from '@epsx/theme';
import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <GlobalThemeProvider defaultTheme="system" enableSystem>
      <PerformanceProvider>
        <AppStateProvider>
          <UIProvider>
            <SessionProvider>
              <ToastProvider>
                <BackgroundDecorationsClient />

                <OptimizedSuspenseBoundary identifier="main content">
                  {children}
                </OptimizedSuspenseBoundary>
              </ToastProvider>
            </SessionProvider>
          </UIProvider>
        </AppStateProvider>
      </PerformanceProvider>
    </GlobalThemeProvider>
  );
}