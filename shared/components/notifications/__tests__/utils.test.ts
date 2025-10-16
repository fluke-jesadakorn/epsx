/**
 * Unit Tests for Notification Utilities
 *
 * Tests all utility functions for notifications including:
 * - Icon mapping
 * - Timestamp formatting
 * - Wallet address formatting
 * - Priority color/styling functions
 */

import {
  getNotificationIcon,
  formatTimestamp,
  formatWalletAddress,
  getPriorityColor,
  getPriorityBgColor,
  getPriorityBgGradient,
  getPriorityBorderColor,
  getPriorityTextColor,
  getPrioritySubTextColor,
} from '../utils'

describe('Notification Utilities', () => {
  // ============================================================================
  // ICON MAPPING TESTS
  // ============================================================================

  describe('getNotificationIcon', () => {
    it('should return correct icon for each notification type', () => {
      expect(getNotificationIcon('system')).toBe('⚙️')
      expect(getNotificationIcon('security')).toBe('🔒')
      expect(getNotificationIcon('permission')).toBe('🔑')
      expect(getNotificationIcon('wallet_management')).toBe('💼')
      expect(getNotificationIcon('user_management')).toBe('👥')
      expect(getNotificationIcon('wallet')).toBe('👛')
      expect(getNotificationIcon('payment')).toBe('💳')
      expect(getNotificationIcon('general')).toBe('📬')
    })

    it('should return default icon for unknown type', () => {
      expect(getNotificationIcon('unknown' as any)).toBe('🔔')
    })

    it('should handle case sensitivity', () => {
      expect(getNotificationIcon('SYSTEM' as any)).toBe('🔔')
      expect(getNotificationIcon('Security' as any)).toBe('🔔')
    })
  })

  // ============================================================================
  // TIMESTAMP FORMATTING TESTS
  // ============================================================================

  describe('formatTimestamp', () => {
    const now = new Date('2025-10-14T12:00:00Z')

    beforeAll(() => {
      jest.useFakeTimers()
      jest.setSystemTime(now)
    })

    afterAll(() => {
      jest.useRealTimers()
    })

    it('should return "Just now" for timestamps less than 1 minute ago', () => {
      const timestamp = new Date(now.getTime() - 30 * 1000).toISOString()
      expect(formatTimestamp(timestamp)).toBe('Just now')
    })

    it('should return minutes for timestamps less than 60 minutes ago', () => {
      const timestamp = new Date(now.getTime() - 5 * 60 * 1000).toISOString()
      expect(formatTimestamp(timestamp)).toBe('5m ago')

      const timestamp59 = new Date(now.getTime() - 59 * 60 * 1000).toISOString()
      expect(formatTimestamp(timestamp59)).toBe('59m ago')
    })

    it('should return hours for timestamps less than 24 hours ago', () => {
      const timestamp = new Date(now.getTime() - 3 * 60 * 60 * 1000).toISOString()
      expect(formatTimestamp(timestamp)).toBe('3h ago')

      const timestamp23 = new Date(now.getTime() - 23 * 60 * 60 * 1000).toISOString()
      expect(formatTimestamp(timestamp23)).toBe('23h ago')
    })

    it('should return days for timestamps less than 7 days ago', () => {
      const timestamp = new Date(now.getTime() - 2 * 24 * 60 * 60 * 1000).toISOString()
      expect(formatTimestamp(timestamp)).toBe('2d ago')

      const timestamp6 = new Date(now.getTime() - 6 * 24 * 60 * 60 * 1000).toISOString()
      expect(formatTimestamp(timestamp6)).toBe('6d ago')
    })

    it('should return formatted date for timestamps older than 7 days', () => {
      const timestamp = new Date('2025-10-01T12:00:00Z').toISOString()
      const result = formatTimestamp(timestamp)
      expect(result).toMatch(/10\/1\/2025/)
    })

    it('should handle invalid timestamps gracefully', () => {
      expect(formatTimestamp('invalid')).toBe('Invalid date')
    })
  })

  // ============================================================================
  // WALLET ADDRESS FORMATTING TESTS
  // ============================================================================

  describe('formatWalletAddress', () => {
    it('should format standard wallet addresses correctly', () => {
      const address = '0x1234567890abcdef1234567890abcdef12345678'
      expect(formatWalletAddress(address)).toBe('0x1234...5678')
    })

    it('should handle short addresses', () => {
      expect(formatWalletAddress('0x12345')).toBe('0x12345')
    })

    it('should handle empty addresses', () => {
      expect(formatWalletAddress('')).toBe('')
    })

    it('should handle null/undefined', () => {
      expect(formatWalletAddress(null as any)).toBe('')
      expect(formatWalletAddress(undefined as any)).toBe('')
    })

    it('should preserve case of address', () => {
      const address = '0xABCD567890abcdef1234567890abcdef12345678'
      expect(formatWalletAddress(address)).toBe('0xABCD...5678')
    })
  })

  // ============================================================================
  // PRIORITY COLOR TESTS
  // ============================================================================

  describe('getPriorityColor', () => {
    it('should return correct hex colors for each priority', () => {
      expect(getPriorityColor('low')).toBe('#10b981')
      expect(getPriorityColor('normal')).toBe('#3b82f6')
      expect(getPriorityColor('high')).toBe('#f59e0b')
      expect(getPriorityColor('critical')).toBe('#ef4444')
      expect(getPriorityColor('urgent')).toBe('#ef4444')
    })

    it('should return normal color for unknown priority', () => {
      expect(getPriorityColor('unknown' as any)).toBe('#3b82f6')
    })
  })

  describe('getPriorityBgColor', () => {
    it('should return correct background colors for each priority', () => {
      expect(getPriorityBgColor('low')).toBe('bg-green-50')
      expect(getPriorityBgColor('normal')).toBe('bg-blue-50')
      expect(getPriorityBgColor('high')).toBe('bg-amber-50')
      expect(getPriorityBgColor('critical')).toBe('bg-red-50')
      expect(getPriorityBgColor('urgent')).toBe('bg-red-50')
    })

    it('should return normal bg color for unknown priority', () => {
      expect(getPriorityBgColor('unknown' as any)).toBe('bg-blue-50')
    })
  })

  describe('getPriorityBgGradient', () => {
    it('should return correct gradient classes for each priority', () => {
      expect(getPriorityBgGradient('low')).toContain('from-green-50')
      expect(getPriorityBgGradient('normal')).toContain('from-blue-50')
      expect(getPriorityBgGradient('high')).toContain('from-amber-50')
      expect(getPriorityBgGradient('critical')).toContain('from-red-50')
      expect(getPriorityBgGradient('urgent')).toContain('from-red-50')
    })

    it('should include gradient utility classes', () => {
      const gradient = getPriorityBgGradient('normal')
      expect(gradient).toContain('bg-gradient-to-r')
    })
  })

  describe('getPriorityBorderColor', () => {
    it('should return correct border colors for each priority', () => {
      expect(getPriorityBorderColor('low')).toBe('border-green-200')
      expect(getPriorityBorderColor('normal')).toBe('border-blue-200')
      expect(getPriorityBorderColor('high')).toBe('border-amber-200')
      expect(getPriorityBorderColor('critical')).toBe('border-red-200')
      expect(getPriorityBorderColor('urgent')).toBe('border-red-200')
    })
  })

  describe('getPriorityTextColor', () => {
    it('should return correct text colors for each priority', () => {
      expect(getPriorityTextColor('low')).toBe('text-green-700')
      expect(getPriorityTextColor('normal')).toBe('text-blue-700')
      expect(getPriorityTextColor('high')).toBe('text-amber-700')
      expect(getPriorityTextColor('critical')).toBe('text-red-700')
      expect(getPriorityTextColor('urgent')).toBe('text-red-700')
    })
  })

  describe('getPrioritySubTextColor', () => {
    it('should return correct subtext colors for each priority', () => {
      expect(getPrioritySubTextColor('low')).toBe('text-green-600')
      expect(getPrioritySubTextColor('normal')).toBe('text-blue-600')
      expect(getPrioritySubTextColor('high')).toBe('text-amber-600')
      expect(getPrioritySubTextColor('critical')).toBe('text-red-600')
      expect(getPrioritySubTextColor('urgent')).toBe('text-red-600')
    })
  })

  // ============================================================================
  // INTEGRATION TESTS
  // ============================================================================

  describe('Integration tests', () => {
    it('should work together for a complete notification display', () => {
      const notification = {
        type: 'security' as const,
        priority: 'critical' as const,
        timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
        wallet_address: '0x1234567890abcdef1234567890abcdef12345678',
      }

      const icon = getNotificationIcon(notification.type)
      const time = formatTimestamp(notification.timestamp)
      const wallet = formatWalletAddress(notification.wallet_address)
      const color = getPriorityColor(notification.priority)
      const bgGradient = getPriorityBgGradient(notification.priority)

      expect(icon).toBe('🔒')
      expect(time).toBe('5m ago')
      expect(wallet).toBe('0x1234...5678')
      expect(color).toBe('#ef4444')
      expect(bgGradient).toContain('from-red-50')
    })
  })
})
