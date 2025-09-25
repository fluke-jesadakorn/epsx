/**
 * DATE FORMATTING UTILITIES
 * Consolidated date formatting functions from admin monolith
 */

/**
 * Format date value with flexible format options
 */
export function formatDate(date: Date | string, format: 'short' | 'medium' | 'long' = 'medium'): string {
  const d = typeof date === 'string' ? new Date(date) : date
  
  const formats = {
    short: { month: 'short' as const, day: 'numeric' as const },
    medium: { month: 'short' as const, day: 'numeric' as const, year: 'numeric' as const },
    long: { month: 'long' as const, day: 'numeric' as const, year: 'numeric' as const }
  }
  
  return new Intl.DateTimeFormat('en-US', formats[format]).format(d)
}

/**
 * Short alias for date formatting
 */
export const dt = formatDate

/**
 * Alias for backwards compatibility
 */
export const fmtDate = formatDate

/**
 * Format relative time (e.g., "2 hours ago", "in 3 days")
 */
export function formatRelativeTime(date: Date | string): string {
  const now = new Date()
  const then = new Date(date)
  const diffInSeconds = Math.floor((now.getTime() - then.getTime()) / 1000)

  if (diffInSeconds < 60) return 'just now'
  if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`
  if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`
  if (diffInSeconds < 2592000) return `${Math.floor(diffInSeconds / 86400)}d ago`
  
  return formatDate(date)
}

/**
 * Alias for relative time formatting
 */
export const fmtRelativeTime = formatRelativeTime

/**
 * Format date for quarter display (e.g., "Dec 15, 2024")
 */
export function formatQuarterDate(dateString: string): string {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
  } catch (error) {
    console.error('Error formatting quarter date:', error)
    return dateString
  }
}

/**
 * Format date for announcement display (e.g., "January 28, 2025")
 */
export function formatAnnouncementDate(dateString: string): string {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('en-US', {
      month: 'long',
      day: 'numeric',
      year: 'numeric'
    })
  } catch (error) {
    console.error('Error formatting announcement date:', error)
    return dateString
  }
}

/**
 * Format date for compact display (e.g., "Jan 28")
 */
export function formatCompactDate(dateString: string): string {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric'
    })
  } catch (error) {
    console.error('Error formatting compact date:', error)
    return dateString
  }
}

/**
 * Get relative time description (enhanced version)
 */
export function getRelativeTime(dateString: string): string {
  try {
    const date = new Date(dateString)
    const now = new Date()
    const diffTime = date.getTime() - now.getTime()
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
    
    if (diffDays === 0) {
      return 'today'
    } else if (diffDays === 1) {
      return 'tomorrow'
    } else if (diffDays === -1) {
      return 'yesterday'
    } else if (diffDays > 1) {
      return `in ${diffDays} days`
    } else {
      return `${Math.abs(diffDays)} days ago`
    }
  } catch (error) {
    console.error('Error getting relative time:', error)
    return 'unknown'
  }
}

/**
 * Format time remaining with appropriate units
 */
export function formatTimeRemaining(dateString: string): string {
  try {
    const date = new Date(dateString)
    const now = new Date()
    const diffTime = date.getTime() - now.getTime()
    
    if (diffTime <= 0) {
      return 'Time passed'
    }
    
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24))
    const diffHours = Math.floor((diffTime % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
    
    if (diffDays > 0) {
      return `${diffDays} days${diffHours > 0 ? `, ${diffHours} hours` : ''}`
    } else if (diffHours > 0) {
      const diffMinutes = Math.floor((diffTime % (1000 * 60 * 60)) / (1000 * 60))
      return `${diffHours} hours${diffMinutes > 0 ? `, ${diffMinutes} minutes` : ''}`
    } else {
      const diffMinutes = Math.floor(diffTime / (1000 * 60))
      return `${Math.max(1, diffMinutes)} minutes`
    }
  } catch (error) {
    console.error('Error formatting time remaining:', error)
    return 'Unknown time remaining'
  }
}

/**
 * Get quarter label from date (e.g., "Q4 2024")
 */
export function getQuarterLabel(dateString: string): string {
  try {
    const date = new Date(dateString)
    const month = date.getMonth() + 1 // 1-based month
    const year = date.getFullYear()
    
    let quarter: number
    if (month >= 1 && month <= 3) {
      quarter = 1
    } else if (month >= 4 && month <= 6) {
      quarter = 2
    } else if (month >= 7 && month <= 9) {
      quarter = 3
    } else {
      quarter = 4
    }
    
    return `Q${quarter} ${year}`
  } catch (error) {
    console.error('Error getting quarter label:', error)
    return 'Unknown Quarter'
  }
}

/**
 * Check if a date is in the future
 */
export function isFutureDate(dateString: string): boolean {
  try {
    const date = new Date(dateString)
    const now = new Date()
    return date > now
  } catch (error) {
    console.error('Error checking if date is future:', error)
    return false
  }
}

/**
 * Format countdown display with days and hours
 */
export function formatCountdown(announcementDate: string): string {
  const daysRemaining = calculateDaysRemaining(announcementDate)
  
  if (daysRemaining === 0) {
    return 'Today'
  } else if (daysRemaining === 1) {
    return 'Tomorrow'
  } else {
    return `${daysRemaining} days remaining`
  }
}

/**
 * Calculate days remaining until announcement date
 */
export function calculateDaysRemaining(announcementDate: string): number {
  try {
    const now = new Date()
    const announcement = new Date(announcementDate)
    
    // Set to start of day for accurate day counting
    now.setHours(0, 0, 0, 0)
    announcement.setHours(0, 0, 0, 0)
    
    const diffTime = announcement.getTime() - now.getTime()
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
    
    return Math.max(0, diffDays)
  } catch (error) {
    console.error('Error calculating days remaining:', error)
    return 0
  }
}

/**
 * Calculate hours remaining until announcement
 */
export function calculateHoursRemaining(announcementDate: string): number {
  try {
    const now = new Date()
    const announcement = new Date(announcementDate)
    
    const diffTime = announcement.getTime() - now.getTime()
    const diffHours = Math.ceil(diffTime / (1000 * 60 * 60))
    
    return Math.max(0, diffHours)
  } catch (error) {
    console.error('Error calculating hours remaining:', error)
    return 0
  }
}