// Frontend Lib Tests - Web3 Authentication Service
// Tests for Web3 wallet authentication service business logic
// Clean Architecture: Application Layer - Service coordination and business workflows

import { AuthService } from '../auth-service'
import { TokenManager } from '../token-manager'

// Mock dependencies
jest.mock('../token-manager')
const mockTokenManager = TokenManager as jest.MockedClass<typeof TokenManager>

describe('AuthService - Web3 Authentication', () => {
  let authService: AuthService
  let mockTokenManagerInstance: jest.Mocked<TokenManager>

  beforeEach(() => {
    mockTokenManagerInstance = {
      getAccessToken: jest.fn(),
      setTokens: jest.fn(),
      clearTokens: jest.fn(),
      isTokenExpired: jest.fn(),
      refreshAccessToken: jest.fn()
    } as any

    mockTokenManager.mockImplementation(() => mockTokenManagerInstance)
    authService = new AuthService()
  })

  describe('Web3 Wallet Authentication', () => {
    test('successful wallet authentication should store tokens and return user data', async () => {
      const mockWalletAuthResponse = {
        accessToken: 'mock-web3-access-token',
        refreshToken: 'mock-web3-refresh-token',
        user: {
          id: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
          wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
          permissions: ['epsx:analytics:view', 'epsx:trading:execute'],
          tier: 'nft'
        }
      }

      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: true,
        json: async () => mockWalletAuthResponse
      } as Response)

      const result = await authService.walletLogin('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6', 'mock-signature', 'mock-message')

      expect(mockTokenManagerInstance.setTokens).toHaveBeenCalledWith(
        'mock-web3-access-token',
        'mock-web3-refresh-token'
      )
      expect(result).toEqual(mockWalletAuthResponse.user)
    })

    test('failed wallet authentication should throw authentication error', async () => {
      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: 'Invalid Signature'
      } as Response)

      await expect(
        authService.walletLogin('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6', 'invalid-signature', 'mock-message')
      ).rejects.toThrow('Wallet authentication failed: 401')

      expect(mockTokenManagerInstance.setTokens).not.toHaveBeenCalled()
    })
  })

  describe('Logout Workflow', () => {
    test('logout should clear tokens and call logout endpoint', async () => {
      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: true
      } as Response)

      await authService.logout()

      expect(mockTokenManagerInstance.clearTokens).toHaveBeenCalled()
      expect(fetch).toHaveBeenCalledWith('/api/auth/logout', expect.objectContaining({
        method: 'POST'
      }))
    })

    test('logout should clear tokens even if endpoint fails', async () => {
      global.fetch = jest.fn().mockRejectedValueOnce(new Error('Network error'))

      await authService.logout()

      expect(mockTokenManagerInstance.clearTokens).toHaveBeenCalled()
    })
  })

  describe('Session Management', () => {
    test('getCurrentUser should return cached user if token valid', async () => {
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(false)
      mockTokenManagerInstance.getAccessToken.mockReturnValue('valid-token')

      // Mock cached user data
      authService['cachedUser'] = {
        id: '123',
        email: 'test@example.com',
        permissionProfile: 'user-premium-002'
      }

      const result = await authService.getCurrentUser()

      expect(result).toEqual(authService['cachedUser'])
      expect(fetch).not.toHaveBeenCalled()
    })

    test('getCurrentUser should refresh token if expired', async () => {
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(true)
      mockTokenManagerInstance.refreshAccessToken.mockResolvedValue('new-token')

      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          id: '123',
          email: 'test@example.com',
          permissionProfile: 'user-premium-002'
        })
      } as Response)

      await authService.getCurrentUser()

      expect(mockTokenManagerInstance.refreshAccessToken).toHaveBeenCalled()
      expect(fetch).toHaveBeenCalledWith('/api/auth/me', expect.objectContaining({
        headers: expect.objectContaining({
          'Authorization': 'Bearer new-token'
        })
      }))
    })

    test('getCurrentUser should return null if no token available', async () => {
      mockTokenManagerInstance.getAccessToken.mockReturnValue(null)

      const result = await authService.getCurrentUser()

      expect(result).toBeNull()
    })
  })

  describe('Web3 Permission Checking', () => {
    test('hasPermission should check structured Web3 permissions', async () => {
      const mockWeb3User = {
        id: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        permissions: ['epsx:analytics:view', 'epsx:trading:execute', 'admin:users:manage'],
        tier: 'nft'
      }

      authService['cachedUser'] = mockWeb3User
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(false)

      const hasAnalyticsAccess = await authService.hasPermission('epsx:analytics:view')
      const hasTradingAccess = await authService.hasPermission('epsx:trading:execute')
      const hasAdminAccess = await authService.hasPermission('admin:users:manage')

      expect(hasAnalyticsAccess).toBe(true) // NFT user has analytics access
      expect(hasTradingAccess).toBe(true) // NFT user has trading access
      expect(hasAdminAccess).toBe(true) // NFT user has admin access
    })

    test('hasPermission should return false for missing structured permissions', async () => {
      const mockWeb3User = {
        id: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        permissions: ['epsx:analytics:view'], // Only has basic analytics
        tier: 'basic'
      }

      authService['cachedUser'] = mockWeb3User
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(false)

      const hasAdvancedAccess = await authService.hasPermission('epsx:trading:execute')
      const hasAdminAccess = await authService.hasPermission('admin:users:manage')

      expect(hasAdvancedAccess).toBe(false) // Basic user doesn't have trading access
      expect(hasAdminAccess).toBe(false) // Basic user doesn't have admin access
    })

    test('hasPermission should handle wildcard permissions', async () => {
      const mockWeb3User = {
        id: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
        permissions: ['epsx:*:*', 'admin:users:*'], // Wildcard permissions
        tier: 'enterprise'
      }

      authService['cachedUser'] = mockWeb3User
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(false)

      const hasAnalyticsAccess = await authService.hasPermission('epsx:analytics:view')
      const hasTradingAccess = await authService.hasPermission('epsx:trading:execute')
      const hasUserManageAccess = await authService.hasPermission('admin:users:manage')
      const hasUserViewAccess = await authService.hasPermission('admin:users:view')

      expect(hasAnalyticsAccess).toBe(true) // epsx:*:* covers analytics
      expect(hasTradingAccess).toBe(true) // epsx:*:* covers trading
      expect(hasUserManageAccess).toBe(true) // admin:users:* covers manage
      expect(hasUserViewAccess).toBe(true) // admin:users:* covers view
    })

    test('hasPermission should return false if no user logged in', async () => {
      authService['cachedUser'] = null

      const hasAccess = await authService.hasPermission('epsx:analytics:view')

      expect(hasAccess).toBe(false)
    })
  })

  describe('Web3 Error Handling', () => {
    test('should handle network errors gracefully during wallet auth', async () => {
      global.fetch = jest.fn().mockRejectedValue(new Error('Network error'))

      await expect(
        authService.walletLogin('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6', 'signature', 'message')
      ).rejects.toThrow('Network error')
    })

    test('should handle malformed response data from wallet auth', async () => {
      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: true,
        json: async () => { throw new Error('Invalid JSON') }
      } as Response)

      await expect(
        authService.walletLogin('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6', 'signature', 'message')
      ).rejects.toThrow('Invalid JSON')
    })

    test('should handle invalid wallet address format', async () => {
      await expect(
        authService.walletLogin('invalid-address', 'signature', 'message')
      ).rejects.toThrow('Invalid wallet address format')
    })

    test('should handle invalid signature format', async () => {
      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: false,
        status: 400,
        statusText: 'Invalid Signature Format'
      } as Response)

      await expect(
        authService.walletLogin('0x742d35Cc6634C0532925a3b8D369D7763F3c45c6', 'invalid-sig', 'message')
      ).rejects.toThrow('Wallet authentication failed: 400')
    })
  })
})