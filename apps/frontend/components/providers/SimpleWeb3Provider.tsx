'use client';

import '@/lib/browser-polyfills';
import '@rainbow-me/rainbowkit/styles.css';
import { connectorsForWallets, RainbowKitProvider, darkTheme, lightTheme } from '@rainbow-me/rainbowkit';
import { metaMaskWallet, walletConnectWallet, injectedWallet } from '@rainbow-me/rainbowkit/wallets';
import { createConfig, http } from 'wagmi';
import { WagmiProvider } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';
import { QueryClientProvider, QueryClient } from '@tanstack/react-query';
import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
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

// Create connectors function
const createConnectors = () => {
  console.log('🔗 Creating connectors, window available:', typeof window !== 'undefined');
  
  if (typeof window === 'undefined') {
    console.log('❌ Window not available');
    return [];
  }
  
  try {
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
    
    console.log(`✅ Successfully created ${connectors.length} connectors`);
    return connectors;
  } catch (error) {
    console.error('❌ Error creating connectors:', error);
    return [];
  }
};

// Create initial connectors (will be empty on server)
let connectors = createConnectors();

// Create wagmi config
const createWagmiConfig = (connectors: any[]) => {
  return createConfig({
    connectors,
    chains: [bsc, bscTestnet],
    transports: {
      [bsc.id]: http(),
      [bscTestnet.id]: http(),
    },
    ssr: false,
  });
};

let wagmiConfig = createWagmiConfig(connectors);

// Simple context
interface SimpleWeb3ContextType {
  isInitialized: boolean;
  forceReset: () => void;
  forceRecreateConnectors: () => void;
  getCurrentConfig: () => any;
}

const SimpleWeb3Context = createContext<SimpleWeb3ContextType>({
  isInitialized: true,
  forceReset: () => window.location.reload(),
  forceRecreateConnectors: () => window.location.reload(),
  getCurrentConfig: () => wagmiConfig,
});

export const useWeb3Context = () => {
  const context = useContext(SimpleWeb3Context);
  if (!context) {
    throw new Error('useWeb3Context must be used within SimpleWeb3Provider');
  }
  return context;
};

interface SimpleWeb3ProviderProps {
  children: ReactNode;
}

export function SimpleWeb3Provider({ children }: SimpleWeb3ProviderProps) {
  const [mounted, setMounted] = useState(false);
  const [clientConfig, setClientConfig] = useState(wagmiConfig);
  
  console.log('🎯 SimpleWeb3Provider: Rendering', {
    mounted,
    hasWindow: typeof window !== 'undefined',
    connectorsLength: connectors.length
  });
  
  // Force client-side mounting and connector creation
  useEffect(() => {
    console.log('🔄 SimpleWeb3Provider: useEffect - client side!');
    setMounted(true);
    
    // Create connectors on client side
    console.log('🔄 Creating connectors on true client side...');
    const clientConnectors = createConnectors();
    const clientWagmiConfig = createWagmiConfig(clientConnectors);
    setClientConfig(clientWagmiConfig);
  }, []);
  
  const { theme } = useTheme();

  // Don't render until mounted (client-side only)
  if (!mounted) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500 mx-auto mb-2"></div>
          <p className="text-sm text-gray-600 dark:text-gray-400">Loading wallet connectors...</p>
        </div>
      </div>
    );
  }

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

  const contextValue: SimpleWeb3ContextType = {
    isInitialized: mounted,
    forceReset: () => window.location.reload(),
    forceRecreateConnectors: () => window.location.reload(),
    getCurrentConfig: () => clientConfig,
  };

  return (
    <SimpleWeb3Context.Provider value={contextValue}>
      <WagmiProvider config={clientConfig} reconnectOnMount={false}>
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
    </SimpleWeb3Context.Provider>
  );
}