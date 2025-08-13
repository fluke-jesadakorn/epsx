// Admin Frontend Lib Tests - Application Layer (Service Layer)
// Tests for permission management service business logic
// Clean Architecture: Application Layer - Service coordination and business workflows

import { PermissionService } from '../permission-service'
import { AdminApiClient } from '../admin-api-client'

// Mock AdminApiClient
jest.mock('../admin-api-client')
const mockAdminApiClient = AdminApiClient as jest.MockedClass<typeof AdminApiClient>

describe('PermissionService', () => {
  let permissionService: PermissionService
  let mockApiClientInstance: jest.Mocked<AdminApiClient>

  beforeEach(() => {
    mockApiClientInstance = {
      fetchUsers: jest.fn(),
      updateUserPermissions: jest.fn(),
      bulkUpdatePermissions: jest.fn(),
      fetchPermissionProfiles: jest.fn(),
      createUser: jest.fn()
    } as any

    mockAdminApiClient.mockImplementation(() => mockApiClientInstance)
    permissionService = new PermissionService()
  })

  describe('Permission Profile Management', () => {
    test('getAvailableProfiles should return cached profiles after first fetch', async () => {
      const mockProfiles = [
        {
          id: 'user-basic-001',
          name: 'Basic User',
          permissions: ['basic-analytics', 'stock-ranking']
        },
        {
          id: 'user-premium-002',
          name: 'Premium User',
          permissions: ['basic-analytics', 'stock-ranking', 'advanced-analytics']
        }
      ]

      mockApiClientInstance.fetchPermissionProfiles.mockResolvedValue(mockProfiles)

      // First call should fetch from API
      const firstResult = await permissionService.getAvailableProfiles()
      expect(firstResult).toEqual(mockProfiles)
      expect(mockApiClientInstance.fetchPermissionProfiles).toHaveBeenCalledTimes(1)

      // Second call should return cached result
      const secondResult = await permissionService.getAvailableProfiles()
      expect(secondResult).toEqual(mockProfiles)
      expect(mockApiClientInstance.fetchPermissionProfiles).toHaveBeenCalledTimes(1)
    })

    test('validatePermissionUpgrade should check upgrade validity', async () => {
      const mockProfiles = [
        {
          id: 'user-basic-001',
          name: 'Basic User',
          tier: 1,
          permissions: ['basic-analytics']
        },
        {
          id: 'user-premium-002',
          name: 'Premium User',
          tier: 2,
          permissions: ['basic-analytics', 'advanced-analytics']
        },
        {
          id: 'admin-full-004',
          name: 'Admin',
          tier: 4,
          permissions: ['*']
        }
      ]

      mockApiClientInstance.fetchPermissionProfiles.mockResolvedValue(mockProfiles)

      // Valid upgrade (basic to premium)
      const validUpgrade = await permissionService.validatePermissionUpgrade(
        'user-basic-001',
        'user-premium-002'
      )
      expect(validUpgrade.isValid).toBe(true)
      expect(validUpgrade.reason).toContain('upgrade')

      // Invalid downgrade (premium to basic)
      const invalidDowngrade = await permissionService.validatePermissionUpgrade(
        'user-premium-002',
        'user-basic-001'
      )
      expect(invalidDowngrade.isValid).toBe(false)
      expect(invalidDowngrade.reason).toContain('downgrade')

      // Same level (no change)
      const sameLevel = await permissionService.validatePermissionUpgrade(
        'user-basic-001',
        'user-basic-001'
      )
      expect(sameLevel.isValid).toBe(false)
      expect(sameLevel.reason).toContain('no change')
    })
  })

  describe('Bulk Permission Operations', () => {
    test('processBulkPermissionUpdate should handle mixed results', async () => {
      const bulkUpdateRequest = {
        userIds: ['123', '456', '789'],
        permissionProfile: 'user-premium-002',
        reason: 'Bulk promotion'
      }

      const mockBulkResponse = {
        success: true,
        updated: 2,
        failed: 1,
        results: [
          { userId: '123', success: true },
          { userId: '456', success: true },
          { userId: '789', success: false, error: 'User not found' }
        ]
      }

      mockApiClientInstance.bulkUpdatePermissions.mockResolvedValue(mockBulkResponse)

      const result = await permissionService.processBulkPermissionUpdate(bulkUpdateRequest)

      expect(result.summary.successful).toBe(2)
      expect(result.summary.failed).toBe(1)
      expect(result.summary.total).toBe(3)
      expect(result.failures).toHaveLength(1)
      expect(result.failures[0]).toEqual({
        userId: '789',
        error: 'User not found'
      })
    })

    test('processBulkPermissionUpdate should validate permission profiles before execution', async () => {
      const mockProfiles = [
        { id: 'user-basic-001', name: 'Basic User' },
        { id: 'user-premium-002', name: 'Premium User' }
      ]

      mockApiClientInstance.fetchPermissionProfiles.mockResolvedValue(mockProfiles)

      const invalidRequest = {
        userIds: ['123'],
        permissionProfile: 'invalid-profile',
        reason: 'Test'
      }

      await expect(
        permissionService.processBulkPermissionUpdate(invalidRequest)
      ).rejects.toThrow('Invalid permission profile: invalid-profile')

      expect(mockApiClientInstance.bulkUpdatePermissions).not.toHaveBeenCalled()
    })

    test('generatePermissionChangeAuditLog should create detailed audit entries', async () => {
      const changes = [
        {
          userId: '123',
          fromProfile: 'user-basic-001',
          toProfile: 'user-premium-002',
          reason: 'Subscription upgrade',
          adminId: 'admin-456',
          timestamp: new Date('2024-08-13T12:00:00Z')
        },
        {
          userId: '789',
          fromProfile: 'user-premium-002',
          toProfile: 'moderator-standard-003',
          reason: 'Staff promotion',
          adminId: 'admin-456',
          timestamp: new Date('2024-08-13T12:01:00Z')
        }
      ]

      const auditLog = await permissionService.generatePermissionChangeAuditLog(changes)

      expect(auditLog).toHaveLength(2)
      expect(auditLog[0]).toEqual({
        id: expect.any(String),
        userId: '123',
        action: 'permission_change',
        details: {
          fromProfile: 'user-basic-001',
          toProfile: 'user-premium-002',
          reason: 'Subscription upgrade'
        },
        performedBy: 'admin-456',
        timestamp: '2024-08-13T12:00:00.000Z'
      })
    })
  })

  describe('Permission Analysis', () => {
    test('analyzePermissionDistribution should calculate user distribution by profile', async () => {
      const mockUsers = {
        users: [
          { permissionProfile: 'user-basic-001' },
          { permissionProfile: 'user-basic-001' },
          { permissionProfile: 'user-premium-002' },
          { permissionProfile: 'user-premium-002' },
          { permissionProfile: 'user-premium-002' },
          { permissionProfile: 'moderator-standard-003' }
        ],
        pagination: { totalUsers: 6 }
      }

      mockApiClientInstance.fetchUsers.mockResolvedValue(mockUsers)

      const distribution = await permissionService.analyzePermissionDistribution()

      expect(distribution).toEqual({
        total: 6,
        byProfile: {
          'user-basic-001': { count: 2, percentage: 33.33 },
          'user-premium-002': { count: 3, percentage: 50.00 },
          'moderator-standard-003': { count: 1, percentage: 16.67 }
        }
      })
    })

    test('identifyPermissionAnomalies should detect unusual permission patterns', async () => {
      const mockUsers = {
        users: [
          { 
            id: '1', 
            permissionProfile: 'user-basic-001',
            lastLoginAt: '2024-01-01T00:00:00Z' // Very old login
          },
          { 
            id: '2', 
            permissionProfile: 'admin-full-004',
            createdAt: '2024-08-13T00:00:00Z' // Brand new admin
          },
          { 
            id: '3', 
            permissionProfile: 'user-premium-002',
            lastLoginAt: null // Never logged in
          }
        ]
      }

      mockApiClientInstance.fetchUsers.mockResolvedValue(mockUsers)

      const anomalies = await permissionService.identifyPermissionAnomalies()

      expect(anomalies.staleHighPermissions).toContainEqual({
        userId: '1',
        profile: 'user-basic-001',
        issue: 'No login activity for over 180 days',
        lastLogin: '2024-01-01T00:00:00Z'
      })

      expect(anomalies.newHighPermissions).toContainEqual({
        userId: '2',
        profile: 'admin-full-004',
        issue: 'High-level permissions granted to new account',
        createdAt: '2024-08-13T00:00:00Z'
      })

      expect(anomalies.neverUsedAccounts).toContainEqual({
        userId: '3',
        profile: 'user-premium-002',
        issue: 'Account created but never used'
      })
    })
  })

  describe('Error Handling', () => {
    test('should handle API errors gracefully', async () => {
      mockApiClientInstance.fetchPermissionProfiles.mockRejectedValue(
        new Error('API temporarily unavailable')
      )

      await expect(
        permissionService.getAvailableProfiles()
      ).rejects.toThrow('API temporarily unavailable')
    })

    test('should handle malformed profile data', async () => {
      mockApiClientInstance.fetchPermissionProfiles.mockResolvedValue([
        { id: 'invalid-profile' } // Missing required fields
      ] as any)

      await expect(
        permissionService.getAvailableProfiles()
      ).rejects.toThrow('Invalid permission profile data received')
    })
  })
})