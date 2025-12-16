'use client';

import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { Toaster } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { UIProvider } from '@/context/ui-context';
import { ThemeProvider } from 'next-themes';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <PerformanceProvider>
        <AppStateProvider>
          <UIProvider>
            {/* <BackgroundDecorationsClient /> */}
            <OptimizedSuspenseBoundary identifier="main content">
              {children}
            </OptimizedSuspenseBoundary>
            <Toaster />
          </UIProvider>
        </AppStateProvider>
      </PerformanceProvider>
    </ThemeProvider>
  );
}