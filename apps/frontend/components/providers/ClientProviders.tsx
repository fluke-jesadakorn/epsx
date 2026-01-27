'use client';

import '@/lib/polyfills';

import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { Toaster } from '@/components/ui/toaster';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';
import { UnifiedWeb3Provider } from '@/shared/components/providers/UnifiedWeb3Provider';
import React from 'react';
import { State } from 'wagmi';

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