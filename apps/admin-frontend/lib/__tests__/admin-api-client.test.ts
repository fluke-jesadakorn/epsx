// Admin Frontend Lib Tests - Web3 Infrastructure Layer
// Tests for Web3 admin API clients and wallet-based user management
// Clean Architecture: Infrastructure Layer - Tests external integrations with mocked responses

import { 
  fetchUsers, 
  createUser, 
  updateUserPermissions, 
  fetchPermissionProfiles, 
  bulkUpdatePermissions 
} from '../admin-api-client'

// Mock fetch globally
global.fetch = jest.fn()
const mockFetch = fetch as jest.MockedFunction<typeof fetch>

describe('Admin API Client - Web3 Users', () => {
  beforeEach(() => {
    mockFetch.mockClear()
    
    // Mock wallet-based admin session
    Storage.prototype.getItem = jest.fn(() => 'mock-wallet-admin-token')
  })

  describe('Web3 User Management API', () => {
    test('fetchUsers should return paginated Web3 user data', async () => {
      const mockUsersResponse = {
        users: [
          {
            id: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
            wallet_address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6',
            email: 'user1@example.com',
            permissions: ['epsx:analytics:view', 'epsx:trading:basic'],
            tier: 'nft',
            createdAt: '2024-01-01T00:00:00Z',
            lastLoginAt: '2024-08-01T10:00:00Z'
          },
          {
            id: '0x8f42C5A9F7FaE8D8C3A5e9B2F3c8F9E6A7B8C9D0',
            wallet_address: '0x8f42C5A9F7FaE8D8C3A5e9B2F3c8F9E6A7B8C9D0',
            email: 'user2@example.com',
            permissions: ['epsx:*:*', 'admin:users:view'],
            tier: 'enterprise',
            createdAt: '2024-01-02T00:00:00Z',
            lastLoginAt: '2024-08-02T11:00:00Z'
          }
        ],
        pagination: {
          currentPage: 1,
          totalPages: 10,
          totalUsers: 95,
          pageSize: 10
        }
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockUsersResponse
      } as Response)

      const result = await fetchUsers({ page: 1, pageSize: 10 })
      
      expect(result).toEqual(mockUsersResponse)
      expect(mockFetch).toHaveBeenCalledWith('/api/admin/users?page=1&pageSize=10', {
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer mock-admin-jwt-token'
        }
      })
    })

    test('fetchUsers should handle search and filtering', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ users: [], pagination: {} })
      } as Response)

      await fetchUsers({ 
        page: 1, 
        pageSize: 10, 
        search: 'john@example.com',
        permissionFilter: 'user-premium-002'
      })

      expect(mockFetch).toHaveBeenCalledWith(
        '/api/admin/users?page=1&pageSize=10&search=john%40example.com&permissionProfile=user-premium-002',
        expect.objectContaining({
          headers: expect.objectContaining({
            'Authorization': 'Bearer mock-admin-jwt-token'
          })
        })
      )
    })

    test('createUser should send POST request with user data', async () => {
      const newUserData = {
        email: 'newuser@example.com',
        permissionProfile: 'user-basic-001',
        sendWelcomeEmail: true
      }

      const mockResponse = {
        id: '789',
        ...newUserData,
        createdAt: '2024-08-13T12:00:00Z'
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse
      } as Response)

      const result = await createUser(newUserData)

      expect(result).toEqual(mockResponse)
      expect(mockFetch).toHaveBeenCalledWith('/api/admin/users', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer mock-admin-jwt-token'
        },
        body: JSON.stringify(newUserData)
      })
    })
  })

  describe('Permission Management API', () => {
    test('updateUserPermissions should update single user permissions', async () => {
      const permissionUpdate = {
        userId: '123',
        permissionProfile: 'user-premium-002',
        reason: 'Subscription upgrade'
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, updated: true })
      } as Response)

      await updateUserPermissions(permissionUpdate)

      expect(mockFetch).toHaveBeenCalledWith('/api/admin/users/123/permissions', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer mock-admin-jwt-token'
        },
        body: JSON.stringify({
          permissionProfile: 'user-premium-002',
          reason: 'Subscription upgrade'
        })
      })
    })

    test('bulkUpdatePermissions should handle multiple users', async () => {
      const bulkUpdate = {
        userIds: ['123', '456', '789'],
        permissionProfile: 'user-premium-002',
        reason: 'Bulk upgrade promotion'
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ 
          success: true, 
          updated: 3,
          failed: 0,
          results: []
        })
      } as Response)

      const result = await bulkUpdatePermissions(bulkUpdate)

      expect(result.updated).toBe(3)
      expect(mockFetch).toHaveBeenCalledWith('/api/admin/users/permissions/bulk', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer mock-admin-jwt-token'
        },
        body: JSON.stringify(bulkUpdate)
      })
    })

    test('fetchPermissionProfiles should return available profiles', async () => {
      const mockProfiles = [
        {
          id: 'user-basic-001',
          name: 'Basic User',
          description: 'Basic trading features',
          permissions: ['basic-analytics', 'stock-ranking']
        },
        {
          id: 'user-premium-002',
          name: 'Premium User',
          description: 'Premium features + advanced analytics',
          permissions: ['basic-analytics', 'stock-ranking', 'advanced-analytics', 'export-data']
        }
      ]

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockProfiles
      } as Response)

      const result = await fetchPermissionProfiles()

      expect(result).toEqual(mockProfiles)
      expect(mockFetch).toHaveBeenCalledWith('/api/admin/permission-profiles')
    })
  })

  describe('Error Handling', () => {
    test('should handle 403 Forbidden errors', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 403,
        statusText: 'Forbidden'
      } as Response)

      await expect(fetchUsers({ page: 1 })).rejects.toThrow('Admin access required: 403')
    })

    test('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      await expect(fetchUsers({ page: 1 })).rejects.toThrow('Network error')
    })

    test('should handle rate limiting', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 429,
        statusText: 'Too Many Requests'
      } as Response)

      await expect(fetchUsers({ page: 1 })).rejects.toThrow('Rate limited: 429')
    })
  })

  describe('Authentication', () => {
    test('should include admin authentication headers', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ users: [], pagination: {} })
      } as Response)

      await fetchUsers({ page: 1 })

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            'Authorization': 'Bearer mock-admin-jwt-token',
            'Content-Type': 'application/json'
          })
        })
      )
    })

    test('should handle missing admin token', async () => {
      Storage.prototype.getItem = jest.fn(() => null)

      await expect(fetchUsers({ page: 1 })).rejects.toThrow('Admin authentication required')
    })
  })

  describe('Pagination and Filtering', () => {
    test('should build correct query parameters for complex filters', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ users: [], pagination: {} })
      } as Response)

      await fetchUsers({
        page: 3,
        pageSize: 25,
        search: 'premium users',
        permissionFilter: 'user-premium-002',
        sortBy: 'createdAt',
        sortOrder: 'desc',
        dateFrom: '2024-01-01',
        dateTo: '2024-08-01'
      })

      const expectedUrl = '/api/admin/users?page=3&pageSize=25&search=premium%20users&permissionProfile=user-premium-002&sortBy=createdAt&sortOrder=desc&dateFrom=2024-01-01&dateTo=2024-08-01'
      
      expect(mockFetch).toHaveBeenCalledWith(
        expectedUrl,
        expect.any(Object)
      )
    })
  })
})