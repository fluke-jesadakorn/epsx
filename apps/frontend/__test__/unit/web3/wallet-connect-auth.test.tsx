import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WalletConnectAuth } from '@/components/auth/wallet-connect-auth';
import { useSharedAuth } from '@/shared/components/auth';
import { useRouter } from 'next/navigation';

// Mock dependencies
jest.mock('@/shared/components/auth');
jest.mock('next/navigation');
jest.mock('wagmi', () => ({
  WagmiProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  useAccount: jest.fn(),
  useSignMessage: jest.fn(),
}));
jest.mock('@rainbow-me/rainbowkit', () => ({
  RainbowKitProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

const mockUseSharedAuth = useSharedAuth as jest.MockedFunction<typeof useSharedAuth>;
const mockUseRouter = useRouter as jest.MockedFunction<typeof useRouter>;

// Mock router
const mockPush = jest.fn();
mockUseRouter.mockReturnValue({ push: mockPush } as any);

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
      {children}
    </QueryClientProvider>
  );
}

describe('WalletConnectauth', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  const { useAccount, useSignMessage } = require('wagmi');
  const mockUseAccount = useAccount;
  const mockUseSignMessage = useSignMessage;

  const defaultSharedAuthState = {
    user: null,
    isLoading: false,
    error: null,
    requestChallenge: jest.fn(),
    authenticateWithWallet: jest.fn(),
    refreshUser: jest.fn(),
    logout: jest.fn(),
  };

  const defaultAccountState = {
    address: undefined,
    isConnected: false,
  };

  const defaultSignMessageState = {
    signMessageAsync: jest.fn(),
  };

  describe('Not Connected State', () => {
    beforeEach(() => {
      mockUseSharedAuth.mockReturnValue(defaultSharedAuthState);
      mockUseAccount.mockReturnValue(defaultAccountState as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);
    });

    it('renders connect wallet button for default variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
    });

    it('renders compact connect button for compact variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth compact={true} />
        </TestWrapper>
      );

      expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
    });

    it('applies custom className', () => {
      const { container } = render(
        <TestWrapper>
          <WalletConnectAuth className="custom-class" />
        </TestWrapper>
      );

      expect(container.firstChild?.firstChild).toHaveClass('custom-class');
    });
  });

  describe('Connected but Not Authenticated State', () => {
    beforeEach(() => {
      mockUseSharedAuth.mockReturnValue(defaultSharedAuthState);
      mockUseAccount.mockReturnValue({
        address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        isConnected: true,
      } as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);
    });

    it('renders wallet address and sign message button', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('0x742d...45c6')).toBeInTheDocument();
      expect(screen.getByText('Sign Message')).toBeInTheDocument();
      expect(screen.getByText(/Connected:/)).toBeInTheDocument();
    });

    it('displays error message when present', () => {
      mockUseSharedAuth.mockReturnValue({
        ...defaultSharedAuthState,
        error: 'User rejected signature',
      });

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('User rejected signature')).toBeInTheDocument();
    });

    it('shows compact sign message for compact variant', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth compact={true} />
        </TestWrapper>
      );

      expect(screen.getByText('Sign Message')).toBeInTheDocument();
    });
  });

  describe('Authenticated State', () => {
    beforeEach(() => {
      mockUseSharedAuth.mockReturnValue({
        ...defaultSharedAuthState,
        user: {
          wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        } as any,
      });
      mockUseAccount.mockReturnValue({
        address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        isConnected: true,
      } as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);
    });

    it('renders authenticated state with wallet address', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      expect(screen.getByText('0x742d...45c6')).toBeInTheDocument();
    });

    it('shows wallet dropdown when authenticated', () => {
      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      // Component should render the ConnectedWalletDropdown
      const walletButton = screen.getByText('0x742d...45c6');
      expect(walletButton).toBeInTheDocument();
    });
  });

  describe('Callback Functions', () => {
    it('calls onAuthSuccess when authenticated', () => {
      const onAuthSuccess = jest.fn();

      mockUseSharedAuth.mockReturnValue({
        ...defaultSharedAuthState,
        user: {
          wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        } as any,
      });
      mockUseAccount.mockReturnValue({
        address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        isConnected: true,
      } as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);

      render(
        <TestWrapper>
          <WalletConnectAuth onAuthSuccess={onAuthSuccess} />
        </TestWrapper>
      );

      expect(onAuthSuccess).toHaveBeenCalledWith('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6');
    });

    it('calls onAuthError when error occurs', () => {
      const onAuthError = jest.fn();

      mockUseSharedAuth.mockReturnValue({
        ...defaultSharedAuthState,
        error: 'Connection failed',
      });
      mockUseAccount.mockReturnValue(defaultAccountState as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);

      render(
        <TestWrapper>
          <WalletConnectAuth onAuthError={onAuthError} />
        </TestWrapper>
      );

      expect(onAuthError).toHaveBeenCalledWith('Connection failed');
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

      expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles null wallet address gracefully', () => {
      mockUseSharedAuth.mockReturnValue(defaultSharedAuthState);
      mockUseAccount.mockReturnValue({
        address: undefined,
        isConnected: true,
      } as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);

      const { container } = render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      // Should not crash
      expect(container).toBeInTheDocument();
    });

    it('handles authenticated user', () => {
      mockUseSharedAuth.mockReturnValue({
        ...defaultSharedAuthState,
        user: {
          wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        } as any,
      });
      mockUseAccount.mockReturnValue({
        address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        isConnected: true,
      } as any);
      mockUseSignMessage.mockReturnValue(defaultSignMessageState as any);

      render(
        <TestWrapper>
          <WalletConnectAuth />
        </TestWrapper>
      );

      // Should render wallet address
      expect(screen.getByText('0x742d...45c6')).toBeInTheDocument();
    });
  });
});