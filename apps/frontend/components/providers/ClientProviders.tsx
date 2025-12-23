'use client';

import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { Toaster } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { UIProvider } from '@/context/ui-context';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';
import { UnifiedWeb3Provider } from '@/shared/components/providers/UnifiedWeb3Provider';
import React from 'react';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <CommonProviders>
      <UnifiedWeb3Provider>
        <PerformanceProvider>
          <AppStateProvider>
            <UIProvider>
              <OptimizedSuspenseBoundary identifier="main content">
                {children}
              </OptimizedSuspenseBoundary>
              <Toaster />
            </UIProvider>
          </AppStateProvider>
        </PerformanceProvider>
      </UnifiedWeb3Provider>
    </CommonProviders>
  );
}