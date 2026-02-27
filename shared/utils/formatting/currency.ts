/**
 * CURRENCY FORMATTING UTILITIES
 * Consolidated currency formatting functions from admin monolith
 */

/**
 * Format currency value with locale support
 */
export function formatCurrency(value: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
  }).format(value)
}

/**
 * Format currency value (alias for compatibility)
 */
export function fmtCurrency(value: number, currency = 'USD'): string {
  return formatCurrency(value, currency)
}

/**
 * Short alias for currency formatting
 */
export const cur = (amount: number, currency = 'USD'): string => 
  formatCurrency(amount, currency)

/**
 * Format EPS value for display with proper precision
 */
export function formatEPS(value: number): string {
  if (value === 0) {return '$0.00'}
  if (Math.abs(value) < 0.01) {
    return value < 0 ? '-$0.01' : '$0.01'
  }
  return formatCurrency(value)
}

/**
 * Format price with appropriate precision based on value
 */
export function formatPrice(price: number | null): string {
  if (price === null) {return 'N/A'}
  if (price >= 1000) {return price.toLocaleString('en-US', { maximumFractionDigits: 2 })}
  if (price >= 1) {return price.toFixed(2)}
  return price.toFixed(4)
}

/**
 * Short alias for price formatting
 */
export const prc = formatPrice

/**
 * Format large numbers with suffixes (K, M, B, T)
 */
export function formatLargeNumber(value: number): string {
  const suffixes = ['', 'K', 'M', 'B', 'T']
  let suffixIndex = 0
  let formattedValue = value

  while (Math.abs(formattedValue) >= 1000 && suffixIndex < suffixes.length - 1) {
    formattedValue /= 1000
    suffixIndex++
  }

  const precision = formattedValue >= 100 ? 0 : formattedValue >= 10 ? 1 : 2
  return formattedValue.toFixed(precision) + suffixes[suffixIndex]
}

/**
 * Format file size in human readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) {return '0 Bytes'}
  
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))  } ${  sizes[i]}`
}

/**
 * Alias for file size formatting
 */
export const formatBytes = formatFileSize

/**
 * Format a monetary amount - strips .00 from whole numbers, keeps meaningful decimals
 */
export const fmtAmt = (n: number): string => parseFloat(n.toFixed(2)).toString()