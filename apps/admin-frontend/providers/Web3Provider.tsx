'use client';

import '@rainbow-me/rainbowkit/styles.css';
import { getDefaultConfig, RainbowKitProvider, darkTheme, lightTheme } from '@rainbow-me/rainbowkit';
import { WagmiProvider } from 'wagmi';
import { mainnet, polygon, arbitrum, base, optimism } from 'wagmi/chains';
import { QueryClientProvider, QueryClient } from '@tanstack/react-query';
import { createContext, useContext, type ReactNode } from 'react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

// Create Query Client with admin-optimized settings
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 3,
      staleTime: 60000, // 1 minute for admin data
      cacheTime: 300000, // 5 minutes cache
    },
  },
});

// Configure Wagmi for Admin with enhanced chains
const config = getDefaultConfig({
  appName: 'EPSX Admin - Web3 Management',
  projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-admin-web3',
  chains: [mainnet, polygon, arbitrum, base, optimism],
  ssr: true, // Next.js SSR support
});

// Admin Web3 Context for additional admin-specific state
interface AdminWeb3ContextType {
  isInitialized: boolean;
  isAdminMode: boolean;
}

const AdminWeb3Context = createContext<AdminWeb3ContextType>({
  isInitialized: true,
  isAdminMode: true,
});

export const useAdminWeb3Context = () => {
  const context = useContext(AdminWeb3Context);
  if (!context) {
    throw new Error('useAdminWeb3Context must be used within AdminWeb3Provider');
  }
  return context;
};

interface AdminWeb3ProviderProps {
  children: ReactNode;
}

// Theme-aware RainbowKit wrapper component
function ThemedRainbowKitProvider({ children }: { children: ReactNode }) {
  const { resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Return basic provider during SSR
  if (!mounted) {
    return (
      <RainbowKitProvider
        modalSize="compact"
        showRecentTransactions={true}
        appInfo={{
          appName: 'EPSX Admin',
          learnMoreUrl: 'https://admin.epsx.io/docs',
        }}
      >
        {children}
      </RainbowKitProvider>
    );
  }

  // PancakeSwap Admin theme configuration
  const rainbowKitTheme = {
    lightMode: lightTheme({
      accentColor: '#f97316', // orange-500 - matches admin primary color
      accentColorForeground: 'white',
      borderRadius: 'large',
      fontStack: 'system',
      overlayBlur: 'small',
    }),
    darkMode: darkTheme({
      accentColor: '#fbbf24', // yellow-400 - better visibility in dark mode
      accentColorForeground: '#1f2937', // gray-800 - readable on yellow
      borderRadius: 'large',
      fontStack: 'system',
      overlayBlur: 'small',
    }),
  };

  return (
    <RainbowKitProvider
      theme={rainbowKitTheme}
      modalSize="compact"
      showRecentTransactions={true}
      appInfo={{
        appName: 'EPSX Admin',
        learnMoreUrl: 'https://admin.epsx.io/docs',
      }}
    >
      {children}
    </RainbowKitProvider>
  );
}

export function AdminWeb3Provider({ children }: AdminWeb3ProviderProps) {
  return (
    <AdminWeb3Context.Provider value={{ isInitialized: true, isAdminMode: true }}>
      <WagmiProvider config={config}>
        <QueryClientProvider client={queryClient}>
          <ThemedRainbowKitProvider>
            {children}
          </ThemedRainbowKitProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </AdminWeb3Context.Provider>
  );
}

// Export as Web3Provider for consistent naming
export { AdminWeb3Provider as Web3Provider };