'use client';

import '@/lib/polyfills';

import { PerformanceProvider } from '@/components/common/performance-provider';
import { Toaster } from '@/components/ui/toaster';
import { CommonProviders } from '@/shared/components/providers/common-providers';
import { SessionExpiryProvider } from '@/shared/components/providers/session-expiry-provider';
import { UnifiedWeb3Provider } from '@/shared/components/providers/web3-provider';
import React from 'react';
import type { State } from 'wagmi';

export function ClientProviders({ children, initialState }: { children: React.ReactNode; initialState?: State }) {
  return (
    <CommonProviders>
      <SessionExpiryProvider>
        <UnifiedWeb3Provider initialState={initialState}>
          <PerformanceProvider>
            {children}
            <Toaster />
          </PerformanceProvider>
        </UnifiedWeb3Provider>
      </SessionExpiryProvider>
    </CommonProviders>
  );
}