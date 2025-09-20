import { test, expect, Page, BrowserContext } from '@playwright/test';

// Test configuration
const BASE_URL = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000';
const API_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';

// Mock wallet addresses for testing
const TEST_WALLETS = {
  unregistered: '0x1234567890123456789012345678901234567890',
  registered: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
  nftHolder: '0xabcdef0123456789012345678901234567890123',
  tokenHolder: '0xfedcba9876543210987654321098765432109876',
  enterprise: '0x999888777666555444333222111000999888777'
};

// Helper to mock Web3 wallet connection
async function mockWalletConnection(page: Page, walletAddress: string) {
  // Mock the window.ethereum object and wallet connection
  await page.addInitScript((address) => {
    // Mock ethereum provider
    (window as any).ethereum = {
      isMetaMask: true,
      request: async ({ method, params }: any) => {
        switch (method) {
          case 'eth_requestAccounts':
            return [address];
          case 'eth_accounts':
            return [address];
          case 'eth_chainId':
            return '0x1'; // Ethereum mainnet
          case 'personal_sign':
            // Mock signature - in real test you'd use actual cryptographic signature
            return '0x' + 'a'.repeat(130); // Mock valid signature
          default:
            throw new Error(`Unsupported method: ${method}`);
        }
      },
      on: () => {},
      removeListener: () => {},
    };

    // Mock wagmi/rainbowkit connection
    (window as any).__MOCK_WALLET_CONNECTED__ = true;
    (window as any).__MOCK_WALLET_ADDRESS__ = address;
  }, walletAddress);
}

// Helper to mock API responses
async function mockApiResponse(page: Page, route: string, response: any, status = 200) {
  await page.route(`${API_URL}${route}`, async (route) => {
    await route.fulfill({
      status,
      contentType: 'application/json',
      body: JSON.stringify(response),
    });
  });
}

test.describe('Web3 Authentication Flow E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Mock basic API endpoints that are always called
    await mockApiResponse(page, '/health', { status: 'ok' });
    await mockApiResponse(page, '/api/auth/session', { error: 'No active session' }, 401);
  });

  test.describe('Wallet Connection Flow', () => {
    test('should connect wallet and show wallet address', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Mock challenge generation
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'test_nonce_12345',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.unregistered}\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: test_nonce_12345\nIssued At: ${new Date().toISOString()}`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/auth`);

      // Should show unified auth form
      await expect(page.getByText('Connect to EPSX')).toBeVisible();
      
      // Click connect wallet button
      await page.getByText('Connect Web3 Wallet').click();

      // Should show wallet address after connection
      await expect(page.getByText('0x1234...7890')).toBeVisible();
      
      // Should show sign in button
      await expect(page.getByText('Sign In with Wallet')).toBeVisible();
    });

    test('should handle wallet connection rejection', async ({ page }) => {
      // Mock wallet rejection
      await page.addInitScript(() => {
        (window as any).ethereum = {
          isMetaMask: true,
          request: async ({ method }: any) => {
            if (method === 'eth_requestAccounts') {
              throw new Error('User rejected the request');
            }
            return [];
          },
          on: () => {},
          removeListener: () => {},
        };
      });

      await page.goto(`${BASE_URL}/auth`);
      
      // Click connect wallet button
      await page.getByText('Connect Web3 Wallet').click();

      // Should show error or remain on connection screen
      await expect(page.getByText('Connect Web3 Wallet')).toBeVisible();
    });
  });

  test.describe('SIWE Authentication Flow', () => {
    test('should complete full Web3 authentication flow', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Mock challenge generation
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'test_nonce_12345',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.unregistered}\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: test_nonce_12345\nIssued At: ${new Date().toISOString()}`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      // Mock signature verification
      await mockApiResponse(page, '/api/auth/web3/verify', {
        access_token: 'mock_access_token',
        id_token: 'mock_id_token',
        refresh_token: 'mock_refresh_token',
        user_id: 'test_user_id',
        wallet_address: TEST_WALLETS.unregistered,
        permissions: ['user:profile:view', 'user:trading:access'],
        expires_in: 3600
      });

      // Mock permissions endpoint
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.unregistered,
        permissions: [
          { permission: 'user:profile:view', permission_type: 'manual', granted_at: new Date().toISOString(), is_active: true }
        ],
        automatic_grants: []
      });

      await page.goto(`${BASE_URL}/auth`);

      // Connect wallet
      await page.getByText('Connect Web3 Wallet').click();
      await expect(page.getByText('0x1234...7890')).toBeVisible();

      // Sign in with wallet
      await page.getByText('Sign In with Wallet').click();

      // Should redirect to dashboard after successful authentication
      await expect(page).toHaveURL(/dashboard/);
    });

    test('should handle signature rejection', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Override the personal_sign method to reject
      await page.addInitScript(() => {
        if ((window as any).ethereum) {
          const originalRequest = (window as any).ethereum.request;
          (window as any).ethereum.request = async ({ method, params }: any) => {
            if (method === 'personal_sign') {
              throw new Error('User rejected signing');
            }
            return originalRequest({ method, params });
          };
        }
      });

      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'test_nonce_12345',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.unregistered}\n\nSign in to EPSX trading platform`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/auth`);

      // Connect wallet and attempt to sign
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();

      // Should show error message
      await expect(page.getByText(/rejected|error/i)).toBeVisible();
      await expect(page.getByText('Sign In with Wallet')).toBeVisible();
    });

    test('should handle invalid signature response', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'test_nonce_12345',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.unregistered}\n\nSign in to EPSX trading platform`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      // Mock failed verification
      await mockApiResponse(page, '/api/auth/web3/verify', {
        error: 'Invalid signature'
      }, 401);

      await page.goto(`${BASE_URL}/auth`);

      // Connect wallet and attempt to sign
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();

      // Should show error and remain on auth page
      await expect(page.getByText(/invalid|error/i)).toBeVisible();
    });
  });

  test.describe('Permission-Based Access', () => {
    test('should show NFT-gated content for NFT holders', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.nftHolder);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'test_nonce_nft',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.nftHolder}\n\nSign in to EPSX trading platform`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await mockApiResponse(page, '/api/auth/web3/verify', {
        access_token: 'mock_access_token',
        id_token: 'mock_id_token',
        refresh_token: 'mock_refresh_token',
        user_id: 'nft_user_id',
        wallet_address: TEST_WALLETS.nftHolder,
        permissions: ['nft:holder:access', 'user:profile:view', 'premium:features:access'],
        expires_in: 3600
      });

      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.nftHolder,
        permissions: [
          { permission: 'nft:holder:access', permission_type: 'nft_gated', granted_at: new Date().toISOString(), is_active: true },
          { permission: 'premium:features:access', permission_type: 'nft_gated', granted_at: new Date().toISOString(), is_active: true }
        ],
        automatic_grants: ['nft:holder:access', 'premium:features:access']
      });

      await page.goto(`${BASE_URL}/auth`);

      // Complete authentication flow
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();

      // Navigate to a premium feature page
      await page.goto(`${BASE_URL}/premium`);

      // Should show premium content
      await expect(page.getByText(/premium|nft/i)).toBeVisible();
    });

    test('should show token-gated content for token holders', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.tokenHolder);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.tokenHolder,
        permissions: [
          { permission: 'token:holder:access', permission_type: 'token_gated', granted_at: new Date().toISOString(), is_active: true },
          { permission: 'advanced:trading:access', permission_type: 'token_gated', granted_at: new Date().toISOString(), is_active: true }
        ],
        automatic_grants: ['token:holder:access', 'advanced:trading:access']
      });

      // Mock authentication for token holder
      await mockApiResponse(page, '/api/auth/web3/verify', {
        access_token: 'mock_access_token',
        id_token: 'mock_id_token',
        refresh_token: 'mock_refresh_token',
        user_id: 'token_user_id',
        wallet_address: TEST_WALLETS.tokenHolder,
        permissions: ['token:holder:access', 'advanced:trading:access'],
        expires_in: 3600
      });

      await page.goto(`${BASE_URL}/profile`);

      // Should show token-gated features
      await expect(page.getByText(/advanced|token/i)).toBeVisible();
    });

    test('should restrict access for basic users', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.unregistered,
        permissions: [
          { permission: 'user:profile:view', permission_type: 'manual', granted_at: new Date().toISOString(), is_active: true }
        ],
        automatic_grants: []
      });

      // Navigate to premium feature page
      await page.goto(`${BASE_URL}/premium`);

      // Should show upgrade or access denied message
      await expect(page.getByText(/upgrade|premium|access denied/i)).toBeVisible();
    });
  });

  test.describe('Session Management', () => {
    test('should restore session on page reload', async ({ page, context }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      
      // Mock existing session
      await mockApiResponse(page, '/api/auth/session', {
        user_id: 'test_user_id',
        wallet_address: TEST_WALLETS.registered,
        is_authenticated: true,
        permissions: ['user:profile:view', 'user:trading:access'],
        expires_at: new Date(Date.now() + 3600 * 1000).toISOString()
      });

      // Set authentication cookies
      await context.addCookies([
        {
          name: 'access_token',
          value: 'mock_access_token',
          domain: 'localhost',
          path: '/',
          httpOnly: true,
          secure: false, // For localhost
          sameSite: 'Lax'
        },
        {
          name: 'id_token',
          value: 'mock_id_token',
          domain: 'localhost',
          path: '/',
          httpOnly: true,
          secure: false,
          sameSite: 'Lax'
        }
      ]);

      await page.goto(`${BASE_URL}/dashboard`);

      // Should be automatically authenticated
      await expect(page.getByText('0x742d...45c6')).toBeVisible();
      
      // Reload page
      await page.reload();

      // Should still be authenticated
      await expect(page.getByText('0x742d...45c6')).toBeVisible();
    });

    test('should handle session expiry', async ({ page, context }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      
      // Mock expired session
      await mockApiResponse(page, '/api/auth/session', {
        error: 'Session expired'
      }, 401);

      await page.goto(`${BASE_URL}/dashboard`);

      // Should redirect to auth page
      await expect(page).toHaveURL(/auth/);
      await expect(page.getByText('Connect to EPSX')).toBeVisible();
    });

    test('should disconnect and clear session', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      
      // Mock authenticated state
      await mockApiResponse(page, '/api/auth/session', {
        user_id: 'test_user_id',
        wallet_address: TEST_WALLETS.registered,
        is_authenticated: true,
        permissions: ['user:profile:view'],
        expires_at: new Date(Date.now() + 3600 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/dashboard`);
      
      // Should be authenticated
      await expect(page.getByText('0x742d...45c6')).toBeVisible();

      // Mock logout endpoint
      await mockApiResponse(page, '/api/auth/logout', { success: true });

      // Click disconnect button
      await page.getByTitle('Disconnect Wallet').click();

      // Should redirect to auth page
      await expect(page).toHaveURL(/auth/);
    });
  });

  test.describe('Wallet Status and Registration', () => {
    test('should check wallet registration status', async ({ page }) => {
      // Mock wallet status check
      await mockApiResponse(page, '/api/auth/web3/status', {
        wallet_address: TEST_WALLETS.unregistered,
        is_registered: false,
        is_available: true,
        user_id: null,
        status: 'available'
      });

      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();

      // Should show unregistered status
      await expect(page.getByText('Sign In with Wallet')).toBeVisible();
    });

    test('should show registered wallet status', async ({ page }) => {
      await mockApiResponse(page, '/api/auth/web3/status', {
        wallet_address: TEST_WALLETS.registered,
        is_registered: true,
        is_available: false,
        user_id: 'existing_user_id',
        status: 'registered'
      });

      await mockWalletConnection(page, TEST_WALLETS.registered);
      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();

      // Should show that wallet is already registered
      await expect(page.getByText('Sign In with Wallet')).toBeVisible();
    });
  });

  test.describe('Email Linking Flow', () => {
    test('should show email linking option for connected wallets', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();

      // Should show email linking section
      await expect(page.getByText('Optional: Link Email Account')).toBeVisible();
    });

    test('should complete email linking flow', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Mock email linking endpoint
      await mockApiResponse(page, '/api/auth/web3/link-wallet', {
        success: true,
        message: 'Wallet linked successfully',
        user_id: 'linked_user_id',
        wallet_address: TEST_WALLETS.unregistered
      });

      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();

      // Fill email linking form
      await page.getByPlaceholder('Enter email address').fill('test@example.com');
      await page.getByText('Link Email').click();

      // Should show success message
      await expect(page.getByText(/successfully linked/i)).toBeVisible();
    });
  });

  test.describe('Error Handling', () => {
    test('should handle API errors gracefully', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Mock API error
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        error: 'Service temporarily unavailable'
      }, 503);

      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();

      // Should show error message
      await expect(page.getByText(/error|unavailable/i)).toBeVisible();
    });

    test('should handle network errors', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Mock network error by not mocking the endpoint
      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();

      // Should show error or retry option
      await expect(page.getByText(/error|retry|failed/i)).toBeVisible();
    });

    test('should handle wallet not installed', async ({ page }) => {
      // Don't mock ethereum object (wallet not installed)
      await page.goto(`${BASE_URL}/auth`);

      await page.getByText('Connect Web3 Wallet').click();

      // Should show wallet installation prompt
      await expect(page.getByText(/install|wallet/i)).toBeVisible();
    });
  });

  test.describe('Mobile Responsiveness', () => {
    test('should work on mobile devices', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });
      
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      await page.goto(`${BASE_URL}/auth`);

      // Should show mobile-optimized layout
      await expect(page.getByText('Connect Web3 Wallet')).toBeVisible();
      
      // Check that elements are properly sized for mobile
      const connectButton = page.getByText('Connect Web3 Wallet');
      const boundingBox = await connectButton.boundingBox();
      
      expect(boundingBox?.width).toBeGreaterThan(200); // Should be large enough to tap
      expect(boundingBox?.height).toBeGreaterThan(40);
    });
  });

  test.describe('Cross-Browser Compatibility', () => {
    test('should work in different browsers', async ({ page, browserName }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await page.goto(`${BASE_URL}/auth`);

      // Should work regardless of browser
      await expect(page.getByText('Connect Web3 Wallet')).toBeVisible();
      
      console.log(`Testing in ${browserName}`);
    });
  });
});