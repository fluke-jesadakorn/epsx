/**
 * DISPLAY FORMATTING UTILITIES
 * Text formatting, truncation, and display helpers
 */

import { logger } from '../logger';

/**
 * Format percentage with proper sign and precision
 */
export function formatPercentage(value: number, precision = 1): string {
  const sign = value >= 0 ? '+' : ''
  return `${sign}${value.toFixed(precision)}%`
}

/**
 * Short alias for percentage formatting
 */
export const pct = (val: number, dec = 2): string => `${(val * 100).toFixed(dec)}%`

/**
 * Format EPS growth as percentage
 */
export function formatEPSGrowth(growth: number): string {
  const sign = growth >= 0 ? '+' : ''
  return `${sign}${growth.toFixed(1)}%`
}

/**
 * Short alias for EPS growth formatting
 */
export const epsGr = (growth: number | null | undefined): string =>
  growth === null || growth === undefined ? 'N/A' : `${growth > 0 ? '+' : ''}${growth}%`

/**
 * Capitalize first letter of string
 */
export function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

/**
 * Truncate text to specified length
 */
export function truncate(str: string, length = 50): string {
  if (str.length <= length) { return str }
  return `${str.slice(0, length)}...`
}

/**
 * Truncate text (alternative implementation)
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text
  }
  return `${text.slice(0, Math.max(0, maxLength - 3))}...`
}

/**
 * Convert string to a URL-friendly slug
 */
export function slugify(text: string): string {
  return text
    .toString()
    .toLowerCase()
    .replace(/\s+/g, '-')           // Replace spaces with -
    .replace(/[^w-]+/g, '')       // Remove all non-word chars
    .replace(/-+/g, '-')            // Replace multiple - with single -
    .replace(/^-+/, '')             // Trim - from start of text
    .replace(/-+$/, '')             // Trim - from end of text
}

/**
 * Convert string to kebab-case
 */
export function kebabCase(str: string): string {
  return str
    .replace(/([a-z])([A-Z])/g, '$1-$2')
    .replace(/[\s_]+/g, '-')
    .toLowerCase()
}

/**
 * Convert string to camelCase
 */
export function camelCase(str: string): string {
  return str
    .replace(/[-_\s]+(.)?/g, (_match: string, char: string | undefined) => char !== undefined ? char.toUpperCase() : '')
    .replace(/^[A-Z]/, char => char.toLowerCase())
}

/**
 * Check if email is valid
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email)
}

/**
 * Validate phone number (basic validation)
 */
export function isValidPhone(phone: string): boolean {
  const phoneRegex = /^[+]?[1-9][\d]{0,15}$/
  return phoneRegex.test(phone.replace(/[\s\-()]/g, ''))
}

/**
 * Get EPS performance color class
 */
export function getEPSPerformanceColor(growth: number): string {
  if (growth > 0) { return 'text-green-600' }
  if (growth < 0) { return 'text-red-600' }
  return 'text-gray-600'
}

/**
 * Determine growth indicator color and emoji
 */
export function getGrowthIndicator(growthPercent: number): { emoji: string; color: string; isPositive: boolean } {
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
 * Parse a JWT token without verification (client-side only)
 */
export function parseJWT(token: string): unknown {
  try {
    const parts = token.split('.');
    const base64Url = parts[1] || '';
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map(c => `%${(`00${c.charCodeAt(0).toString(16)}`).slice(-2)}`)
        .join('')
    )
    return JSON.parse(jsonPayload) as unknown
  } catch (error) {
    logger.error('Error parsing JWT:', error)
    return null
  }
}

/**
 * Check if a value is empty (null, undefined, empty string, empty array, empty object)
 */
export function isEmpty(value: unknown): boolean {
  if (value === null || value === undefined) { return true }
  if (typeof value === 'string' && value.trim() === '') { return true }
  if (Array.isArray(value) && value.length === 0) { return true }
  if (typeof value === 'object' && Object.keys(value).length === 0) { return true }
  return false
}

/**
 * Generate random ID
 */
export function generateId(prefix = 'id'): string {
  const timestamp = Date.now().toString(36)
  const randomStr = Math.random().toString(36).slice(2, 11)
  return `${prefix}_${timestamp}_${randomStr}`
}

/**
 * Simple random ID (legacy compatibility)
 */
export function generateSimpleId(): string {
  return Math.random().toString(36).slice(2, 11)
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (typeof window === 'undefined') { return false }

  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch (_error) {
    // Fallback for older browsers
    try {
      const textArea = document.createElement('textarea')
      textArea.value = text
      document.body.appendChild(textArea)
      textArea.select()
      document.execCommand('copy')
      document.body.removeChild(textArea)
      return true
    } catch (fallbackError) {
      logger.error('Failed to copy text to clipboard', fallbackError)
      return false
    }
  }
}

/**
 * Check if code is running in browser
 */
export function isBrowser(): boolean {
  return typeof window !== 'undefined'
}

/**
 * Check if code is running on mobile
 */
export function isMobile(): boolean {
  if (!isBrowser()) { return false }
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
}

/**
 * Clamp number between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max)
}