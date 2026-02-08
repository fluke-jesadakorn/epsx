'use client';

import '@/lib/polyfills';

import { OptimizedSuspenseBoundary } from '@/components/common/optimized-suspense-boundary';
import { PerformanceProvider } from '@/components/common/performance-provider';
import { Toaster } from '@/components/ui/toaster';
import { CommonProviders } from '@/shared/components/providers/common-providers';
import { UnifiedWeb3Provider } from '@/shared/components/providers/unified-web3-provider';
import React from 'react';
import type { State } from 'wagmi';

export function ClientProviders({ children, initialState }: { children: React.ReactNode; initialState?: State }) {
  return (
    <CommonProviders>
      <UnifiedWeb3Provider initialState={initialState}>
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