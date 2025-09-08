/**
 * EPS Calculation Utilities
 * Dynamic calculations for enhanced admin EPS cards
 */

/**
 * Calculate estimated growth percentage between two EPS values
 */
export const calculateEstimatedGrowth = (estimatedEPS: number, currentEPS: number): number => {
  if (!currentEPS || currentEPS === 0) return 0
  return ((estimatedEPS - currentEPS) / currentEPS) * 100
}

/**
 * Calculate days remaining until announcement date
 * Returns 0 if announcement date has passed
 */
export const calculateDaysRemaining = (announcementDate: string): number => {
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
 * Calculate hours remaining until announcement (for more precise countdown)
 */
export const calculateHoursRemaining = (announcementDate: string): number => {
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

/**
 * Format countdown display with days and hours
 */
export const formatCountdown = (announcementDate: string): string => {
  const daysRemaining = calculateDaysRemaining(announcementDate)
  
  if (daysRemaining === 0) {
    return 'Today'
  } else if (daysRemaining === 1) {
    return 'Tomorrow'
  } else if (daysRemaining <= 7) {
    return `${daysRemaining} days remaining`
  } else {
    return `${daysRemaining} days remaining`
  }
}

/**
 * Determine growth indicator color and emoji
 */
export const getGrowthIndicator = (growthPercent: number): { emoji: string; color: string; isPositive: boolean } => {
  if (growthPercent > 0) {
    return {
      emoji: '📈',
      color: 'text-green-600 dark:text-green-400',
      isPositive: true
    }
  } else if (growthPercent < 0) {
    return {
      emoji: '📉', 
      color: 'text-red-600 dark:text-red-400',
      isPositive: false
    }
  } else {
    return {
      emoji: '➡️',
      color: 'text-gray-600 dark:text-gray-400',
      isPositive: false
    }
  }
}

/**
 * Format percentage with proper sign and precision
 */
export const formatPercentage = (value: number, precision: number = 1): string => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}${value.toFixed(precision)}%`
}

/**
 * Format currency values consistently
 */
export const formatCurrency = (value: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value)
}

/**
 * Format EPS values with proper precision (no currency symbol)
 */
export const formatEPS = (value: number): string => {
  return value.toFixed(2)
}

/**
 * Calculate P/E ratio from price and EPS
 */
export const calculatePERatio = (price: number, eps: number): number => {
  if (!eps || eps === 0) return 0
  return price / eps
}

/**
 * Format P/E ratio with proper precision and 'x' suffix
 */
export const formatPERatio = (peRatio: number): string => {
  if (peRatio === 0 || !isFinite(peRatio)) return 'N/A'
  return `${peRatio.toFixed(1)}x`
}

/**
 * Calculate price growth percentage between two price values
 */
export const calculatePriceGrowth = (currentPrice: number, previousPrice: number): number => {
  if (!previousPrice || previousPrice === 0) return 0
  return ((currentPrice - previousPrice) / previousPrice) * 100
}

/**
 * Calculate estimated price target based on estimated EPS and current P/E ratio
 */
export const calculatePriceTarget = (estimatedEPS: number, currentPE: number): number => {
  if (!estimatedEPS || !currentPE || currentPE === 0) return 0
  return estimatedEPS * currentPE
}

/**
 * Get quarter priority based on growth and timing
 */
export const getQuarterPriority = (epsGrowth: number, daysUntilAnnouncement: number): 'high' | 'medium' | 'low' => {
  if (Math.abs(epsGrowth) > 20 || daysUntilAnnouncement <= 7) {
    return 'high'
  } else if (Math.abs(epsGrowth) > 10 || daysUntilAnnouncement <= 14) {
    return 'medium'
  }
  return 'low'
}