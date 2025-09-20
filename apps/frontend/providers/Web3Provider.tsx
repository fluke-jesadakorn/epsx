'use client';

import '@rainbow-me/rainbowkit/styles.css';
import { getDefaultConfig, RainbowKitProvider } from '@rainbow-me/rainbowkit';
import { WagmiProvider } from 'wagmi';
import { mainnet, polygon, arbitrum, base, optimism } from 'wagmi/chains';
import { QueryClientProvider, QueryClient } from '@tanstack/react-query';
import { createContext, useContext, type ReactNode } from 'react';

// Create Query Client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30000, // 30 seconds
    },
  },
});

// Configure Wagmi
const config = getDefaultConfig({
  appName: 'EPSX - Web3 Trading Platform',
  projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend',
  chains: [mainnet, polygon, arbitrum, base, optimism],
  ssr: true, // Next.js SSR support
});

// Web3 Context for additional state management
interface Web3ContextType {
  isInitialized: boolean;
}

const Web3Context = createContext<Web3ContextType>({
  isInitialized: true,
});

export const useWeb3Context = () => {
  const context = useContext(Web3Context);
  if (!context) {
    throw new Error('useWeb3Context must be used within Web3Provider');
  }
  return context;
};

interface Web3ProviderProps {
  children: ReactNode;
}

export function Web3Provider({ children }: Web3ProviderProps) {
  return (
    <Web3Context.Provider value={{ isInitialized: true }}>
      <WagmiProvider config={config}>
        <QueryClientProvider client={queryClient}>
          <RainbowKitProvider
            modalSize="compact"
            showRecentTransactions={true}
            appInfo={{
              appName: 'EPSX',
              learnMoreUrl: 'https://epsx.io/docs',
            }}
          >
            {children}
          </RainbowKitProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </Web3Context.Provider>
  );
}