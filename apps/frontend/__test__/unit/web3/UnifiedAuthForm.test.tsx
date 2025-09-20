import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WagmiProvider } from 'wagmi';
import { RainbowKitProvider } from '@rainbow-me/rainbowkit';
import { UnifiedAuthForm } from '@/components/auth/UnifiedAuthForm';
import { useAccount } from 'wagmi';
import { config } from '@/lib/wagmi';

// Mock dependencies
jest.mock('wagmi');
jest.mock('@/components/auth/WalletConnectAuth');
jest.mock('@/components/auth/EmailLinking');
jest.mock('@/lib/wagmi');

const mockUseAccount = useAccount as jest.MockedFunction<typeof useAccount>;

// Mock fetch for OAuth flow
global.fetch = jest.fn();
const mockFetch = fetch as jest.MockedFunction<typeof fetch>;

// Mock window location
Object.defineProperty(window, 'location', {
  value: {
    href: '',
  },
  writable: true,
});

// Mock components
jest.mock('@/components/auth/WalletConnectAuth', () => ({
  WalletConnectAuth: ({ onAuthSuccess, onAuthError, variant }: any) => (
    <div data-testid="wallet-connect-auth">
      <div>Variant: {variant}</div>
      <button onClick={() => onAuthSuccess?.('0x123')}>Mock Auth Success</button>
      <button onClick={() => onAuthError?.('Mock error')}>Mock Auth Error</button>
    </div>
  ),
}));

jest.mock('@/components/auth/EmailLinking', () => ({
  EmailLinking: ({ showAsDialog, autoShow }: any) => (
    <div data-testid="email-linking">
      Email Linking - Dialog: {showAsDialog?.toString()}, Auto: {autoShow?.toString()}
    </div>
  ),
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

describe('UnifiedAuthForm', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseAccount.mockReturnValue({
      isConnected: false,
      address: undefined,
    } as any);
    window.location.href = '';
  });

  describe('Initial Render', () => {
    it('renders with default web3-first configuration', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      expect(screen.getByText('Connect to EPSX')).toBeInTheDocument();
      expect(screen.getByText('Connect your Web3 wallet for enhanced features and permissions')).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /web3 wallet/i })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /email\/password/i })).toBeInTheDocument();
    });

    it('renders with traditional auth title when web3First is false', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm web3First={false} />
        </TestWrapper>
      );

      expect(screen.getByText('Sign In to EPSX')).toBeInTheDocument();
      expect(screen.getByText('Choose your preferred authentication method')).toBeInTheDocument();
    });

    it('starts with correct default tab', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm defaultTab="oidc" />
        </TestWrapper>
      );

      // OIDC tab should be active
      expect(screen.getByText('Traditional Authentication')).toBeInTheDocument();
    });
  });

  describe('Tab Navigation', () => {
    it('switches between tabs correctly', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      // Should start on Web3 tab
      expect(screen.getByText('Web3-First Authentication')).toBeInTheDocument();

      // Click OIDC tab
      fireEvent.click(screen.getByRole('tab', { name: /email\/password/i }));
      expect(screen.getByText('Traditional Authentication')).toBeInTheDocument();

      // Click Web3 tab
      fireEvent.click(screen.getByRole('tab', { name: /web3 wallet/i }));
      expect(screen.getByText('Web3-First Authentication')).toBeInTheDocument();
    });

    it('shows correct tab labels based on web3First prop', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm web3First={false} />
        </TestWrapper>
      );

      expect(screen.getByRole('tab', { name: /web3 wallet/i })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /traditional/i })).toBeInTheDocument();
    });
  });

  describe('Web3 Tab Content', () => {
    it('renders Web3 hero section with benefits', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      expect(screen.getByText('Web3-First Authentication')).toBeInTheDocument();
      expect(screen.getByText('Connect your crypto wallet for instant access to Web3 features, NFT-gated content, token-based permissions, and DAO governance.')).toBeInTheDocument();
      
      // Check supported wallets
      expect(screen.getByText('MetaMask')).toBeInTheDocument();
      expect(screen.getByText('WalletConnect')).toBeInTheDocument();
      expect(screen.getByText('Coinbase')).toBeInTheDocument();

      // Check Web3 benefits
      expect(screen.getByText('NFT Access')).toBeInTheDocument();
      expect(screen.getByText('Token Gates')).toBeInTheDocument();
      expect(screen.getByText('DAO Voting')).toBeInTheDocument();
    });

    it('renders WalletConnectAuth component with detailed variant', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      const walletAuth = screen.getByTestId('wallet-connect-auth');
      expect(walletAuth).toBeInTheDocument();
      expect(screen.getByText('Variant: detailed')).toBeInTheDocument();
    });

    it('shows email linking when wallet is connected and showEmailLinking is true', () => {
      mockUseAccount.mockReturnValue({
        isConnected: true,
        address: '0x123',
      } as any);

      render(
        <TestWrapper>
          <UnifiedAuthForm showEmailLinking={true} />
        </TestWrapper>
      );

      expect(screen.getByText('Optional: Link Email Account')).toBeInTheDocument();
      expect(screen.getByTestId('email-linking')).toBeInTheDocument();
    });

    it('hides email linking when showEmailLinking is false', () => {
      mockUseAccount.mockReturnValue({
        isConnected: true,
        address: '0x123',
      } as any);

      render(
        <TestWrapper>
          <UnifiedAuthForm showEmailLinking={false} />
        </TestWrapper>
      );

      expect(screen.queryByText('Optional: Link Email Account')).not.toBeInTheDocument();
    });

    it('renders security features section', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      expect(screen.getByText('Self-Sovereign')).toBeInTheDocument();
      expect(screen.getByText('Your keys, your control')).toBeInTheDocument();
      expect(screen.getByText('SIWE Standard')).toBeInTheDocument();
      expect(screen.getByText('Ethereum-native auth')).toBeInTheDocument();
    });
  });

  describe('OIDC Tab Content', () => {
    beforeEach(() => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );
      
      // Switch to OIDC tab
      fireEvent.click(screen.getByRole('tab', { name: /email\/password/i }));
    });

    it('renders traditional authentication notice', () => {
      expect(screen.getByText('Traditional Authentication')).toBeInTheDocument();
      expect(screen.getByText('Use this option if you don\'t have a Web3 wallet or prefer traditional email/password authentication.')).toBeInTheDocument();
    });

    it('renders sign in button', () => {
      expect(screen.getByText('Sign In with Email')).toBeInTheDocument();
    });

    it('renders OIDC features', () => {
      expect(screen.getByText('Secure OAuth')).toBeInTheDocument();
      expect(screen.getByText('OIDC Compliant')).toBeInTheDocument();
    });

    it('renders security notice', () => {
      expect(screen.getByText('🔒 Standard email/password authentication')).toBeInTheDocument();
      expect(screen.getByText('HttpOnly cookies • PKCE protection • Bearer tokens')).toBeInTheDocument();
    });
  });

  describe('OAuth Flow', () => {
    beforeEach(() => {
      render(
        <TestWrapper>
          <UnifiedAuthForm redirectTo="/custom-redirect" />
        </TestWrapper>
      );
      
      // Switch to OIDC tab
      fireEvent.click(screen.getByRole('tab', { name: /email\/password/i }));
    });

    it('initiates OAuth flow successfully', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          authorizationUrl: 'https://auth.epsx.io/oauth/authorize?code=123'
        }),
      } as Response);

      fireEvent.click(screen.getByText('Sign In with Email'));

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith('/api/auth/initiate', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            redirectTo: '/custom-redirect'
          }),
          credentials: 'include'
        });
      });

      expect(window.location.href).toBe('https://auth.epsx.io/oauth/authorize?code=123');
    });

    it('shows loading state during OAuth initiation', async () => {
      mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves

      fireEvent.click(screen.getByText('Sign In with Email'));

      await waitFor(() => {
        expect(screen.getByText('Redirecting to secure authentication...')).toBeInTheDocument();
      });
    });

    it('handles OAuth initiation error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({
          message: 'OAuth service unavailable'
        }),
      } as Response);

      fireEvent.click(screen.getByText('Sign In with Email'));

      await waitFor(() => {
        expect(screen.getByText('⚠️ OAuth service unavailable')).toBeInTheDocument();
      });
    });

    it('handles network error during OAuth initiation', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      fireEvent.click(screen.getByText('Sign In with Email'));

      await waitFor(() => {
        expect(screen.getByText('⚠️ Network error')).toBeInTheDocument();
      });
    });

    it('disables button during loading', async () => {
      mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves

      const button = screen.getByText('Sign In with Email');
      fireEvent.click(button);

      await waitFor(() => {
        expect(button).toBeDisabled();
      });
    });
  });

  describe('Web3 Authentication Callbacks', () => {
    it('handles successful Web3 authentication', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm redirectTo="/dashboard" />
        </TestWrapper>
      );

      fireEvent.click(screen.getByText('Mock Auth Success'));

      expect(window.location.href).toBe('/dashboard');
    });

    it('handles Web3 authentication error', () => {
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation();

      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      fireEvent.click(screen.getByText('Mock Auth Error'));

      expect(consoleSpy).toHaveBeenCalledWith('Web3 authentication error:', 'Mock error');
      
      consoleSpy.mockRestore();
    });
  });

  describe('Platform Notice', () => {
    it('shows web3-first notice when web3First is true', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm web3First={true} />
        </TestWrapper>
      );

      expect(screen.getByText('Web3-First Platform')).toBeInTheDocument();
      expect(screen.getByText('EPSX prioritizes Web3 authentication for enhanced features, permissions, and decentralized access. Traditional auth available as fallback.')).toBeInTheDocument();
    });

    it('shows hybrid notice when web3First is false', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm web3First={false} />
        </TestWrapper>
      );

      expect(screen.getByText('Hybrid Authentication System')).toBeInTheDocument();
      expect(screen.getByText('Both methods are OIDC-compliant and provide secure access to your EPSX account. Choose the method that best fits your preferences.')).toBeInTheDocument();
    });
  });

  describe('Props and Configuration', () => {
    it('uses custom redirectTo prop', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          authorizationUrl: 'https://auth.epsx.io/oauth/authorize'
        }),
      } as Response);

      render(
        <TestWrapper>
          <UnifiedAuthForm redirectTo="/custom-dashboard" />
        </TestWrapper>
      );

      fireEvent.click(screen.getByRole('tab', { name: /email\/password/i }));
      fireEvent.click(screen.getByText('Sign In with Email'));

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith('/api/auth/initiate', 
          expect.objectContaining({
            body: JSON.stringify({
              redirectTo: '/custom-dashboard'
            }),
          })
        );
      });
    });

    it('starts with specified defaultTab', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm defaultTab="oidc" />
        </TestWrapper>
      );

      expect(screen.getByText('Traditional Authentication')).toBeInTheDocument();
    });

    it('respects showEmailLinking prop', () => {
      mockUseAccount.mockReturnValue({
        isConnected: true,
        address: '0x123',
      } as any);

      const { rerender } = render(
        <TestWrapper>
          <UnifiedAuthForm showEmailLinking={false} />
        </TestWrapper>
      );

      expect(screen.queryByText('Optional: Link Email Account')).not.toBeInTheDocument();

      rerender(
        <TestWrapper>
          <UnifiedAuthForm showEmailLinking={true} />
        </TestWrapper>
      );

      expect(screen.getByText('Optional: Link Email Account')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels and roles', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      expect(screen.getByRole('tablist')).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /web3 wallet/i })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /email\/password/i })).toBeInTheDocument();
      expect(screen.getByRole('tabpanel')).toBeInTheDocument();
    });

    it('maintains focus management between tabs', () => {
      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      const oidcTab = screen.getByRole('tab', { name: /email\/password/i });
      const web3Tab = screen.getByRole('tab', { name: /web3 wallet/i });

      fireEvent.click(oidcTab);
      expect(oidcTab).toHaveAttribute('aria-selected', 'true');
      expect(web3Tab).toHaveAttribute('aria-selected', 'false');

      fireEvent.click(web3Tab);
      expect(web3Tab).toHaveAttribute('aria-selected', 'true');
      expect(oidcTab).toHaveAttribute('aria-selected', 'false');
    });
  });

  describe('Edge Cases', () => {
    it('handles OAuth response without authorizationUrl', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      } as Response);

      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      fireEvent.click(screen.getByRole('tab', { name: /email\/password/i }));
      fireEvent.click(screen.getByText('Sign In with Email'));

      // Should handle gracefully without crashing
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled();
      });
    });

    it('handles JSON parsing error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => { throw new Error('Invalid JSON'); },
      } as Response);

      render(
        <TestWrapper>
          <UnifiedAuthForm />
        </TestWrapper>
      );

      fireEvent.click(screen.getByRole('tab', { name: /email\/password/i }));
      fireEvent.click(screen.getByText('Sign In with Email'));

      await waitFor(() => {
        expect(screen.getByText(/Authentication failed. Please try again./)).toBeInTheDocument();
      });
    });
  });
});