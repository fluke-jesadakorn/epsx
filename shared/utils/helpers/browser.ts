/**
 * BROWSER UTILITIES
 * Client-side utilities for DOM, storage, clipboard, and device detection
 */

/**
 * Cookie storage utilities with error handling (replacing localStorage)
 */
export const storage = {
  /**
   * Set item in cookie with error handling
   */
  set(key: string, value: any, maxAge?: number): boolean {
    try {
      if (typeof window === 'undefined') return false
      const encodedValue = encodeURIComponent(JSON.stringify(value))
      const maxAgeStr = maxAge ? `max-age=${maxAge}` : ''
      document.cookie = `${key}=${encodedValue}; path=/; ${maxAgeStr} SameSite=lax`
      return true
    } catch (error) {
      console.error('Failed to save to cookie:', error)
      return false
    }
  },

  /**
   * Get item from cookie with error handling, fallback to localStorage for migration
   */
  get<T>(key: string, defaultValue?: T): T | null {
    try {
      if (typeof window === 'undefined') return defaultValue || null

      // Try cookie first
      const cookies = document.cookie.split(';').reduce((acc, cookie) => {
        const parts = cookie.trim().split('=')
        const k = parts[0] || ''
        const v = parts[1] || ''
        if (k && v) acc[k] = v
        return acc
      }, {} as Record<string, string>)

      const cookieValue = cookies[key]
      if (cookieValue) {
        return JSON.parse(decodeURIComponent(cookieValue))
      }

      // Fallback to localStorage for migration
      const item = localStorage.getItem(key)
      return item ? JSON.parse(item) : defaultValue || null
    } catch (error) {
      console.error('Failed to read from cookie:', error)
      return defaultValue || null
    }
  },

  /**
   * Remove item from cookie and localStorage
   */
  remove(key: string): boolean {
    try {
      if (typeof window === 'undefined') return false

      // Remove from cookie
      document.cookie = `${key}=; max-age=0; path=/; SameSite=lax`

      // Remove from localStorage (fallback cleanup)
      localStorage.removeItem(key)
      return true
    } catch (error) {
      console.error('Failed to remove from cookie:', error)
      return false
    }
  },

  /**
   * Clear all cookies and localStorage
   */
  clear(): boolean {
    try {
      if (typeof window === 'undefined') return false

      // Clear localStorage
      localStorage.clear()

      // Clear all cookies by setting them to expire
      document.cookie.split(';').forEach(cookie => {
        const key = (cookie.split('=')[0] || '').trim()
        if (key) {
          document.cookie = `${key}=; max-age=0; path=/; SameSite=lax`
        }
      })
      return true
    } catch (error) {
      console.error('Failed to clear storage:', error)
      return false
    }
  },

  /**
   * Check if storage is available
   */
  isAvailable(): boolean {
    if (typeof window === 'undefined') return false

    // Try cookies
    const test = '__storage_test__'
    try {
      document.cookie = `${test}=test; path=/; SameSite=lax`
      const hasCookie = document.cookie.includes(test)
      document.cookie = `${test}=; max-age=0; path=/; SameSite=lax`
      return hasCookie
    } catch {
      return false
    }
  }
}

/**
 * Session storage utilities
 */
export const sessionStorage = {
  /**
   * Set item in sessionStorage with error handling
   */
  set(key: string, value: any): boolean {
    try {
      if (typeof window === 'undefined') return false
      window.sessionStorage.setItem(key, JSON.stringify(value))
      return true
    } catch (error) {
      console.error('Failed to save to sessionStorage:', error)
      return false
    }
  },

  /**
   * Get item from sessionStorage with error handling
   */
  get<T>(key: string, defaultValue?: T): T | null {
    try {
      if (typeof window === 'undefined') return defaultValue || null
      const item = window.sessionStorage.getItem(key)
      return item ? JSON.parse(item) : defaultValue || null
    } catch (error) {
      console.error('Failed to read from sessionStorage:', error)
      return defaultValue || null
    }
  },

  /**
   * Remove item from sessionStorage
   */
  remove(key: string): boolean {
    try {
      if (typeof window === 'undefined') return false
      window.sessionStorage.removeItem(key)
      return true
    } catch (error) {
      console.error('Failed to remove from sessionStorage:', error)
      return false
    }
  },

  /**
   * Clear all sessionStorage
   */
  clear(): boolean {
    try {
      if (typeof window === 'undefined') return false
      window.sessionStorage.clear()
      return true
    } catch (error) {
      console.error('Failed to clear sessionStorage:', error)
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
 * Check if code is running on mobile device
 */
export function isMobile(): boolean {
  if (!isBrowser()) return false
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
}

/**
 * Check if device is iOS
 */
export function isIOS(): boolean {
  if (!isBrowser()) return false
  return /iPad|iPhone|iPod/.test(navigator.userAgent)
}

/**
 * Check if device is Android
 */
export function isAndroid(): boolean {
  if (!isBrowser()) return false
  return /Android/.test(navigator.userAgent)
}

/**
 * Get device type
 */
export function getDeviceType(): 'desktop' | 'tablet' | 'mobile' {
  if (!isBrowser()) return 'desktop'

  const userAgent = navigator.userAgent

  if (/iPad/.test(userAgent) || (isMobile() && window.innerWidth > 768)) {
    return 'tablet'
  }

  if (isMobile()) {
    return 'mobile'
  }

  return 'desktop'
}

/**
 * Get browser information
 */
export function getBrowserInfo(): {
  name: string
  version: string
  engine: string
} {
  if (!isBrowser()) {
    return { name: 'unknown', version: 'unknown', engine: 'unknown' }
  }

  const userAgent = navigator.userAgent

  // Browser detection
  let name = 'unknown'
  let version = 'unknown'
  let engine = 'unknown'

  if (userAgent.includes('Chrome')) {
    name = 'Chrome'
    version = userAgent.match(/Chrome\/(\d+)/)?.[1] || 'unknown'
    engine = 'Blink'
  } else if (userAgent.includes('Firefox')) {
    name = 'Firefox'
    version = userAgent.match(/Firefox\/(\d+)/)?.[1] || 'unknown'
    engine = 'Gecko'
  } else if (userAgent.includes('Safari')) {
    name = 'Safari'
    version = userAgent.match(/Version\/(\d+)/)?.[1] || 'unknown'
    engine = 'WebKit'
  } else if (userAgent.includes('Edge')) {
    name = 'Edge'
    version = userAgent.match(/Edge\/(\d+)/)?.[1] || 'unknown'
    engine = 'EdgeHTML'
  }

  return { name, version, engine }
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (!isBrowser()) return false

  try {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text)
      return true
    }
    throw new Error('Clipboard API not available')
  } catch (error) {
    // Fallback for older browsers
    try {
      const textArea = document.createElement('textarea')
      textArea.value = text
      textArea.style.position = 'absolute'
      textArea.style.left = '-9999px'
      document.body.appendChild(textArea)
      textArea.select()
      document.execCommand('copy')
      document.body.removeChild(textArea)
      return true
    } catch (fallbackError) {
      console.error('Failed to copy text to clipboard', fallbackError)
      return false
    }
  }
}

/**
 * Read text from clipboard
 */
export async function readFromClipboard(): Promise<string | null> {
  if (!isBrowser()) return null

  try {
    const text = await navigator.clipboard.readText()
    return text
  } catch (error) {
    console.error('Failed to read from clipboard:', error)
    return null
  }
}

/**
 * Download data as file
 */
export function downloadFile(data: string | Blob, filename: string, mimeType?: string): boolean {
  if (!isBrowser()) return false

  try {
    const blob = data instanceof Blob ? data : new Blob([data], { type: mimeType || 'text/plain' })
    const url = URL.createObjectURL(blob)

    const link = document.createElement('a')
    link.href = url
    link.download = filename
    link.style.display = 'none'

    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)

    URL.revokeObjectURL(url)
    return true
  } catch (error) {
    console.error('Failed to download file:', error)
    return false
  }
}

/**
 * Get viewport dimensions
 */
export function getViewportSize(): { width: number; height: number } {
  if (!isBrowser()) return { width: 0, height: 0 }

  return {
    width: window.innerWidth,
    height: window.innerHeight
  }
}

/**
 * Get scroll position
 */
export function getScrollPosition(): { x: number; y: number } {
  if (!isBrowser()) return { x: 0, y: 0 }

  return {
    x: window.pageXOffset || document.documentElement.scrollLeft,
    y: window.pageYOffset || document.documentElement.scrollTop
  }
}

/**
 * Smooth scroll to element or position
 */
export function smoothScrollTo(target: number | Element, options?: ScrollToOptions): void {
  if (!isBrowser()) return

  if (typeof target === 'number') {
    window.scrollTo({
      top: target,
      behavior: 'smooth',
      ...options
    })
  } else {
    target.scrollIntoView({
      behavior: 'smooth',
      block: 'start',
      ...options
    })
  }
}

/**
 * Check if element is in viewport
 */
export function isElementInViewport(element: Element): boolean {
  if (!isBrowser()) return false

  const rect = element.getBoundingClientRect()
  const viewport = getViewportSize()

  return (
    rect.top >= 0 &&
    rect.left >= 0 &&
    rect.bottom <= viewport.height &&
    rect.right <= viewport.width
  )
}

/**
 * Add event listener with cleanup
 */
export function addEventListener<K extends keyof WindowEventMap>(
  type: K,
  listener: (event: WindowEventMap[K]) => void,
  options?: boolean | AddEventListenerOptions
): () => void {
  if (!isBrowser()) return () => { }

  window.addEventListener(type, listener, options)

  return () => {
    window.removeEventListener(type, listener, options)
  }
}

/**
 * Debounced resize listener
 */
export function onResize(
  callback: (size: { width: number; height: number }) => void,
  delay: number = 100
): () => void {
  if (!isBrowser()) return () => { }

  let timeoutId: number

  const debouncedCallback = () => {
    clearTimeout(timeoutId)
    timeoutId = window.setTimeout(() => {
      callback(getViewportSize())
    }, delay)
  }

  window.addEventListener('resize', debouncedCallback)

  return () => {
    clearTimeout(timeoutId)
    window.removeEventListener('resize', debouncedCallback)
  }
}

/**
 * Create and manage CSS variables
 */
export const cssVariables = {
  /**
   * Set CSS variable
   */
  set(name: string, value: string, element: Element = document.documentElement): void {
    if (!isBrowser()) return
      ; (element as HTMLElement).style.setProperty(`--${name}`, value)
  },

  /**
   * Get CSS variable
   */
  get(name: string, element: Element = document.documentElement): string {
    if (!isBrowser()) return ''
    return getComputedStyle(element).getPropertyValue(`--${name}`).trim()
  },

  /**
   * Remove CSS variable
   */
  remove(name: string, element: Element = document.documentElement): void {
    if (!isBrowser()) return
      ; (element as HTMLElement).style.removeProperty(`--${name}`)
  }
}

/**
 * Detect color scheme preference
 */
export function getColorSchemePreference(): 'light' | 'dark' | 'no-preference' {
  if (!isBrowser()) return 'no-preference'

  if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }

  if (window.matchMedia('(prefers-color-scheme: light)').matches) {
    return 'light'
  }

  return 'no-preference'
}

/**
 * Watch for color scheme changes
 */
export function watchColorScheme(
  callback: (scheme: 'light' | 'dark' | 'no-preference') => void
): () => void {
  if (!isBrowser()) return () => { }

  const darkQuery = window.matchMedia('(prefers-color-scheme: dark)')
  const lightQuery = window.matchMedia('(prefers-color-scheme: light)')

  const handler = () => {
    callback(getColorSchemePreference())
  }

  darkQuery.addEventListener('change', handler)
  lightQuery.addEventListener('change', handler)

  return () => {
    darkQuery.removeEventListener('change', handler)
    lightQuery.removeEventListener('change', handler)
  }
}