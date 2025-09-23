'use client';

import '@/lib/browser-polyfills';
import '@rainbow-me/rainbowkit/styles.css';
import { connectorsForWallets, RainbowKitProvider, darkTheme, lightTheme } from '@rainbow-me/rainbowkit';
import { metaMaskWallet, walletConnectWallet, injectedWallet } from '@rainbow-me/rainbowkit/wallets';
import { createConfig, http } from 'wagmi';
import { WagmiProvider } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';
import { QueryClientProvider, QueryClient } from '@tanstack/react-query';
import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from 'react';
import { useTheme } from 'next-themes';

// Create stable instances to prevent re-initialization
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30000,
      refetchOnWindowFocus: false,
    },
  },
});

// Support both BSC chains
const supportedChains = [bsc, bscTestnet];

// Create connectors immediately when called
const createConnectors = () => {
  console.log('🔄 createConnectors called, window exists:', typeof window !== 'undefined');
  
  if (typeof window === 'undefined') {
    console.log('❌ Window not available, returning empty connectors');
    return [];
  }
  
  try {
    console.log('🔗 Creating RainbowKit connectors...');
    console.log('📦 Available wallet functions:', {
      metaMaskWallet: typeof metaMaskWallet,
      walletConnectWallet: typeof walletConnectWallet,
      injectedWallet: typeof injectedWallet,
      connectorsForWallets: typeof connectorsForWallets
    });
    
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
    
    console.log(`✅ Successfully created ${connectors.length} wallet connectors`);
    console.log('🔗 Connector details:', connectors.map(c => ({ name: c.name, id: c.id, type: c.type })));
    return connectors;
  } catch (error) {
    console.error('❌ Error creating connectors:', error);
    console.error('❌ Error stack:', error.stack);
    return [];
  }
};

// Create wagmi config with connectors
const createWagmiConfig = (connectors: any[]) => {
  return createConfig({
    connectors,
    chains: supportedChains,
    transports: {
      [bsc.id]: http(),
      [bscTestnet.id]: http(),
    },
    ssr: false,
  });
};

// Web3 Context
interface Web3ContextType {
  isInitialized: boolean;
  forceReset: () => void;
  forceRecreateConnectors: () => void;
  getCurrentConfig: () => any;
}

const Web3Context = createContext<Web3ContextType>({
  isInitialized: false,
  forceReset: () => {},
  forceRecreateConnectors: () => {},
  getCurrentConfig: () => null,
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

export function Web3ProviderClient({ children }: Web3ProviderProps) {
  console.log('🚀 Web3ProviderClient RENDER - Component called');
  
  const [mounted, setMounted] = useState(false);
  const [config, setConfig] = useState<any>(null);
  const [isInitializing, setIsInitializing] = useState(true);
  
  console.log('🎯 State values:', { mounted, config: !!config, isInitializing });

  // Simplified useEffect for debugging
  useEffect(() => {
    console.log('🚀 SIMPLE USEEFFECT WORKS!!!');
    
    setTimeout(() => {
      console.log('🚀 Timeout works too!');
      setMounted(true);
      
      try {
        console.log('🔗 About to create connectors...');
        const connectors = createConnectors();
        console.log(`🔗 Successfully created ${connectors.length} connectors`);
        
        const wagmiConfig = createWagmiConfig(connectors);
        setConfig(wagmiConfig);
        setIsInitializing(false);
        
        console.log('✅ Everything initialized successfully');
      } catch (error) {
        console.error('❌ Initialization error:', error);
        const fallbackConfig = createWagmiConfig([]);
        setConfig(fallbackConfig);
        setIsInitializing(false);
      }
    }, 1000);
  }, []);

  // Move theme hook AFTER useEffect
  let theme;
  try {
    const themeHook = useTheme();
    theme = themeHook.theme;
    console.log('🎨 Theme loaded:', theme);
  } catch (error) {
    console.error('❌ Theme hook error:', error);
    theme = 'light';
  }

  const forceRecreateConnectors = useCallback(() => {
    console.log('🔄 Recreating connectors...');
    setIsInitializing(true);
    
    // Clear localStorage
    const keysToRemove = [
      'wagmi.cache',
      'wagmi.store', 
      'wagmi.recent-connectors',
      'wagmi.wallet',
      'wagmi.connected',
      'wagmi.recentConnector'
    ];
    
    keysToRemove.forEach(key => {
      try {
        window.localStorage.removeItem(key);
      } catch (e) {
        // Ignore
      }
    });
    
    try {
      // Recreate config
      const connectors = createConnectors();
      console.log(`🔗 Recreated ${connectors.length} connectors`);
      
      const wagmiConfig = createWagmiConfig(connectors);
      setConfig(wagmiConfig);
      setIsInitializing(false);
    } catch (error) {
      console.error('❌ Failed to recreate connectors:', error);
      setIsInitializing(false);
    }
  }, []);

  const forceReset = useCallback(() => {
    window.location.reload();
  }, []);

  const getCurrentConfig = useCallback(() => {
    return config;
  }, [config]);

  // Theme configuration (after theme is defined)
  const rainbowKitTheme = (theme === 'dark') 
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

  // Always render children, even during initialization
  // This prevents blocking the entire app
  if (!mounted) {
    // Only show loading during SSR/hydration
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500 mx-auto mb-2"></div>
          <p className="text-sm text-gray-600 dark:text-gray-400">Loading...</p>
        </div>
      </div>
    );
  }

  // If config failed to load, create a minimal fallback
  const safeConfig = config || createWagmiConfig([]);

  return (
    <Web3Context.Provider 
      value={{ 
        isInitialized: mounted && !!config && !isInitializing, 
        forceReset,
        forceRecreateConnectors,
        getCurrentConfig
      }}
    >
      <WagmiProvider config={safeConfig} reconnectOnMount={false}>
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