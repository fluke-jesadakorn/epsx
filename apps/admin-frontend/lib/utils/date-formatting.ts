/**
 * Date Formatting Utilities
 * Consistent date formatting for admin EPS cards
 */

/**
 * Format date for quarter display (e.g., "Dec 15, 2024")
 */
export const formatQuarterDate = (dateString: string): string => {
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
export const formatAnnouncementDate = (dateString: string): string => {
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
export const formatCompactDate = (dateString: string): string => {
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
 * Get quarter label from date (e.g., "Q4 2024")
 */
export const getQuarterLabel = (dateString: string): string => {
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
export const isFutureDate = (dateString: string): boolean => {
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
 * Get relative time description (e.g., "2 days ago", "in 5 days")
 */
export const getRelativeTime = (dateString: string): string => {
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
export const formatTimeRemaining = (dateString: string): string => {
  try {
    const date = new Date(dateString)
    const now = new Date()
    const diffTime = date.getTime() - now.getTime()
    
    if (diffTime <= 0) {
      return 'Announcement passed'
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