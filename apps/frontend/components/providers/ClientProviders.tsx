'use client';

import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { Toaster } from '@/components/ui/toaster';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';
import { UnifiedWeb3Provider } from '@/shared/components/providers/UnifiedWeb3Provider';
import React from 'react';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <CommonProviders>
      <UnifiedWeb3Provider>
        <PerformanceProvider>
          <OptimizedSuspenseBoundary identifier="main content">
            {children}
          </OptimizedSuspenseBoundary>
          <Toaster />
        </PerformanceProvider>
      </UnifiedWeb3Provider>
    </CommonProviders>
  );
}