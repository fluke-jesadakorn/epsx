'use client';

import '@/lib/browser-polyfills';
import '@rainbow-me/rainbowkit/styles.css';
import { connectorsForWallets, RainbowKitProvider, darkTheme, lightTheme } from '@rainbow-me/rainbowkit';
import { metaMaskWallet, walletConnectWallet, injectedWallet } from '@rainbow-me/rainbowkit/wallets';
import { createConfig, http } from 'wagmi';
import { WagmiProvider } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';
import { QueryClientProvider, QueryClient } from '@tanstack/react-query';
import { createContext, useContext, type ReactNode } from 'react';
import { useTheme } from 'next-themes';

// Create query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30000,
      refetchOnWindowFocus: false,
    },
  },
});

// Create connectors
const connectors = connectorsForWallets(
  [
    {
      groupName: 'Recommended',
      wallets: [
        metaMaskWallet,
        walletConnectWallet,
        injectedWallet,
      ],
    },
  ],
  {
    appName: 'EPSX - Web3 Trading Platform',
    projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend',
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

console.log('🎯 WorkingWeb3Provider: Created config with connectors at module level');

// Context interface matching SimpleWeb3Provider
interface Web3ContextType {
  isInitialized: boolean;
  forceReset: () => void;
  forceRecreateConnectors: () => void;
  getCurrentConfig: () => any;
}

// Create context with simple implementations
const Web3Context = createContext<Web3ContextType>({
  isInitialized: true,
  forceReset: () => window.location.reload(),
  forceRecreateConnectors: () => window.location.reload(),
  getCurrentConfig: () => wagmiConfig,
});

// Export the hook for WalletConnectionModal
export const useWeb3Context = () => {
  const context = useContext(Web3Context);
  if (!context) {
    throw new Error('useWeb3Context must be used within WorkingWeb3Provider');
  }
  return context;
};

interface WorkingWeb3ProviderProps {
  children: ReactNode;
}

export function WorkingWeb3Provider({ children }: WorkingWeb3ProviderProps) {
  console.log('🎯 WorkingWeb3Provider: Rendering component');
  
  const { theme } = useTheme();

  const rainbowKitTheme = theme === 'dark' 
    ? darkTheme({
        accentColor: '#f97316',
        accentColorForeground: 'white',
        borderRadius: 'large',
        fontStack: 'system',
        overlayBlur: 'small'
      })
    : lightTheme({
        accentColor: '#f97316',
        accentColorForeground: 'white',
        borderRadius: 'large',
        fontStack: 'system',
        overlayBlur: 'small'
      });

  const contextValue: Web3ContextType = {
    isInitialized: true, // Always true since connectors created at module level
    forceReset: () => window.location.reload(),
    forceRecreateConnectors: () => window.location.reload(),
    getCurrentConfig: () => wagmiConfig,
  };

  return (
    <Web3Context.Provider value={contextValue}>
      <WagmiProvider config={wagmiConfig} reconnectOnMount={false}>
        <QueryClientProvider client={queryClient}>
          <RainbowKitProvider
            theme={rainbowKitTheme}
            modalSize="compact"
            showRecentTransactions={false}
            connectModalIntro="Connect your wallet to EPSX"
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