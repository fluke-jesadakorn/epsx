'use client';

import { PerformanceProvider } from '@/components/common/PerformanceProvider';
// import { BackgroundDecorationsClient } from '../layout/BackgroundDecorations.client';
import { ToastProvider } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { UIProvider } from '@/context/ui-context';
import { ThemeProvider } from 'next-themes';
import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { RemoteConfigProvider } from '@/providers/RemoteConfigProvider';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider defaultTheme="system" enableSystem>
      <RemoteConfigProvider 
        autoRefreshInterval={300000} // 5 minutes
        fetchOnMount={true}
      >
        <PerformanceProvider>
          <AppStateProvider>
            <UIProvider>
              <ToastProvider>
                {/* <BackgroundDecorationsClient /> */}

                <OptimizedSuspenseBoundary identifier="main content">
                  {children}
                </OptimizedSuspenseBoundary>
              </ToastProvider>
            </UIProvider>
          </AppStateProvider>
        </PerformanceProvider>
      </RemoteConfigProvider>
    </ThemeProvider>
  );
}