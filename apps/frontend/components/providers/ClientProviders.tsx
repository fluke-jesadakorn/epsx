'use client';

import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { BackgroundDecorationsClient } from '@/components/layout/BackgroundDecorations.client';
import { Navigation } from '@/components/nav';
import { ToastProvider } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { AuthProvider } from '@/context/auth-context';
import { SessionProvider } from 'next-auth/react';
import { PermissionProvider } from '@epsx/server-providers/client';
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
              <AuthProvider>
                <PermissionProvider serverData={{
                  paymentStatus: null,
                  permissions: [],
                  featureAccess: {},
                  rankingAccess: {},
                  error: null
                }}>
                  <ToastProvider>
                    <BackgroundDecorationsClient />

                    <OptimizedSuspenseBoundary identifier="navigation">
                      <Navigation />
                    </OptimizedSuspenseBoundary>

                    <OptimizedSuspenseBoundary identifier="main content">
                      {children}
                    </OptimizedSuspenseBoundary>
                  </ToastProvider>
                </PermissionProvider>
              </AuthProvider>
            </SessionProvider>
          </UIProvider>
        </AppStateProvider>
      </PerformanceProvider>
    </GlobalThemeProvider>
  );
}