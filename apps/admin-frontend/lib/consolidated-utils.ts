/**
 * Consolidated utility functions for admin frontend
 */

// Financial calculations
/**
 *
 * @param current
 * @param previous
 */
export function calculateEstimatedGrowth(current: number, previous: number): number {
  if (previous === 0) {return 0}
  return ((current - previous) / previous) * 100
}

/**
 *
 * @param targetDate
 */
export function calculateDaysRemaining(targetDate: string): number {
  const target = new Date(targetDate)
  const now = new Date()
  const diffTime = target.getTime() - now.getTime()
  return Math.ceil(diffTime / (1000 * 60 * 60 * 24))
}

/**
 *
 * @param price
 * @param eps
 */
export function calculatePERatio(price: number, eps: number): number {
  if (eps === 0) {return 0}
  return price / eps
}

/**
 *
 * @param current
 * @param previous
 */
export function calculatePriceGrowth(current: number, previous: number): number {
  if (previous === 0) {return 0}
  return ((current - previous) / previous) * 100
}

/**
 *
 * @param currentPrice
 * @param growth
 */
export function calculatePriceTarget(currentPrice: number, growth: number): number {
  return currentPrice * (1 + growth / 100)
}

// Formatters
/**
 *
 * @param days
 */
export function formatCountdown(days: number): string {
  if (days <= 0) {return "Overdue"}
  if (days === 1) {return "1 day"}
  return `${days} days`
}

/**
 *
 * @param growth
 */
export function getGrowthIndicator(growth: number): { color: string; emoji: string } {
  if (growth > 1) {return { color: "text-green-600 dark:text-green-400", emoji: "📈" }}
  if (growth < -1) {return { color: "text-red-600 dark:text-red-400", emoji: "📉" }}
  return { color: "text-gray-600 dark:text-gray-400", emoji: "➖" }
}

/**
 *
 * @param value
 */
export function formatPercentage(value: number): string {
  return `${value.toFixed(2)}%`
}

/**
 *
 * @param value
 */
export function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(value)
}

/**
 *
 * @param value
 */
export function formatEPS(value: number): string {
  return `$${value.toFixed(2)}`
}

/**
 *
 * @param value
 */
export function formatPERatio(value: number): string {
  if (value === 0) {return "N/A"}
  return value.toFixed(1)
}

/**
 *
 * @param date
 */
export function formatQuarterDate(date: string): string {
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  })
}

/**
 *
 * @param date
 */
export function formatAnnouncementDate(date: string): string {
  return new Date(date).toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric'
  })
}

/**
 *
 * @param quarterNumber
 */
export function getQuarterLabel(quarterNumber: number): string {
  return `Q${quarterNumber}`
}

/**
 *
 * @param date
 */
export function formatCompactDate(date: string): string {
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short',
    year: '2-digit'
  })
}

/**
 *
 * @param days
 */
export function formatTimeRemaining(days: number): string {
  if (days <= 0) {return "Overdue"}
  if (days === 1) {return "1 day remaining"}
  if (days <= 7) {return `${days} days remaining`}
  if (days <= 30) {return `${Math.ceil(days / 7)} weeks remaining`}
  return `${Math.ceil(days / 30)} months remaining`
}