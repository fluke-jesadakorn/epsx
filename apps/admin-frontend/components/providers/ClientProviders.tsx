'use client';

import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/Provider';
import { CommonProviders } from '@/shared/components/providers/CommonProviders';
import { UnifiedWeb3Provider } from '@/shared/components/providers/UnifiedWeb3Provider';
import React from 'react';
import { bscTestnet } from 'wagmi/chains';
import { SettingsProvider } from './SettingsProvider';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <CommonProviders>
      <UnifiedWeb3Provider
        appName="EPSX Admin"
        isAdminMode={true}
        chains={[bscTestnet]}
        learnMoreUrl="https://admin.epsx.io/docs"
      >
        <SettingsProvider>
          <SharedOpenIDWeb3Provider
            clientId="epsx-admin"
            backendUrl={process.env.NEXT_PUBLIC_BACKEND_URL}
          >
            {children}
          </SharedOpenIDWeb3Provider>
        </SettingsProvider>
      </UnifiedWeb3Provider>
    </CommonProviders>
  );
}