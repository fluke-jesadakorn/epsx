'use client';

import '@/lib/polyfills';

import dynamic from 'next/dynamic';
import React from 'react';
import { bsc, bscTestnet, foundry } from 'wagmi/chains';

import { SettingsProvider } from './SettingsProvider';

import { AuthRedirectHandler } from '@/components/auth/AuthRedirectHandler';
import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/Provider';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';

// Dynamic import with ssr: false to prevent WalletConnect localStorage errors during SSR
// The @walletconnect/keyvaluestorage package tries to access localStorage during module init
const UnifiedWeb3Provider = dynamic(
  () => import('@/shared/components/providers/UnifiedWeb3Provider').then(mod => mod.UnifiedWeb3Provider),
  { ssr: false }
);

/**
 *
 * @param root0
 * @param root0.children
 */
export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <CommonProviders>
      <UnifiedWeb3Provider
        appName="EPSX Admin"
        isAdminMode={true}
        chains={[bsc, bscTestnet, foundry]}
        learnMoreUrl="https://admin.epsx.io/docs"
      >
        <SettingsProvider>
          <SharedOpenIDWeb3Provider
            clientId="epsx-admin"
            backendUrl={process.env.NEXT_PUBLIC_BACKEND_URL}
          >
            <AuthRedirectHandler />
            {children}
          </SharedOpenIDWeb3Provider>
        </SettingsProvider>
      </UnifiedWeb3Provider>
    </CommonProviders>
  );
}