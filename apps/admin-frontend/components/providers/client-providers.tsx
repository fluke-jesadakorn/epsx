'use client';
import '@/lib/polyfills';

import { bsc, bscTestnet, foundry } from '@/config/wagmi-chains';
import React from 'react';

import { SettingsProvider } from './settings-provider';

import { SharedOpenIDWeb3Provider } from '@/shared/components/auth';
import { CommonProviders } from '@/shared/components/providers/common-providers';
import { SessionExpiryProvider } from '@/shared/components/providers/session-expiry-provider';
import { UnifiedWeb3Provider } from '@/shared/components/providers/web3-provider';
import type { State } from 'wagmi';

/**
 *
 * @param root0
 * @param root0.children
 */
export function ClientProviders({ children, initialUser, initialState }: { children: React.ReactNode; initialUser?: any, initialState?: State }) {
  return (
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
  );
}