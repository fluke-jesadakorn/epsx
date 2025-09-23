'use client';

import '@/lib/browser-polyfills';
import {
  connectorsForWallets,
  darkTheme,
  RainbowKitProvider,
} from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
import {
  injectedWallet,
  metaMaskWallet,
  walletConnectWallet,
} from '@rainbow-me/rainbowkit/wallets';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { type ReactNode } from 'react';
import { createConfig, http, WagmiProvider } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

// Create query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 60000,
      refetchOnWindowFocus: false,
    },
  },
});

// Create connectors with minimal config
const connectors = connectorsForWallets(
  [
    {
      groupName: 'Popular',
      wallets: [metaMaskWallet, walletConnectWallet, injectedWallet],
    },
  ],
  {
    appName: 'EPSX',
    projectId:
      process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend',
  }
);

// Create wagmi config
const wagmiConfig = createConfig({
  connectors,
  chains: [bsc, bscTestnet],
  transports: {
    [bsc.id]: http(),
    [bscTestnet.id]: http(),
  },
  ssr: false,
});

console.log('🔥 MinimalWeb3Provider: Simple config created');

interface MinimalWeb3ProviderProps {
  children: ReactNode;
}

export function MinimalWeb3Provider({ children }: MinimalWeb3ProviderProps) {
  console.log('🔥 MinimalWeb3Provider: Rendering');

  // Minimal dark theme
  const minimalDarkTheme = darkTheme({
    accentColor: '#f97316', // Orange
    accentColorForeground: 'white',
    borderRadius: 'medium',
    fontStack: 'system',
    overlayBlur: 'small',
  });

  return (
    <WagmiProvider config={wagmiConfig} reconnectOnMount={true}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider
          theme={minimalDarkTheme}
          modalSize="compact"
          showRecentTransactions={false}
          connectModalIntro="Select your wallet"
          appInfo={{
            appName: 'EPSX',
          }}
        >
          {children}
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
