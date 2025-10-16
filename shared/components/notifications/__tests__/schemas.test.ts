/**
 * Unit Tests for Notification Schemas
 *
 * Tests all Zod validation schemas for notifications including:
 * - NotificationSchema validation
 * - SSENotificationSchema validation
 * - NotificationFiltersSchema validation
 * - SendNotificationRequestSchema validation
 * - Validation helper functions
 */

import {
  NotificationSchema,
  SSENotificationSchema,
  NotificationFiltersSchema,
  SendNotificationRequestSchema,
  validateNotification,
  validateSSENotification,
  validateNotificationFilters,
  validateSendNotificationRequest,
  validateNotifications,
} from '../schemas'

describe('Notification Schemas', () => {
  // ============================================================================
  // NOTIFICATION SCHEMA TESTS
  // ============================================================================

  describe('NotificationSchema', () => {
    const validNotification = {
      id: '123e4567-e89b-12d3-a456-426614174000',
      title: 'Test Notification',
      message: 'This is a test message',
      type: 'system',
      priority: 'normal',
      timestamp: '2025-10-14T12:00:00Z',
      read: false,
    }

    it('should validate a complete valid notification', () => {
      const result = NotificationSchema.safeParse(validNotification)
      expect(result.success).toBe(true)
    })

    it('should validate notification with optional fields', () => {
      const notification = {
        ...validNotification,
        expires_at: '2025-10-15T12:00:00Z',
        read_at: '2025-10-14T13:00:00Z',
        clicked_at: '2025-10-14T13:05:00Z',
        delivered_at: '2025-10-14T12:01:00Z',
        action_url: 'https://example.com/action',
        image_url: 'https://example.com/image.png',
        wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
        data: { key: 'value' },
      }

      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(true)
    })

    it('should reject invalid UUID', () => {
      const notification = { ...validNotification, id: 'not-a-uuid' }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject empty title', () => {
      const notification = { ...validNotification, title: '' }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject title exceeding 200 characters', () => {
      const notification = { ...validNotification, title: 'a'.repeat(201) }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject empty message', () => {
      const notification = { ...validNotification, message: '' }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject message exceeding 1000 characters', () => {
      const notification = { ...validNotification, message: 'a'.repeat(1001) }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject invalid notification type', () => {
      const notification = { ...validNotification, type: 'invalid' }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject invalid priority', () => {
      const notification = { ...validNotification, priority: 'invalid' }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject invalid timestamp format', () => {
      const notification = { ...validNotification, timestamp: 'not-a-date' }
      const result = NotificationSchema.safeParse(notification)
      expect(result.success).toBe(false)
    })

    it('should reject invalid URL formats', () => {
      const notification1 = { ...validNotification, action_url: 'not-a-url' }
      const result1 = NotificationSchema.safeParse(notification1)
      expect(result1.success).toBe(false)

      const notification2 = { ...validNotification, image_url: 'not-a-url' }
      const result2 = NotificationSchema.safeParse(notification2)
      expect(result2.success).toBe(false)
    })
  })

  // ============================================================================
  // SSE NOTIFICATION SCHEMA TESTS
  // ============================================================================

  describe('SSENotificationSchema', () => {
    const validSSENotification = {
      id: '123e4567-e89b-12d3-a456-426614174000',
      wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
      notification_type: 'system',
      title: 'Test Notification',
      message: 'This is a test message',
      priority: 'normal',
      timestamp: '2025-10-14T12:00:00Z',
    }

    it('should validate a complete valid SSE notification', () => {
      const result = SSENotificationSchema.safeParse(validSSENotification)
      expect(result.success).toBe(true)
    })

    it('should validate SSE notification with optional fields', () => {
      const notification = {
        ...validSSENotification,
        data: { key: 'value' },
        expires_at: '2025-10-15T12:00:00Z',
      }

      const result = SSENotificationSchema.safeParse(notification)
      expect(result.success).toBe(true)
    })

    it('should reject missing required fields', () => {
      const { id, ...withoutId } = validSSENotification
      const result = SSENotificationSchema.safeParse(withoutId)
      expect(result.success).toBe(false)
    })
  })

  // ============================================================================
  // NOTIFICATION FILTERS SCHEMA TESTS
  // ============================================================================

  describe('NotificationFiltersSchema', () => {
    it('should validate empty filters', () => {
      const result = NotificationFiltersSchema.safeParse({})
      expect(result.success).toBe(true)
    })

    it('should validate complete filters', () => {
      const filters = {
        page: 1,
        limit: 20,
        type: 'system',
        priority: 'high',
        status: 'unread',
        start_date: '2025-10-01T00:00:00Z',
        end_date: '2025-10-14T23:59:59Z',
        wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
      }

      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(true)
    })

    it('should reject negative page number', () => {
      const filters = { page: -1 }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject zero page number', () => {
      const filters = { page: 0 }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject negative limit', () => {
      const filters = { limit: -1 }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject limit exceeding MAX_FETCH_LIMIT', () => {
      const filters = { limit: 101 }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject invalid type', () => {
      const filters = { type: 'invalid' }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject invalid priority', () => {
      const filters = { priority: 'invalid' }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject invalid status', () => {
      const filters = { status: 'invalid' }
      const result = NotificationFiltersSchema.safeParse(filters)
      expect(result.success).toBe(false)
    })

    it('should reject invalid date formats', () => {
      const filters1 = { start_date: 'not-a-date' }
      const result1 = NotificationFiltersSchema.safeParse(filters1)
      expect(result1.success).toBe(false)

      const filters2 = { end_date: 'not-a-date' }
      const result2 = NotificationFiltersSchema.safeParse(filters2)
      expect(result2.success).toBe(false)
    })
  })

  // ============================================================================
  // SEND NOTIFICATION REQUEST SCHEMA TESTS
  // ============================================================================

  describe('SendNotificationRequestSchema', () => {
    const validRequest = {
      recipient_wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
      notification_type: 'system',
      priority: 'normal',
      title: 'Test Notification',
      message: 'This is a test message',
    }

    it('should validate request with recipient_wallet_address', () => {
      const result = SendNotificationRequestSchema.safeParse(validRequest)
      expect(result.success).toBe(true)
    })

    it('should validate request with recipient_group', () => {
      const request = {
        ...validRequest,
        recipient_wallet_address: undefined,
        recipient_group: 'admins',
      }
      const result = SendNotificationRequestSchema.safeParse(request)
      expect(result.success).toBe(true)
    })

    it('should validate request with broadcast', () => {
      const request = {
        ...validRequest,
        recipient_wallet_address: undefined,
        broadcast: true,
      }
      const result = SendNotificationRequestSchema.safeParse(request)
      expect(result.success).toBe(true)
    })

    it('should validate request with optional fields', () => {
      const request = {
        ...validRequest,
        data: { key: 'value' },
        action_url: 'https://example.com/action',
        image_url: 'https://example.com/image.png',
        expires_at: '2025-10-15T12:00:00Z',
        schedule_at: '2025-10-14T15:00:00Z',
      }

      const result = SendNotificationRequestSchema.safeParse(request)
      expect(result.success).toBe(true)
    })

    it('should reject request without any recipient specification', () => {
      const request = {
        ...validRequest,
        recipient_wallet_address: undefined,
      }
      const result = SendNotificationRequestSchema.safeParse(request)
      expect(result.success).toBe(false)
    })

    it('should reject empty title', () => {
      const request = { ...validRequest, title: '' }
      const result = SendNotificationRequestSchema.safeParse(request)
      expect(result.success).toBe(false)
    })

    it('should reject empty message', () => {
      const request = { ...validRequest, message: '' }
      const result = SendNotificationRequestSchema.safeParse(request)
      expect(result.success).toBe(false)
    })

    it('should reject invalid URLs', () => {
      const request1 = { ...validRequest, action_url: 'not-a-url' }
      const result1 = SendNotificationRequestSchema.safeParse(request1)
      expect(result1.success).toBe(false)

      const request2 = { ...validRequest, image_url: 'not-a-url' }
      const result2 = SendNotificationRequestSchema.safeParse(request2)
      expect(result2.success).toBe(false)
    })
  })

  // ============================================================================
  // VALIDATION HELPER FUNCTION TESTS
  // ============================================================================

  describe('validateNotification', () => {
    it('should return validated notification for valid data', () => {
      const data = {
        id: '123e4567-e89b-12d3-a456-426614174000',
        title: 'Test',
        message: 'Message',
        type: 'system',
        priority: 'normal',
        timestamp: '2025-10-14T12:00:00Z',
        read: false,
      }

      const result = validateNotification(data)
      expect(result).not.toBeNull()
      expect(result?.id).toBe(data.id)
    })

    it('should return null for invalid data', () => {
      const data = { invalid: 'data' }
      const result = validateNotification(data)
      expect(result).toBeNull()
    })

    it('should log error for invalid data', () => {
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation()
      const data = { invalid: 'data' }
      validateNotification(data)
      expect(consoleSpy).toHaveBeenCalled()
      consoleSpy.mockRestore()
    })
  })

  describe('validateSSENotification', () => {
    it('should return validated SSE notification for valid data', () => {
      const data = {
        id: '123e4567-e89b-12d3-a456-426614174000',
        wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
        notification_type: 'system',
        title: 'Test',
        message: 'Message',
        priority: 'normal',
        timestamp: '2025-10-14T12:00:00Z',
      }

      const result = validateSSENotification(data)
      expect(result).not.toBeNull()
      expect(result?.id).toBe(data.id)
    })

    it('should return null for invalid data', () => {
      const data = { invalid: 'data' }
      const result = validateSSENotification(data)
      expect(result).toBeNull()
    })
  })

  describe('validateNotificationFilters', () => {
    it('should return success result for valid filters', () => {
      const filters = { page: 1, limit: 20 }
      const result = validateNotificationFilters(filters)
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.data.page).toBe(1)
      }
    })

    it('should return error result for invalid filters', () => {
      const filters = { page: -1 }
      const result = validateNotificationFilters(filters)
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.errors).toBeDefined()
      }
    })
  })

  describe('validateSendNotificationRequest', () => {
    it('should return success result for valid request', () => {
      const request = {
        recipient_wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
        notification_type: 'system',
        priority: 'normal',
        title: 'Test',
        message: 'Message',
      }

      const result = validateSendNotificationRequest(request)
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.data.title).toBe('Test')
      }
    })

    it('should return error result for invalid request', () => {
      const request = { invalid: 'data' }
      const result = validateSendNotificationRequest(request)
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.errors).toBeDefined()
      }
    })
  })

  describe('validateNotifications', () => {
    it('should filter out invalid notifications and return valid ones', () => {
      const data = [
        {
          id: '123e4567-e89b-12d3-a456-426614174000',
          title: 'Valid',
          message: 'Message',
          type: 'system',
          priority: 'normal',
          timestamp: '2025-10-14T12:00:00Z',
          read: false,
        },
        { invalid: 'data' },
        {
          id: '223e4567-e89b-12d3-a456-426614174001',
          title: 'Valid 2',
          message: 'Message 2',
          type: 'security',
          priority: 'high',
          timestamp: '2025-10-14T13:00:00Z',
          read: false,
        },
      ]

      const result = validateNotifications(data)
      expect(result).toHaveLength(2)
      expect(result[0].title).toBe('Valid')
      expect(result[1].title).toBe('Valid 2')
    })

    it('should return empty array for all invalid notifications', () => {
      const data = [{ invalid: 'data1' }, { invalid: 'data2' }]
      const result = validateNotifications(data)
      expect(result).toHaveLength(0)
    })
  })
})
