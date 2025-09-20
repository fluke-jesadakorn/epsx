import { render, screen, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WagmiProvider } from 'wagmi';
import { RainbowKitProvider } from '@rainbow-me/rainbowkit';
import { 
  Web3AuthProvider, 
  useWeb3AuthContext, 
  withWeb3Auth,
  useWeb3Permission,
  useWeb3Tier
} from '@/providers/Web3AuthProvider';
import { useWeb3Auth } from '@/lib/auth/web3';
import { useAccount } from 'wagmi';
import { toast } from 'sonner';
import { config } from '@/lib/wagmi';

// Mock dependencies
jest.mock('@/lib/auth/web3');
jest.mock('wagmi');
jest.mock('sonner');
jest.mock('@/lib/wagmi');

const mockUseWeb3Auth = useWeb3Auth as jest.MockedFunction<typeof useWeb3Auth>;
const mockUseAccount = useAccount as jest.MockedFunction<typeof useAccount>;
const mockToast = toast as jest.MockedFunction<typeof toast>;

// Mock fetch for session endpoint
global.fetch = jest.fn();
const mockFetch = fetch as jest.MockedFunction<typeof fetch>;

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

// Test component to access context
function TestConsumer() {
  const context = useWeb3AuthContext();
  return (
    <div>
      <div data-testid="is-loading">{context.isLoading.toString()}</div>
      <div data-testid="has-initialized">{context.hasInitialized.toString()}</div>
      <div data-testid="is-connected">{context.isConnected.toString()}</div>
      <div data-testid="is-authenticated">{context.isAuthenticated.toString()}</div>
      <div data-testid="wallet-address">{context.walletAddress || 'null'}</div>
      <div data-testid="user-tier">{context.userTier}</div>
      <div data-testid="permissions-count">{context.permissions.length}</div>
    </div>
  );
}

describe('Web3AuthProvider', () => {
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

  beforeEach(() => {
    jest.clearAllMocks();
    mockUseAccount.mockReturnValue({ isConnected: false } as any);
    mockUseWeb3Auth.mockReturnValue(defaultWeb3AuthState);
    mockFetch.mockClear();
  });

  describe('Provider Initialization', () => {
    it('initializes with loading state', async () => {
      const mockCheckAuthStatus = jest.fn().mockResolvedValue(undefined);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        checkAuthStatus: mockCheckAuthStatus,
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      // Should start in loading state
      expect(screen.getByTestId('is-loading')).toHaveTextContent('true');
      expect(screen.getByTestId('has-initialized')).toHaveTextContent('false');

      // Wait for initialization
      await waitFor(() => {
        expect(screen.getByTestId('has-initialized')).toHaveTextContent('true');
        expect(screen.getByTestId('is-loading')).toHaveTextContent('false');
      });
    });

    it('checks auth status when wallet is connected', async () => {
      const mockCheckAuthStatus = jest.fn().mockResolvedValue(undefined);
      mockUseAccount.mockReturnValue({ isConnected: true } as any);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        checkAuthStatus: mockCheckAuthStatus,
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(mockCheckAuthStatus).toHaveBeenCalled();
      });
    });

    it('handles initialization error gracefully', async () => {
      const mockCheckAuthStatus = jest.fn().mockRejectedValue(new Error('Auth check failed'));
      mockUseAccount.mockReturnValue({ isConnected: true } as any);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        checkAuthStatus: mockCheckAuthStatus,
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(mockToast.error).toHaveBeenCalledWith('Failed to initialize authentication');
      });

      // Should still finish initialization
      await waitFor(() => {
        expect(screen.getByTestId('has-initialized')).toHaveTextContent('true');
        expect(screen.getByTestId('is-loading')).toHaveTextContent('false');
      });
    });
  });

  describe('Auto-Authentication', () => {
    it('attempts auto-authentication for connected wallet with valid session', async () => {
      const mockRefreshPermissions = jest.fn().mockResolvedValue(undefined);
      
      mockUseAccount.mockReturnValue({ isConnected: true } as any);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        refreshPermissions: mockRefreshPermissions,
      });

      // Mock successful session response
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          wallet_address: '0x123',
          is_authenticated: true
        }),
      } as Response);

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith('/api/auth/session', {
          credentials: 'include',
        });
      });

      await waitFor(() => {
        expect(mockRefreshPermissions).toHaveBeenCalled();
      });
    });

    it('skips auto-authentication when already authenticated', async () => {
      mockUseAccount.mockReturnValue({ isConnected: true } as any);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('has-initialized')).toHaveTextContent('true');
      });

      // Should not make session check
      expect(mockFetch).not.toHaveBeenCalled();
    });

    it('handles failed session check gracefully', async () => {
      mockUseAccount.mockReturnValue({ isConnected: true } as any);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
      });

      // Mock failed session response
      mockFetch.mockResolvedValueOnce({
        ok: false,
      } as Response);

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled();
      });

      // Should not crash or show error - just silent fail
      expect(mockToast.error).not.toHaveBeenCalled();
    });

    it('handles network error during session check', async () => {
      mockUseAccount.mockReturnValue({ isConnected: true } as any);
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isConnected: true,
      });

      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled();
      });

      // Should handle error silently
      expect(mockToast.error).not.toHaveBeenCalled();
    });
  });

  describe('Context Value', () => {
    it('provides correct context values', async () => {
      const mockState = {
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

      mockUseWeb3Auth.mockReturnValue(mockState);

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('is-connected')).toHaveTextContent('true');
        expect(screen.getByTestId('is-authenticated')).toHaveTextContent('true');
        expect(screen.getByTestId('wallet-address')).toHaveTextContent('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6');
        expect(screen.getByTestId('user-tier')).toHaveTextContent('nft');
        expect(screen.getByTestId('permissions-count')).toHaveTextContent('2');
      });
    });

    it('updates context when web3Auth state changes', async () => {
      let mockState = {
        ...defaultWeb3AuthState,
        isConnected: false,
      };

      mockUseWeb3Auth.mockReturnValue(mockState);

      const { rerender } = render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('is-connected')).toHaveTextContent('false');
      });

      // Update state
      mockState = {
        ...defaultWeb3AuthState,
        isConnected: true,
        isAuthenticated: true,
        walletAddress: '0x123',
      };

      mockUseWeb3Auth.mockReturnValue(mockState);

      rerender(
        <TestWrapper>
          <Web3AuthProvider>
            <TestConsumer />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('is-connected')).toHaveTextContent('true');
        expect(screen.getByTestId('is-authenticated')).toHaveTextContent('true');
        expect(screen.getByTestId('wallet-address')).toHaveTextContent('0x123');
      });
    });
  });

  describe('useWeb3AuthContext Hook', () => {
    it('throws error when used outside provider', () => {
      const TestComponent = () => {
        useWeb3AuthContext();
        return <div>Test</div>;
      };

      // Capture console.error to avoid test output noise
      const originalError = console.error;
      console.error = jest.fn();

      expect(() => {
        render(<TestComponent />);
      }).toThrow('useWeb3AuthContext must be used within Web3AuthProvider');

      console.error = originalError;
    });

    it('returns context when used within provider', () => {
      const TestComponent = () => {
        const context = useWeb3AuthContext();
        return <div data-testid="context-exists">{context ? 'exists' : 'null'}</div>;
      };

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <TestComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      expect(screen.getByTestId('context-exists')).toHaveTextContent('exists');
    });
  });
});

describe('withWeb3Auth HOC', () => {
  const TestComponent = () => <div data-testid="protected-content">Protected Content</div>;

  beforeEach(() => {
    jest.clearAllMocks();
    mockUseAccount.mockReturnValue({ isConnected: false } as any);
  });

  describe('Loading States', () => {
    it('shows loading state during initialization', () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
      });

      const WrappedComponent = withWeb3Auth(TestComponent);

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      expect(screen.getByText('Initializing Web3 authentication...')).toBeInTheDocument();
    });

    it('shows protected content after initialization without auth requirement', async () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { requireAuth: false });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });
  });

  describe('Authentication Requirements', () => {
    it('shows auth required message when not authenticated', async () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { requireAuth: true });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('Web3 Authentication Required')).toBeInTheDocument();
        expect(screen.getByText('Please connect your wallet and sign in to access this feature.')).toBeInTheDocument();
      });
    });

    it('shows protected content when authenticated', async () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isAuthenticated: true,
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { requireAuth: true });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });

    it('shows fallback component when provided and not authenticated', async () => {
      const FallbackComponent = () => <div data-testid="fallback">Custom Fallback</div>;

      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { 
        requireAuth: true,
        fallbackComponent: FallbackComponent 
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('fallback')).toBeInTheDocument();
      });
    });
  });

  describe('Tier Requirements', () => {
    it('shows tier required message when user tier is insufficient', async () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isAuthenticated: true,
        userTier: 'nft',
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { 
        requireAuth: true,
        requireTier: 'enterprise'
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('Higher Tier Required')).toBeInTheDocument();
        expect(screen.getByText('This feature requires enterprise tier access or higher. Your current tier: nft.')).toBeInTheDocument();
      });
    });

    it('shows protected content when user tier meets requirement', async () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isAuthenticated: true,
        userTier: 'enterprise',
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { 
        requireAuth: true,
        requireTier: 'token'
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });

    it('allows equal tier access', async () => {
      mockUseWeb3Auth.mockReturnValue({
        ...defaultWeb3AuthState,
        isAuthenticated: true,
        userTier: 'dao',
      });

      const WrappedComponent = withWeb3Auth(TestComponent, { 
        requireAuth: true,
        requireTier: 'dao'
      });

      render(
        <TestWrapper>
          <Web3AuthProvider>
            <WrappedComponent />
          </Web3AuthProvider>
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });
  });
});

describe('useWeb3Permission Hook', () => {
  const TestComponent = ({ permission }: { permission: string }) => {
    const hasPermission = useWeb3Permission(permission);
    return <div data-testid="has-permission">{hasPermission.toString()}</div>;
  };

  beforeEach(() => {
    mockUseAccount.mockReturnValue({ isConnected: false } as any);
  });

  it('returns false when not authenticated', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: false,
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent permission="admin:users:view" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-permission')).toHaveTextContent('false');
    });
  });

  it('returns true for exact permission match', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      permissions: [
        { permission: 'admin:users:view', source: 'manual' },
      ],
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent permission="admin:users:view" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-permission')).toHaveTextContent('true');
    });
  });

  it('returns true for wildcard permission match', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      permissions: [
        { permission: 'admin:users:*', source: 'manual' },
      ],
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent permission="admin:users:view" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-permission')).toHaveTextContent('true');
    });
  });

  it('returns true for prefix permission match', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      permissions: [
        { permission: 'admin:*', source: 'manual' },
      ],
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent permission="admin:users:view" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-permission')).toHaveTextContent('true');
    });
  });

  it('returns false for non-matching permission', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      permissions: [
        { permission: 'admin:users:view', source: 'manual' },
      ],
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent permission="admin:settings:manage" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-permission')).toHaveTextContent('false');
    });
  });
});

describe('useWeb3Tier Hook', () => {
  const TestComponent = ({ requiredTier }: { requiredTier: 'nft' | 'token' | 'dao' | 'enterprise' }) => {
    const hasTier = useWeb3Tier(requiredTier);
    return <div data-testid="has-tier">{hasTier.toString()}</div>;
  };

  beforeEach(() => {
    mockUseAccount.mockReturnValue({ isConnected: false } as any);
  });

  it('returns false when not authenticated', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: false,
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent requiredTier="nft" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-tier')).toHaveTextContent('false');
    });
  });

  it('returns true when user tier meets requirement', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      userTier: 'enterprise',
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent requiredTier="token" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-tier')).toHaveTextContent('true');
    });
  });

  it('returns false when user tier is insufficient', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      userTier: 'nft',
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent requiredTier="enterprise" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-tier')).toHaveTextContent('false');
    });
  });

  it('returns true for exact tier match', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      userTier: 'dao',
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent requiredTier="dao" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-tier')).toHaveTextContent('true');
    });
  });

  it('handles unknown user tier gracefully', async () => {
    mockUseWeb3Auth.mockReturnValue({
      ...defaultWeb3AuthState,
      isAuthenticated: true,
      userTier: 'unknown' as any,
    });

    render(
      <TestWrapper>
        <Web3AuthProvider>
          <TestComponent requiredTier="nft" />
        </Web3AuthProvider>
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByTestId('has-tier')).toHaveTextContent('false');
    });
  });
});