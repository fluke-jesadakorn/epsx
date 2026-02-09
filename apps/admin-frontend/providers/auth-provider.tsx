'use client';

import { darkTheme, getDefaultConfig, lightTheme, RainbowKitProvider } from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
import { QueryClientProvider } from '@tanstack/react-query';
import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { WagmiProvider, type Config } from 'wagmi';
import { bscTestnet } from 'wagmi/chains';

// Use shared query client factory
import { createQueryClient } from '@/shared/state';

// ============================================================================
// MODULE-LEVEL SINGLETONS
// These must be created outside of React components to prevent WalletConnect
// event listener accumulation during hot reloads (causes MaxListenersExceededWarning)
// ============================================================================

let wagmiConfigSingleton: Config | null = null;
let queryClientSingleton: ReturnType<typeof createQueryClient> | null = null;

function getWagmiConfig(): Config {
  wagmiConfigSingleton ??= getDefaultConfig({
    appName: 'EPSX Admin - Web3 Management',
    projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID ?? '9cc4d9939ee2ffbd56accefa1eec6f06',
    chains: [bscTestnet],
    ssr: true,
  });
  return wagmiConfigSingleton;
}

function getQueryClient() {
  queryClientSingleton ??= createQueryClient('admin');
  return queryClientSingleton;
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

/**
 * Hook to access the Admin Web3 Context
 */
export const useAdminWeb3Context = (): AdminWeb3ContextType => {
  return useContext(AdminWeb3Context);
};

interface AdminWeb3ProviderProps {
  children: ReactNode;
}

// Theme-aware RainbowKit wrapper component
function ThemedRainbowKitProvider({ children }: { children: ReactNode }): React.ReactElement {
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

/**
 *
 * @param root0
 * @param root0.children
 */
export function AdminWeb3Provider({ children }: AdminWeb3ProviderProps): React.ReactElement {
  // Use module-level singletons to prevent WalletConnect event listener accumulation on hot reloads
  const config = typeof window !== 'undefined' ? getWagmiConfig() : null;
  const queryClient = getQueryClient();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Always provide the context but conditionally render Web3 providers
  return (
    <AdminWeb3Context.Provider value={{ isInitialized: mounted && Boolean(config), isAdminMode: true }}>
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
