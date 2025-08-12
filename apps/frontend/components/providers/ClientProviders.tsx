'use client';

import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { BackgroundDecorationsClient } from '@/components/layout/BackgroundDecorations.client';
import { ToastProvider } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { UIProvider } from '@/context/ui-context';
import { GlobalThemeProvider } from '@epsx/theme';
import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <GlobalThemeProvider defaultTheme="system" enableSystem>
      <PerformanceProvider>
        <AppStateProvider>
          <UIProvider>
            <ToastProvider>
              <BackgroundDecorationsClient />

              <OptimizedSuspenseBoundary identifier="main content">
                {children}
              </OptimizedSuspenseBoundary>
            </ToastProvider>
          </UIProvider>
        </AppStateProvider>
      </PerformanceProvider>
    </GlobalThemeProvider>
  );
}