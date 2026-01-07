import { test, expect, Page, BrowserContext } from '@playwright/test';

// Test configuration
const BASE_URL = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000';
const API_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';

// Comprehensive test wallets for different scenarios
const TEST_WALLETS = {
  unregistered: '0x1234567890123456789012345678901234567890',
  registered: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
  nftHolder: '0xabcdef0123456789012345678901234567890123',
  tokenHolder: '0xfedcba9876543210987654321098765432109876',
  daoMember: '0x555444333222111000999888777666555444333',
  enterprise: '0x999888777666555444333222111000999888777',
  premium: '0x111222333444555666777888999000111222333',
  multiPermission: '0x888777666555444333222111000999888777666',
  expired: '0x666555444333222111000999888777666555444'
};

// Network configurations
const NETWORKS = {
  bsc_mainnet: { chainId: '0x38', name: 'BSC Mainnet' },
  bsc_testnet: { chainId: '0x61', name: 'BSC Testnet' },
  ethereum: { chainId: '0x1', name: 'Ethereum Mainnet' }
};

// Mock wallet connection with comprehensive scenarios
async function mockWalletConnection(page: Page, walletAddress: string, options: {
  rejectConnection?: boolean;
  rejectSigning?: boolean;
  networkChainId?: string;
  walletType?: 'metamask' | 'walletconnect' | 'coinbase';
} = {}) {
  const { rejectConnection, rejectSigning, networkChainId = '0x38', walletType = 'metamask' } = options;

  await page.addInitScript((params) => {
    const { address, rejectConn, rejectSign, chainId, type } = params;
    
    // Mock ethereum provider based on wallet type
    const provider = {
      isMetaMask: type === 'metamask',
      isCoinbaseWallet: type === 'coinbase',
      request: async ({ method, params: methodParams }: any) => {
        switch (method) {
          case 'eth_requestAccounts':
            if (rejectConn) {
              throw new Error('User rejected the request');
            }
            return [address];
          case 'eth_accounts':
            return [address];
          case 'eth_chainId':
            return chainId;
          case 'wallet_switchEthereumChain':
            // Simulate network switching
            return null;
          case 'wallet_addEthereumChain':
            // Simulate adding BSC network
            return null;
          case 'personal_sign':
            if (rejectSign) {
              throw new Error('User rejected signing');
            }
            // Generate realistic mock signature
            return '0x' + 'a'.repeat(130);
          case 'eth_getBalance':
            return '0x1bc16d674ec80000'; // 2 ETH/BNB
          case 'eth_getTransactionCount':
            return '0x1'; // 1 transaction
          default:
            throw new Error(`Unsupported method: ${method}`);
        }
      },
      on: (event: string, handler: Function) => {
        // Store event handlers
        if (!window.__ethEventHandlers) window.__ethEventHandlers = {};
        window.__ethEventHandlers[event] = handler;
      },
      removeListener: () => {},
      removeAllListeners: () => {},
    };

    // Set up ethereum provider
    (window as any).ethereum = provider;
    
    // Mock wagmi/rainbowkit connection
    (window as any).__MOCK_WALLET_CONNECTED__ = !rejectConn;
    (window as any).__MOCK_WALLET_ADDRESS__ = address;
    (window as any).__MOCK_WALLET_TYPE__ = type;
    (window as any).__MOCK_CHAIN_ID__ = chainId;
  }, { 
    address: walletAddress, 
    rejectConn: rejectConnection, 
    rejectSign: rejectSigning, 
    chainId: networkChainId,
    type: walletType 
  });
}

// Mock API responses with comprehensive data
async function mockApiResponse(page: Page, route: string, response: any, status = 200) {
  await page.route(`${API_URL}${route}`, async (route) => {
    await route.fulfill({
      status,
      contentType: 'application/json',
      body: JSON.stringify(response),
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type, Authorization',
      }
    });
  });
}

// Mock session with authentication tokens
async function mockAuthenticatedSession(context: BrowserContext, walletAddress: string) {
  await context.addCookies([
    {
      name: 'access_token',
      value: `mock_access_token_${walletAddress}`,
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false,
      sameSite: 'Lax'
    },
    {
      name: 'id_token',
      value: `mock_id_token_${walletAddress}`,
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false,
      sameSite: 'Lax'
    },
    {
      name: 'refresh_token',
      value: `mock_refresh_token_${walletAddress}`,
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false,
      sameSite: 'Lax'
    }
  ]);
}

test.describe('Web3 Wallet E2E - Complete Coverage', () => {
  test.beforeEach(async ({ page }) => {
    // Mock essential API endpoints
    await mockApiResponse(page, '/health', { status: 'ok' });
    await mockApiResponse(page, '/api/auth/session', { error: 'No active session' }, 401);
    
    // Mock CORS preflight
    await page.route(`${API_URL}/**`, async (route) => {
      if (route.request().method() === 'OPTIONS') {
        await route.fulfill({
          status: 200,
          headers: {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type, Authorization, next-router-prefetch',
          }
        });
      } else {
        await route.continue();
      }
    });
  });

  test.describe('Wallet Connection Flows', () => {
    test('should connect MetaMask wallet successfully', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered, { walletType: 'metamask' });
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'test_nonce_metamask_12345',
        message: `epsx.io wants you to sign in with your Ethereum account`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/auth`);
      
      // Should show auth page
      await expect(page.getByText('Connect to EPSX')).toBeVisible();
      
      // Connect MetaMask
      await page.getByText('Connect Web3 Wallet').click();
      
      // Should show connected wallet
      await expect(page.getByText('0x1234...7890')).toBeVisible();
      await expect(page.getByText('Sign In with Wallet')).toBeVisible();
    });

    test('should connect WalletConnect successfully', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered, { walletType: 'walletconnect' });
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0x742d...45c6')).toBeVisible();
    });

    test('should connect Coinbase Wallet successfully', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.premium, { walletType: 'coinbase' });
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0x1112...2333')).toBeVisible();
    });

    test('should handle wallet connection rejection', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered, { rejectConnection: true });
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Should remain on connection screen
      await expect(page.getByText('Connect Web3 Wallet')).toBeVisible();
    });

    test('should detect when no wallet is installed', async ({ page }) => {
      // Don't mock ethereum object (wallet not installed)
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Should show installation prompt or error
      await expect(page.locator('text=/install|wallet|metamask/i')).toBeVisible();
    });
  });

  test.describe('Network Switching and Chain Management', () => {
    test('should switch to BSC Mainnet when required', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.tokenHolder, { 
        networkChainId: NETWORKS.ethereum.chainId // Start on Ethereum
      });
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Mock network switch request
      await page.evaluate(() => {
        window.__MOCK_CHAIN_ID__ = '0x38'; // Switch to BSC
      });
      
      await expect(page.getByText('0xfedc...9876')).toBeVisible();
    });

    test('should switch to BSC Testnet in development', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.nftHolder, { 
        networkChainId: NETWORKS.bsc_testnet.chainId 
      });
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0xabcd...0123')).toBeVisible();
    });

    test('should handle network switch rejection', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.enterprise);
      
      // Mock network switch rejection
      await page.addInitScript(() => {
        const originalRequest = (window as any).ethereum?.request;
        if (originalRequest) {
          (window as any).ethereum.request = async ({ method, params }: any) => {
            if (method === 'wallet_switchEthereumChain') {
              throw new Error('User rejected network switch');
            }
            return originalRequest({ method, params });
          };
        }
      });
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Should still show wallet but may show network warning
      await expect(page.getByText('0x9998...8777')).toBeVisible();
    });
  });

  test.describe('SIWE Authentication Flow', () => {
    test('should complete full SIWE authentication for new user', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'siwe_nonce_12345',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.unregistered}\n\nSign in to EPSX - Web3 Trading Platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 56\nNonce: siwe_nonce_12345\nIssued At: ${new Date().toISOString()}`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await mockApiResponse(page, '/api/auth/web3/verify', {
        access_token: 'siwe_access_token_new_user',
        id_token: 'siwe_id_token_new_user',
        refresh_token: 'siwe_refresh_token_new_user',
        user_id: 'new_user_siwe_id',
        wallet_address: TEST_WALLETS.unregistered,
        permissions: ['user:profile:view', 'user:analytics:basic'],
        expires_in: 3600
      });

      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.unregistered,
        permissions: [
          { 
            permission: 'user:profile:view', 
            source: 'manual', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          }
        ],
        user_tier: 'free',
        has_api_access: false,
        automatic_grants: []
      });

      await page.goto(`${BASE_URL}/auth`);
      
      // Connect wallet
      await page.getByText('Connect Web3 Wallet').click();
      await expect(page.getByText('0x1234...7890')).toBeVisible();
      
      // Sign SIWE message
      await page.getByText('Sign In with Wallet').click();
      
      // Should redirect to dashboard
      await expect(page).toHaveURL(/dashboard/);
    });

    test('should complete SIWE authentication for existing user', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'siwe_existing_nonce',
        message: `epsx.io wants you to sign in with your Ethereum account:\n${TEST_WALLETS.registered}`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await mockApiResponse(page, '/api/auth/web3/verify', {
        access_token: 'siwe_access_token_existing',
        id_token: 'siwe_id_token_existing',
        refresh_token: 'siwe_refresh_token_existing',
        user_id: 'existing_user_id',
        wallet_address: TEST_WALLETS.registered,
        permissions: ['user:profile:view', 'user:analytics:advanced', 'user:analytics:access'],
        expires_in: 3600
      });

      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();
      
      await expect(page).toHaveURL(/dashboard/);
    });

    test('should handle SIWE signature rejection', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered, { rejectSigning: true });
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'siwe_reject_nonce',
        message: `epsx.io wants you to sign in`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();
      
      // Should show error and remain on auth page
      await expect(page.getByText(/rejected|error/i)).toBeVisible();
      await expect(page.getByText('Sign In with Wallet')).toBeVisible();
    });

    test('should handle invalid signature verification', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'invalid_sig_nonce',
        message: `epsx.io wants you to sign in`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await mockApiResponse(page, '/api/auth/web3/verify', {
        error: 'Invalid signature',
        message: 'The provided signature could not be verified'
      }, 401);

      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();
      
      await expect(page.getByText(/invalid|signature|error/i)).toBeVisible();
    });
  });

  test.describe('Permission-Based Access Control', () => {
    test('should grant NFT-gated permissions for NFT holders', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.nftHolder);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.nftHolder,
        permissions: [
          { 
            permission: 'nft:holder:access', 
            source: 'nft', 
            granted_at: new Date().toISOString(), 
            is_active: true,
            metadata: {
              nft_collection: '0x123456789',
              token_id: '1234'
            }
          },
          { 
            permission: 'premium:features:access', 
            source: 'nft', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          }
        ],
        user_tier: 'nft',
        has_api_access: true,
        automatic_grants: ['nft:holder:access', 'premium:features:access']
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should show NFT tier badge
      await expect(page.getByText('NFT Tier')).toBeVisible();
      await expect(page.getByText('🎨')).toBeVisible(); // NFT icon
    });

    test('should grant token-gated permissions for token holders', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.tokenHolder);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.tokenHolder,
        permissions: [
          { 
            permission: 'token:holder:access', 
            source: 'token', 
            granted_at: new Date().toISOString(), 
            is_active: true,
            metadata: {
              token_contract: '0xTokenContract',
              required_amount: '1000'
            }
          },
          { 
            permission: 'advanced:analytics:access', 
            source: 'token', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          }
        ],
        user_tier: 'token',
        has_api_access: true,
        automatic_grants: ['token:holder:access', 'advanced:analytics:access']
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should show token tier
      await expect(page.getByText('Token Tier')).toBeVisible();
      await expect(page.getByText('🪙')).toBeVisible(); // Token icon
    });

    test('should grant DAO permissions for DAO members', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.daoMember);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.daoMember,
        permissions: [
          { 
            permission: 'dao:member:access', 
            source: 'dao', 
            granted_at: new Date().toISOString(), 
            is_active: true,
            metadata: {
              dao_name: 'EPSX DAO',
              voting_power: '5000'
            }
          },
          { 
            permission: 'governance:voting:access', 
            source: 'dao', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          }
        ],
        user_tier: 'dao',
        has_api_access: true,
        automatic_grants: ['dao:member:access', 'governance:voting:access']
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should show DAO tier
      await expect(page.getByText('Dao Tier')).toBeVisible();
      await expect(page.getByText('🗳️')).toBeVisible(); // DAO icon
    });

    test('should grant enterprise permissions', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.enterprise);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.enterprise,
        permissions: [
          { 
            permission: 'enterprise:full:access', 
            source: 'manual', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          },
          { 
            permission: 'api:unlimited:access', 
            source: 'manual', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          },
          { 
            permission: 'team:management:access', 
            source: 'manual', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          }
        ],
        user_tier: 'enterprise',
        has_api_access: true,
        automatic_grants: []
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should show enterprise tier
      await expect(page.getByText('Enterprise Tier')).toBeVisible();
      await expect(page.getByText('API Access')).toBeVisible();
    });

    test('should handle multiple permission sources', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.multiPermission);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.multiPermission,
        permissions: [
          { permission: 'nft:holder:access', source: 'nft', granted_at: new Date().toISOString(), is_active: true },
          { permission: 'token:holder:access', source: 'token', granted_at: new Date().toISOString(), is_active: true },
          { permission: 'dao:member:access', source: 'dao', granted_at: new Date().toISOString(), is_active: true },
          { permission: 'manual:premium:access', source: 'manual', granted_at: new Date().toISOString(), is_active: true }
        ],
        user_tier: 'enterprise',
        has_api_access: true,
        automatic_grants: ['nft:holder:access', 'token:holder:access', 'dao:member:access']
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should show all permission icons
      await expect(page.getByText('🎨')).toBeVisible(); // NFT
      await expect(page.getByText('🪙')).toBeVisible(); // Token
      await expect(page.getByText('🗳️')).toBeVisible(); // DAO
      await expect(page.getByText('👤')).toBeVisible(); // Manual
      
      // Should show highest tier
      await expect(page.getByText('Enterprise Tier')).toBeVisible();
    });

    test('should restrict access for basic users', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.unregistered,
        permissions: [
          { 
            permission: 'user:profile:view', 
            source: 'manual', 
            granted_at: new Date().toISOString(), 
            is_active: true 
          }
        ],
        user_tier: 'free',
        has_api_access: false,
        automatic_grants: []
      });

      await page.goto(`${BASE_URL}/premium`);
      
      // Should show upgrade prompt or access restriction
      await expect(page.getByText(/upgrade|premium|access denied|subscription/i)).toBeVisible();
    });

    test('should handle expired permissions', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.expired);
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.expired,
        permissions: [
          { 
            permission: 'premium:access', 
            source: 'manual', 
            granted_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(), // 30 days ago
            expires_at: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(), // Expired yesterday
            is_active: false 
          }
        ],
        user_tier: 'free',
        has_api_access: false,
        automatic_grants: []
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should show expired status or renewal prompt
      await expect(page.getByText(/expired|renew|upgrade/i)).toBeVisible();
    });
  });

  test.describe('Session Management and Persistence', () => {
    test('should restore session on page reload', async ({ page, context }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      await mockAuthenticatedSession(context, TEST_WALLETS.registered);
      
      await mockApiResponse(page, '/api/auth/session', {
        user_id: 'persistent_user_id',
        wallet_address: TEST_WALLETS.registered,
        is_authenticated: true,
        permissions: ['user:profile:view', 'user:analytics:access'],
        expires_at: new Date(Date.now() + 3600 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/dashboard`);
      
      // Should be automatically authenticated
      await expect(page.getByText('0x742d...45c6')).toBeVisible();
      
      // Reload page
      await page.reload();
      
      // Should maintain authentication
      await expect(page.getByText('0x742d...45c6')).toBeVisible();
    });

    test('should handle session expiry gracefully', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      
      await mockApiResponse(page, '/api/auth/session', {
        error: 'Session expired',
        message: 'Your session has expired, please log in again'
      }, 401);

      await page.goto(`${BASE_URL}/dashboard`);
      
      // Should redirect to auth page
      await expect(page).toHaveURL(/auth/);
      await expect(page.getByText('Connect to EPSX')).toBeVisible();
    });

    test('should disconnect and clear session properly', async ({ page, context }) => {
      await mockWalletConnection(page, TEST_WALLETS.registered);
      await mockAuthenticatedSession(context, TEST_WALLETS.registered);
      
      await mockApiResponse(page, '/api/auth/session', {
        user_id: 'disconnect_test_user',
        wallet_address: TEST_WALLETS.registered,
        is_authenticated: true,
        permissions: ['user:profile:view'],
        expires_at: new Date(Date.now() + 3600 * 1000).toISOString()
      });

      await mockApiResponse(page, '/api/auth/logout', { success: true });

      await page.goto(`${BASE_URL}/dashboard`);
      
      // Should be authenticated initially
      await expect(page.getByText('0x742d...45c6')).toBeVisible();
      
      // Disconnect wallet
      await page.getByTitle('Disconnect Wallet').click();
      
      // Should redirect to auth page
      await expect(page).toHaveURL(/auth/);
      await expect(page.getByText('Connect to EPSX')).toBeVisible();
    });

    test('should handle token refresh automatically', async ({ page, context }) => {
      await mockWalletConnection(page, TEST_WALLETS.premium);
      await mockAuthenticatedSession(context, TEST_WALLETS.premium);
      
      // Mock initial session (about to expire)
      await mockApiResponse(page, '/api/auth/session', {
        user_id: 'refresh_test_user',
        wallet_address: TEST_WALLETS.premium,
        is_authenticated: true,
        permissions: ['user:profile:view'],
        expires_at: new Date(Date.now() + 60 * 1000).toISOString() // Expires in 1 minute
      });

      // Mock token refresh endpoint
      await mockApiResponse(page, '/api/auth/refresh', {
        access_token: 'new_access_token',
        expires_in: 3600
      });

      await page.goto(`${BASE_URL}/dashboard`);
      
      // Should remain authenticated after automatic refresh
      await expect(page.getByText('0x1112...2333')).toBeVisible();
    });

    test('should maintain session across tabs', async ({ page, context }) => {
      await mockWalletConnection(page, TEST_WALLETS.tokenHolder);
      await mockAuthenticatedSession(context, TEST_WALLETS.tokenHolder);
      
      await mockApiResponse(page, '/api/auth/session', {
        user_id: 'multi_tab_user',
        wallet_address: TEST_WALLETS.tokenHolder,
        is_authenticated: true,
        permissions: ['user:profile:view', 'user:analytics:access'],
        expires_at: new Date(Date.now() + 3600 * 1000).toISOString()
      });

      // Open first tab
      await page.goto(`${BASE_URL}/dashboard`);
      await expect(page.getByText('0xfedc...9876')).toBeVisible();
      
      // Open second tab
      const secondTab = await context.newPage();
      await secondTab.goto(`${BASE_URL}/profile`);
      
      // Should be authenticated in both tabs
      await expect(secondTab.getByText('0xfedc...9876')).toBeVisible();
    });
  });

  test.describe('Error Handling and Edge Cases', () => {
    test('should handle API service unavailability', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        error: 'Service temporarily unavailable',
        message: 'Authentication service is currently down'
      }, 503);

      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();
      
      await expect(page.getByText(/service|unavailable|temporarily/i)).toBeVisible();
    });

    test('should handle network connectivity issues', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Simulate network failure by not mocking endpoints
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();
      
      await expect(page.getByText(/network|connection|failed|error/i)).toBeVisible();
    });

    test('should handle malformed API responses', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      // Mock malformed response
      await page.route(`${API_URL}/api/auth/web3/challenge`, async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: 'invalid json{',
        });
      });

      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      await page.getByText('Sign In with Wallet').click();
      
      await expect(page.getByText(/error|invalid|failed/i)).toBeVisible();
    });

    test('should handle wallet state changes during authentication', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Simulate wallet disconnection during auth flow
      await page.evaluate(() => {
        if (window.__ethEventHandlers && window.__ethEventHandlers.accountsChanged) {
          window.__ethEventHandlers.accountsChanged([]);
        }
      });
      
      // Should handle disconnection gracefully
      await expect(page.getByText('Connect Web3 Wallet')).toBeVisible();
    });

    test('should handle concurrent authentication attempts', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.unregistered);
      
      await mockApiResponse(page, '/api/auth/web3/challenge', {
        nonce: 'concurrent_nonce',
        message: 'Sign in message',
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      });

      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Click sign in multiple times rapidly
      const signInButton = page.getByText('Sign In with Wallet');
      await Promise.all([
        signInButton.click(),
        signInButton.click(),
        signInButton.click()
      ]);
      
      // Should handle gracefully without errors
      await expect(page.getByText(/signing|authenticating|please wait/i)).toBeVisible();
    });
  });

  test.describe('Mobile and Responsive Testing', () => {
    test('should work on mobile devices', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await mockWalletConnection(page, TEST_WALLETS.nftHolder);
      
      await page.goto(`${BASE_URL}/auth`);
      
      // Should show mobile-optimized wallet connection
      const connectButton = page.getByText('Connect Web3 Wallet');
      await expect(connectButton).toBeVisible();
      
      // Check button is appropriately sized for mobile
      const buttonBox = await connectButton.boundingBox();
      expect(buttonBox?.width).toBeGreaterThan(200);
      expect(buttonBox?.height).toBeGreaterThan(44); // iOS minimum tap target
      
      await connectButton.click();
      await expect(page.getByText('0xabcd...0123')).toBeVisible();
    });

    test('should adapt to tablet viewport', async ({ page }) => {
      await page.setViewportSize({ width: 768, height: 1024 });
      await mockWalletConnection(page, TEST_WALLETS.enterprise);
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      // Should work well on tablet
      await expect(page.getByText('0x9998...8777')).toBeVisible();
    });

    test('should handle landscape orientation', async ({ page }) => {
      await page.setViewportSize({ width: 667, height: 375 });
      await mockWalletConnection(page, TEST_WALLETS.tokenHolder);
      
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0xfedc...9876')).toBeVisible();
    });
  });

  test.describe('Cross-Browser Compatibility', () => {
    test('should work in Chromium browsers', async ({ page, browserName }) => {
      test.skip(browserName !== 'chromium', 'Chromium-specific test');
      
      await mockWalletConnection(page, TEST_WALLETS.premium);
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0x1112...2333')).toBeVisible();
    });

    test('should work in Firefox', async ({ page, browserName }) => {
      test.skip(browserName !== 'firefox', 'Firefox-specific test');
      
      await mockWalletConnection(page, TEST_WALLETS.daoMember);
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0x5554...4333')).toBeVisible();
    });

    test('should work in WebKit/Safari', async ({ page, browserName }) => {
      test.skip(browserName !== 'webkit', 'WebKit-specific test');
      
      await mockWalletConnection(page, TEST_WALLETS.multiPermission);
      await page.goto(`${BASE_URL}/auth`);
      await page.getByText('Connect Web3 Wallet').click();
      
      await expect(page.getByText('0x8887...7666')).toBeVisible();
    });
  });

  test.describe('Performance and Load Testing', () => {
    test('should handle rapid wallet connections', async ({ page }) => {
      const wallets = [
        TEST_WALLETS.unregistered,
        TEST_WALLETS.registered,
        TEST_WALLETS.nftHolder,
        TEST_WALLETS.tokenHolder
      ];
      
      for (const wallet of wallets) {
        await mockWalletConnection(page, wallet);
        await page.goto(`${BASE_URL}/auth`);
        await page.getByText('Connect Web3 Wallet').click();
        
        const expectedText = `${wallet.slice(0, 6)}...${wallet.slice(-4)}`;
        await expect(page.getByText(expectedText)).toBeVisible();
        
        // Disconnect for next iteration
        await page.evaluate(() => {
          if (window.__ethEventHandlers && window.__ethEventHandlers.accountsChanged) {
            window.__ethEventHandlers.accountsChanged([]);
          }
        });
      }
    });

    test('should maintain performance with many permissions', async ({ page }) => {
      await mockWalletConnection(page, TEST_WALLETS.enterprise);
      
      // Mock user with many permissions
      const manyPermissions = Array.from({ length: 50 }, (_, i) => ({
        permission: `permission:${i}:access`,
        source: 'manual' as const,
        granted_at: new Date().toISOString(),
        is_active: true
      }));
      
      await mockApiResponse(page, '/api/auth/web3/permissions', {
        wallet_address: TEST_WALLETS.enterprise,
        permissions: manyPermissions,
        user_tier: 'enterprise',
        has_api_access: true,
        automatic_grants: []
      });

      await page.goto(`${BASE_URL}/profile`);
      
      // Should load and display efficiently
      await expect(page.getByText('Enterprise Tier')).toBeVisible();
      await expect(page.getByText('50')).toBeVisible(); // Permission count
    });
  });
});