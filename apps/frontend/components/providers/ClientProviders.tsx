'use client';

import '@/lib/polyfills';

import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { Toaster } from '@/components/ui/toaster';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';
import dynamic from 'next/dynamic';
import React from 'react';

// Dynamic import with ssr: false to prevent WalletConnect localStorage errors during SSR
// The @walletconnect/keyvaluestorage package tries to access localStorage during module init
const UnifiedWeb3Provider = dynamic(
  () => import('@/shared/components/providers/UnifiedWeb3Provider').then(mod => mod.UnifiedWeb3Provider),
  { ssr: false }
);

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