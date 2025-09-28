'use client';

import '@/lib/browser-polyfills';
import '@rainbow-me/rainbowkit/styles.css';
import {
  getDefaultConfig,
  darkTheme,
  RainbowKitProvider,
} from '@rainbow-me/rainbowkit';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { type ReactNode, useState, useEffect, createContext, useContext } from 'react';
import { WagmiProvider } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

// Query client factory function
const createQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 60000,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: false,
      onError: (error) => {
        // Silently handle wallet connection errors
        if (error instanceof Error && (
          error.message.includes('User rejected') ||
          error.message.includes('User denied') ||
          error.message.includes('cancelled')
        )) {
          return; // Suppress user cancellation errors
        }
      },
    },
  },
});

// Get the blockchain network from environment
const isMainnet = process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK === 'mainnet';
const chains = isMainnet ? [bsc] as const : [bscTestnet, bsc] as const;

// Singleton wagmi config to prevent multiple WalletConnect initializations
let wagmiConfig: ReturnType<typeof getDefaultConfig> | null = null;
let isInitializing = false;

function getWagmiConfig() {
  if (!wagmiConfig && !isInitializing) {
    isInitializing = true;
    try {
      wagmiConfig = getDefaultConfig({
        appName: 'EPSX',
        projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend',
        chains,
        ssr: true, // Enable SSR support
      });
    } finally {
      isInitializing = false;
    }
  }
  return wagmiConfig;
}


// Web3 Context for compatibility with existing components
interface Web3ContextType {
  isInitialized: boolean;
  forceReset: () => void;
  forceRecreateConnectors: () => void;
}

const Web3Context = createContext<Web3ContextType>({
  isInitialized: false,
  forceReset: () => window?.location?.reload(),
  forceRecreateConnectors: () => window?.location?.reload(),
});

export const useWeb3Context = () => {
  const context = useContext(Web3Context);
  if (!context) {
    throw new Error('useWeb3Context must be used within MinimalWeb3Provider');
  }
  return context;
};

interface MinimalWeb3ProviderProps {
  children: ReactNode;
}

export function MinimalWeb3Provider({ children }: MinimalWeb3ProviderProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [queryClient] = useState(() => createQueryClient());

  // Simple fallback implementations for compatibility
  const forceReset = () => {
    if (typeof window !== 'undefined') {
      window.location.reload();
    }
  };

  const forceRecreateConnectors = () => {
    if (typeof window !== 'undefined') {
      window.location.reload();
    }
  };

  // Prevent hydration issues by ensuring we're client-side
  useEffect(() => {
    setIsHydrated(true);

    // Handle wallet library and database cleanup errors that are safe to ignore
    const handleWalletErrors = (event: PromiseRejectionEvent) => {
      const error = event.reason;
      
      if (error instanceof TypeError) {
        const msg = error.message;
        // These specific errors are from wallet library and database cleanup and are harmless
        if (
          msg === "Cannot set properties of null (setting 'onclose')" ||
          msg === "Cannot read properties of null (reading 'transaction')" ||
          msg.includes("Cannot set properties of null") ||
          msg.includes("Cannot read properties of null") ||
          msg.includes("IndexedDB") ||
          msg.includes("WebSocket") ||
          msg.includes("transaction") ||
          msg.includes("onclose")
        ) {
          event.preventDefault(); // Prevent console spam
          return;
        }
      }

      // Also handle general connection cleanup errors from wallet extensions
      if (error && typeof error === 'object' && error.name === 'TypeError') {
        const errorStr = error.toString();
        if (
          errorStr.includes('null') && 
          (errorStr.includes('onclose') || errorStr.includes('transaction'))
        ) {
          event.preventDefault();
          return;
        }
      }
    };

    window.addEventListener('unhandledrejection', handleWalletErrors);
    return () => window.removeEventListener('unhandledrejection', handleWalletErrors);
  }, []);


  // Official RainbowKit theme configuration
  const epsxTheme = darkTheme({
    accentColor: '#f97316', // Orange
    accentColorForeground: 'white',
    borderRadius: 'medium',
    fontStack: 'system',
    overlayBlur: 'small',
  });

  // Get wagmi config safely
  const config = getWagmiConfig();
  
  // During hydration, provide minimal wagmi context without RainbowKit to prevent SSR issues
  if (!isHydrated || !config) {
    return (
      <Web3Context.Provider value={{ isInitialized: false, forceReset, forceRecreateConnectors }}>
        {config ? (
          <WagmiProvider config={config}>
            <QueryClientProvider client={queryClient}>
              {children}
            </QueryClientProvider>
          </WagmiProvider>
        ) : (
          <QueryClientProvider client={queryClient}>
            {children}
          </QueryClientProvider>
        )}
      </Web3Context.Provider>
    );
  }

  // Full RainbowKit setup after hydration (official pattern)
  return (
    <Web3Context.Provider value={{ isInitialized: true, forceReset, forceRecreateConnectors }}>
      <WagmiProvider config={config}>
        <QueryClientProvider client={queryClient}>
          <RainbowKitProvider
            theme={epsxTheme}
            modalSize="compact"
            showRecentTransactions={false}
            appInfo={{
              appName: 'EPSX',
              learnMoreUrl: 'https://epsx.io',
            }}
          >
            {children}
          </RainbowKitProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </Web3Context.Provider>
  );
}
