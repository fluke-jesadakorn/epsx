// Frontend Lib Tests - Application Layer (Service Layer)
// Tests for authentication service business logic
// Clean Architecture: Application Layer - Service coordination and business workflows

import { AuthService } from '../auth-service'
import { TokenManager } from '../token-manager'

// Mock dependencies
jest.mock('../token-manager')
const mockTokenManager = TokenManager as jest.MockedClass<typeof TokenManager>

describe('AuthService', () => {
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

  describe('Login Workflow', () => {
    test('successful login should store tokens and return user data', async () => {
      const mockLoginResponse = {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        user: {
          id: '123',
          email: 'test@example.com',
          permissionProfile: 'user-premium-002'
        }
      }

      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: true,
        json: async () => mockLoginResponse
      } as Response)

      const result = await authService.login('test@example.com', 'password123')

      expect(mockTokenManagerInstance.setTokens).toHaveBeenCalledWith(
        'mock-access-token',
        'mock-refresh-token'
      )
      expect(result).toEqual(mockLoginResponse.user)
    })

    test('failed login should throw authentication error', async () => {
      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: 'Unauthorized'
      } as Response)

      await expect(
        authService.login('wrong@example.com', 'wrongpassword')
      ).rejects.toThrow('Authentication failed: 401')

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

  describe('Permission Checking', () => {
    test('hasPermission should check user permission profile', async () => {
      const mockUser = {
        id: '123',
        email: 'test@example.com',
        permissionProfile: 'user-premium-002'
      }

      authService['cachedUser'] = mockUser
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(false)

      const hasBasicAccess = await authService.hasPermission('basic-analytics')
      const hasAdvancedAccess = await authService.hasPermission('advanced-analytics')

      expect(hasBasicAccess).toBe(true) // Premium has basic access
      expect(hasAdvancedAccess).toBe(true) // Premium has advanced access
    })

    test('hasPermission should return false for invalid permissions', async () => {
      const mockUser = {
        id: '123',
        email: 'test@example.com',
        permissionProfile: 'user-basic-001'
      }

      authService['cachedUser'] = mockUser
      mockTokenManagerInstance.isTokenExpired.mockReturnValue(false)

      const hasAdvancedAccess = await authService.hasPermission('advanced-analytics')

      expect(hasAdvancedAccess).toBe(false) // Basic doesn't have advanced access
    })

    test('hasPermission should return false if no user logged in', async () => {
      authService['cachedUser'] = null

      const hasAccess = await authService.hasPermission('basic-analytics')

      expect(hasAccess).toBe(false)
    })
  })

  describe('Error Handling', () => {
    test('should handle network errors gracefully', async () => {
      global.fetch = jest.fn().mockRejectedValue(new Error('Network error'))

      await expect(
        authService.login('test@example.com', 'password')
      ).rejects.toThrow('Network error')
    })

    test('should handle malformed response data', async () => {
      global.fetch = jest.fn().mockResolvedValueOnce({
        ok: true,
        json: async () => { throw new Error('Invalid JSON') }
      } as Response)

      await expect(
        authService.login('test@example.com', 'password')
      ).rejects.toThrow('Invalid JSON')
    })
  })
})