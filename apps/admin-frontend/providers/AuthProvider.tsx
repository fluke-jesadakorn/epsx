'use client';

import { darkTheme, getDefaultConfig, lightTheme, RainbowKitProvider } from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTheme } from 'next-themes';
import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { WagmiProvider } from 'wagmi';
import { bscTestnet } from 'wagmi/chains';

// Polyfill for SSR to prevent indexedDB errors
if (typeof window === 'undefined') {
  (global as any).indexedDB = undefined;
}

// Query client factory for admin-optimized settings
const createQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: {
      retry: 3,
      staleTime: 60000, // 1 minute for admin data
      gcTime: 300000, // 5 minutes garbage collection time
    },
  },
});

// Configure Wagmi for Admin with BSC testnet - Client-only
let config: any = null;

// Only create config on client side to prevent SSR issues
if (typeof window !== 'undefined') {
  config = getDefaultConfig({
    appName: 'EPSX Admin - Web3 Management',
    projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || '9cc4d9939ee2ffbd56accefa1eec6f06',
    chains: [bscTestnet],
    ssr: true,
  });
}

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
  const [queryClient] = useState(() => createQueryClient());
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Always provide the context but conditionally render Web3 providers
  return (
    <AdminWeb3Context.Provider value={{ isInitialized: mounted && !!config, isAdminMode: true }}>
      {mounted && config ? (
        <WagmiProvider config={config}>
          <QueryClientProvider client={queryClient}>
            <ThemedRainbowKitProvider>
              {children}
            </ThemedRainbowKitProvider>
          </QueryClientProvider>
        </WagmiProvider>
      ) : (
        // SSR fallback - render children without Web3 providers
        children
      )}
    </AdminWeb3Context.Provider>
  );
}

// Export as Web3Provider for consistent naming
export { AdminWeb3Provider as Web3Provider };
