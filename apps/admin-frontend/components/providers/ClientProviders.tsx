'use client';
import '@/lib/polyfills';

import { bsc, bscTestnet, foundry } from '@/config/wagmi-chains';
import React from 'react';

import { SettingsProvider } from './SettingsProvider';

import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/Provider';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';
import { UnifiedWeb3Provider } from '@/shared/components/providers/UnifiedWeb3Provider';
import { State } from 'wagmi';

/**
 *
 * @param root0
 * @param root0.children
 */
export function ClientProviders({ children, initialUser, initialState }: { children: React.ReactNode; initialUser?: any, initialState?: State }) {
  return (
    <CommonProviders>
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
            backendUrl={process.env.NEXT_PUBLIC_BACKEND_URL}
            initialUser={initialUser}
          >
            {children}
          </SharedOpenIDWeb3Provider>
        </SettingsProvider>
      </UnifiedWeb3Provider>
    </CommonProviders>
  );
}