import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WagmiProvider } from 'wagmi';
import { RainbowKitProvider } from '@rainbow-me/rainbowkit';
import { WalletConnectAuth } from '@/components/auth/WalletConnectAuth';
import { useWeb3Auth } from '@/lib/auth/web3';
import { useRouter } from 'next/navigation';
import { config } from '@/lib/wagmi';

// Mock dependencies
jest.mock('@/lib/auth/web3');
jest.mock('next/navigation');
jest.mock('@/lib/wagmi');

const mockUseWeb3Auth = useWeb3Auth as jest.MockedFunction<typeof useWeb3Auth>;
const mockUseRouter = useRouter as jest.MockedFunction<typeof useRouter>;

// Mock router
const mockPush = jest.fn();
mockUseRouter.mockReturnValue({ push: mockPush } as any);

// Mock wagmi config
jest.mock('@/lib/wagmi', () => ({
  config: {}
}));

// Test wrapper with providers
function TestWrapper({ children }: { children: React.ReactNode }) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return (
    <QueryClientProvider client={queryClient}>
      <WagmiProvider config={config}>
        <RainbowKitProvider>
          {children}
        </RainbowKitProvider>
      </WagmiProvider>
    </QueryClientProvider>
  );
}

describe('WalletConnectAuth', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  const defaultWeb3AuthState = {
    isConnected: false,
    isAuthenticated: false,
    isAuthenticating: false,
    walletAddress: null,
    permissions: [],
    userTier: 'basic',
    hasApiAccess: false,
    error: null,
    authenticate: jest.fn(),
    disconnect: jest.fn(),
    checkAuthStatus: jest.fn(),
    refreshPermissions: jest.fn(),
  };

  describe('Not Connected State', () => {
    beforeEach(() => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: false,
      });
    });

    it('renders connect wallet button for default variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('Connect Web3 Wallet')).toBeInTheDocument();
      expect(screen.getByText('Connect your wallet to access Web3 features and permissions')).toBeInTheDocument();
    });

    it('renders compact connect button for compact variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth variant="compact" />
        </TestWrapper>
      );

      expect(screen.getByText('Connect')).toBeInTheDocument();
    });

    it('applies custom className', () => {
      const { container } = render(
        <TestWrapper>
          <WalletConnectAuth className="custom-class" />
        </TestWrapper>
      );

      expect(container.firstChild).toHaveClass('custom-class');
    });
  });

  describe('Connected but Not Authenticated State', () => {
    beforeEach(() => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
      });
    });

    it('renders wallet address and sign in button', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('0x742d...45c6')).toBeInTheDocument();
      expect(screen.getByText('Sign In with Wallet')).toBeInTheDocument();
      expect(screen.getByText('Sign a message to prove wallet ownership and access your Web3 permissions')).toBeInTheDocument();
    });

    it('calls authenticate when sign in button is clicked', async () => {
      const mockAuthenticate = jest.fn();
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        authenticate: mockAuthenticate,
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      fireEvent.click(screen.getByText('Sign In with Wallet'));
      expect(mockAuthenticate).toHaveBeenCalled();
    });

    it('shows authenticating state', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticating: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('Signing Message...')).toBeInTheDocument();
    });

    it('displays error message when present', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        error: 'User rejected signature',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('User rejected signature')).toBeInTheDocument();
    });

    it('shows compact sign in for compact variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth variant="compact" />
        </TestWrapper>
      );

      expect(screen.getByText('Connect')).toBeInTheDocument();
    });
  });

  describe('Authenticated State', () => {
    const authenticatedState = {
      ...defaultWeb3AuthState,
      isConnected: true,
      isAuthenticated: true,
      walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
      userTier: 'nft' as const,
      permissions: [
        { permission: 'admin:users:view', source: 'nft' },
        { permission: 'admin:settings:manage', source: 'manual' },
      ],
    };

    beforeEach(() => {
      mockUseWeb3Auth.mockReturnValue(authenticatedState);
    });

    it('renders authenticated state with wallet address and tier', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('0x742d...45c6')).toBeInTheDocument();
      expect(screen.getByText('nft')).toBeInTheDocument();
    });

    it('calls disconnect when disconnect button is clicked', async () => {
      const mockDisconnect = jest.fn();
      mockUseWeb3Auth.mockReturnValue({
        ...authenticatedState,
        disconnect: mockDisconnect,
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      const disconnectButton = screen.getByTitle('Disconnect Wallet');
      fireEvent.click(disconnectButton);
      expect(mockDisconnect).toHaveBeenCalled();
    });

    it('shows permissions indicator', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      // Should show permission icons for NFT and manual permissions
      expect(screen.getByText('nft')).toBeInTheDocument();
    });

    it('renders detailed view with all information', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('Web3 Authenticated')).toBeInTheDocument();
      expect(screen.getByText('NFT Tier')).toBeInTheDocument();
      expect(screen.getByText('Active Permissions (2)')).toBeInTheDocument();
      expect(screen.getByText('View Profile')).toBeInTheDocument();
    });

    it('navigates to profile when view profile is clicked', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      fireEvent.click(screen.getByText('View Profile'));
      expect(mockPush).toHaveBeenCalledWith('/profile');
    });

    it('shows API access badge when user has API access', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...authenticatedState,
        hasApiAccess: true,
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('API Access')).toBeInTheDocument();
      expect(screen.getByText('API Keys')).toBeInTheDocument();
    });

    it('navigates to API keys when API Keys button is clicked', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...authenticatedState,
        hasApiAccess: true,
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      fireEvent.click(screen.getByText('API Keys'));
      expect(mockPush).toHaveBeenCalledWith('/profile?tab=api');
    });

    it('shows compact authenticated view for compact variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth variant="compact" />
        </TestWrapper>
      );

      expect(screen.getByText('nft')).toBeInTheDocument();
    });
  });

  describe('Different User Tiers', () => {
    it('renders NFT tier with correct styling', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        userTier: 'nft',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('NFT Tier')).toBeInTheDocument();
    });

    it('renders token tier with correct styling', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        userTier: 'token',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('Token Tier')).toBeInTheDocument();
    });

    it('renders DAO tier with correct styling', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        userTier: 'dao',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('Dao Tier')).toBeInTheDocument();
    });

    it('renders enterprise tier with correct styling', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        userTier: 'enterprise',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('Enterprise Tier')).toBeInTheDocument();
    });
  });

  describe('Callback Functions', () => {
    it('calls onAuthSuccess when authenticated', () => {
      const onAuthSuccess = jest.fn();
      
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth onAuthSuccess={onAuthSuccess} />
        </TestWrapper>
      );

      expect(onAuthSuccess).toHaveBeenCalledWith('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6');
    });

    it('calls onAuthError when error occurs', () => {
      const onAuthError = jest.fn();
      
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        error: 'Connection failed',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth onAuthError={onAuthError} />
        </TestWrapper>
      );

      expect(onAuthError).toHaveBeenCalledWith('Connection failed');
    });
  });

  describe('Permission Display', () => {
    it('shows multiple permissions correctly', () => {
      const permissions = [
        { permission: 'admin:users:view', source: 'nft' },
        { permission: 'admin:users:manage', source: 'manual' },
        { permission: 'admin:settings:view', source: 'token' },
        { permission: 'admin:settings:manage', source: 'dao' },
        { permission: 'api:read', source: 'enterprise' },
      ];

      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        permissions,
        userTier: 'enterprise',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.getByText('Active Permissions (5)')).toBeInTheDocument();
      expect(screen.getByText('View all 5 permissions')).toBeInTheDocument();
    });

    it('shows permission indicator with overflow', () => {
      const permissions = [
        { permission: 'admin:users:view', source: 'nft' },
        { permission: 'admin:users:manage', source: 'manual' },
        { permission: 'admin:settings:view', source: 'token' },
        { permission: 'admin:settings:manage', source: 'dao' },
      ];

      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        permissions,
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      // Should show +1 indicator for the 4th permission
      expect(screen.getByText('+1')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels and roles', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      const disconnectButton = screen.getByTitle('Disconnect Wallet');
      expect(disconnectButton).toBeInTheDocument();
    });

    it('provides meaningful button text for screen readers', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('Connect Web3 Wallet')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles null wallet address gracefully', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        walletAddress: null,
      });

      const { container } = render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      // Should not crash
      expect(container).toBeInTheDocument();
    });

    it('handles empty permissions array', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        permissions: [],
      });

      render(
        <TestWrapper>
          <WalletConnectAuth variant="detailed" />
        </TestWrapper>
      );

      expect(screen.queryByText('Active Permissions')).not.toBeInTheDocument();
    });

    it('handles unknown user tier', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        userTier: 'unknown' as any,
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('unknown')).toBeInTheDocument();
    });
  });
});