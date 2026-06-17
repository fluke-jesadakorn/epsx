'use client';
import '@/lib/polyfills';

import { bsc, bscTestnet, foundry } from '@/config/wagmi-chains';
import React, { useState } from 'react';
import { QueryClientProvider } from '@tanstack/react-query';

import { SettingsProvider } from './settings-provider';

import { SharedOpenIDWeb3Provider } from '@/shared/components/auth';
import type { UserInfoResponse } from '@/shared/auth/client';
import { CommonProviders } from '@/shared/components/providers/common-providers';
import { SessionExpiryProvider } from '@/shared/components/providers/session-expiry-provider';
import { UnifiedWeb3Provider } from '@/shared/components/providers/web3-provider';
import { createQueryClient } from '@/shared/state/query-client';
import type { State } from 'wagmi';

/**
 * @param root0
 * @param root0.children
 * @param root0.initialUser
 * @param root0.initialState
 */
export function ClientProviders({ children, initialUser, initialState }: { children: React.ReactNode; initialUser?: UserInfoResponse | null, initialState?: State }) {
  const [queryClient] = useState(() => createQueryClient('admin'));
  return (
    <QueryClientProvider client={queryClient}>
    <CommonProviders>
      <SessionExpiryProvider>
        <UnifiedWeb3Provider
          appName="EPSX Admin"
          isAdminMode={true}
          chains={[bsc, bscTestnet, foundry]}
          learnMoreUrl="https://admin.epsx.io/docs"
          initialState={initialState}
        >
          <SettingsProvider>
            <SharedOpenIDWeb3Provider
              clientId="epsx-admin"
              initialUser={initialUser}
            >
              {children}
            </SharedOpenIDWeb3Provider>
          </SettingsProvider>
        </UnifiedWeb3Provider>
      </SessionExpiryProvider>
    </CommonProviders>
    </QueryClientProvider>
  );
}