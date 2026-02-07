/**
 * Shared Notification Utilities
 *
 * Centralized utility functions for notification handling across frontend and admin applications.
 * Eliminates code duplication and ensures consistent behavior.
 */


// ============================================================================
// NOTIFICATION ICONS
// ============================================================================

/**
 * Get icon emoji for notification type
 * Used in: NotificationBellClient, AdminNotificationBellClient
 */
export function getNotificationIcon(type: string): string {
  switch (type) {
    case 'security':
      return '🔒'
    case 'permission':
      return '🔑'
    case 'wallet_management':
    case 'user_management':
      return '👥'
    case 'wallet':
      return '💼'
    case 'payment':
      return '💳'
    case 'system':
      return '⚙️'
    default:
      return '📬'
  }
}

// ============================================================================
// TIMESTAMP FORMATTING
// ============================================================================

/**
 * Format notification timestamp to relative time
 * Used in: NotificationBellClient, AdminNotificationBellClient
 */
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(diff / (1000 * 60 * 60))
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (minutes < 1) {return 'Just now'}
  if (minutes < 60) {return `${minutes}m ago`}
  if (hours < 24) {return `${hours}h ago`}
  if (days < 7) {return `${days}d ago`}
  return date.toLocaleDateString()
}

// ============================================================================
// PRIORITY COLORS
// ============================================================================

/**
 * Get Tailwind background color class for notification priority
 * Used in: NotificationBellClient
 */
export function getPriorityColor(priority: string): string {
  switch (priority) {
    case 'critical':
    case 'urgent':
      return '#ef4444'
    case 'high':
      return '#f59e0b'
    case 'normal':
      return '#3b82f6'
    case 'low':
      return '#10b981'
    default:
      return '#3b82f6'
  }
}

/**
 * Get Tailwind background color class for notification priority
 * Used in: AdminNotificationBellClient
 */
export function getPriorityBgColor(priority: string): string {
  switch (priority) {
    case 'critical':
    case 'urgent':
      return 'bg-red-50'
    case 'high':
      return 'bg-amber-50'
    case 'normal':
      return 'bg-blue-50'
    case 'low':
      return 'bg-green-50'
    default:
      return 'bg-blue-50'
  }
}

/**
 * Get gradient background classes for admin UI
 * Used in: AdminNotificationBellClient
 */
export function getPriorityBgGradient(priority: string): string {
  switch (priority) {
    case 'critical':
    case 'urgent':
      return 'from-red-50 to-pink-50 dark:from-red-900/20 dark:to-pink-900/20'
    case 'high':
      return 'from-orange-50 to-red-50 dark:from-orange-900/20 dark:to-red-900/20'
    case 'normal':
      return 'from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20'
    case 'low':
      return 'from-green-50 to-teal-50 dark:from-green-900/20 dark:to-teal-900/20'
    default:
      return 'from-gray-50 to-gray-100 dark:from-gray-900/20 dark:to-gray-800/20'
  }
}

/**
 * Get border color class for admin UI
 * Used in: AdminNotificationBellClient
 */
export function getPriorityBorderColor(priority: string): string {
  switch (priority) {
    case 'critical':
    case 'urgent':
      return 'border-red-200 dark:border-red-500/30'
    case 'high':
      return 'border-orange-200 dark:border-orange-500/30'
    case 'normal':
      return 'border-blue-200 dark:border-blue-500/30'
    case 'low':
      return 'border-green-200 dark:border-green-500/30'
    default:
      return 'border-gray-200 dark:border-gray-500/30'
  }
}

/**
 * Get text color class for priority
 * Used in: AdminNotificationBellClient
 */
export function getPriorityTextColor(priority: string): string {
  switch (priority) {
    case 'critical':
    case 'urgent':
      return 'text-red-800 dark:text-red-300'
    case 'high':
      return 'text-orange-800 dark:text-orange-300'
    case 'normal':
      return 'text-blue-800 dark:text-blue-300'
    case 'low':
      return 'text-green-800 dark:text-green-300'
    default:
      return 'text-gray-800 dark:text-gray-300'
  }
}

/**
 * Get subtext color class for priority
 * Used in: AdminNotificationBellClient
 */
export function getPrioritySubTextColor(priority: string): string {
  switch (priority) {
    case 'critical':
    case 'urgent':
      return 'text-red-600 dark:text-red-400'
    case 'high':
      return 'text-orange-600 dark:text-orange-400'
    case 'normal':
      return 'text-blue-600 dark:text-blue-400'
    case 'low':
      return 'text-green-600 dark:text-green-400'
    default:
      return 'text-gray-600 dark:text-gray-400'
  }
}

// ============================================================================
// NOTIFICATION VALIDATION
// ============================================================================

/**
 * Check if notification has expired
 */
export function isNotificationExpired(expiresAt?: string): boolean {
  if (!expiresAt) {return false}
  return new Date(expiresAt) < new Date()
}

/**
 * Filter expired notifications from array
 */
export function filterExpiredNotifications<T extends { expires_at?: string }>(
  notifications: T[]
): T[] {
  return notifications.filter((notif) => !isNotificationExpired(notif.expires_at))
}

/**
 * Sort notifications by timestamp (newest first)
 */
export function sortNotificationsByTimestamp<T extends { timestamp: string }>(
  notifications: T[]
): T[] {
  return [...notifications].sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  )
}

// ============================================================================
// WALLET ADDRESS FORMATTING
// ============================================================================

/**
 * Format wallet address for display (0x1234...5678)
 */
export function formatWalletAddress(address: string, prefixLen = 6, suffixLen = 4): string {
  if (!address || address.length < prefixLen + suffixLen) {
    return address
  }
  return `${address.slice(0, prefixLen)}...${address.slice(-suffixLen)}`
}
