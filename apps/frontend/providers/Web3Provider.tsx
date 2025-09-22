'use client';

// Import browser polyfills first to handle SSR issues
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

// Prevent multiple initializations with a simpler approach
let isWalletConnectInitialized = false;

// Create stable instances to prevent re-initialization
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30000, // 30 seconds
      refetchOnWindowFocus: false, // Prevent unnecessary refetches
    },
  },
});

// Support both BSC chains for real chain switching
const supportedChains = [bsc, bscTestnet];

// Create connectors only once
const getConnectors = () => {
  if (isWalletConnectInitialized) {
    console.log('♻️ WalletConnect already initialized, skipping connector creation');
    return null;
  }
  
  console.log('🚀 Creating WalletConnect connectors for the first time');
  isWalletConnectInitialized = true;
  
  return connectorsForWallets(
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
};

// Create config with optional connectors
const connectors = getConnectors();
const config = createConfig({
  connectors: connectors || [],
  chains: supportedChains,
  transports: {
    [bsc.id]: http(),
    [bscTestnet.id]: http(),
  },
  ssr: true,
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
  const { resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // Add IndexedDB polyfill for SSR
    if (typeof window === 'undefined' && typeof global !== 'undefined') {
      // @ts-ignore
      global.indexedDB = undefined;
    }
    setMounted(true);
  }, []);

  // Create theme based on current app theme - default to light for SSR
  const rainbowKitTheme = (!mounted || resolvedTheme === 'light') 
    ? lightTheme({
        accentColor: '#3b82f6', // Blue accent to match your design
        accentColorForeground: 'white',
        borderRadius: 'medium',
        overlayBlur: 'small'
      })
    : darkTheme({
        accentColor: '#3b82f6', // Blue accent to match your design
        accentColorForeground: 'white',
        borderRadius: 'medium',
        overlayBlur: 'small'
      });

  return (
    <Web3Context.Provider value={{ isInitialized: true }}>
      <WagmiProvider config={config}>
        <QueryClientProvider client={queryClient}>
          <RainbowKitProvider
            theme={rainbowKitTheme}
            modalSize="compact"
            showRecentTransactions={false} // Disable to reduce API calls
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