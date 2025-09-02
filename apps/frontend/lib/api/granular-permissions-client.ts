/**
 * Granular Permissions API Client - Frontend
 * Provides client-side integration with the granular permission system
 */

import { 
  PermissionStatusResponse,
  PermissionHealthInfo,
  TokenValidationResult,
  HashValidationResult,
  PermissionNotificationEvent,
  PermissionError
} from '@/types/granular-permissions'

// Base configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080'

// Helper function to get auth token
function getAuthToken(): string | null {
  if (typeof window === 'undefined') return null
  
  // Try access_token first (OIDC), fallback to legacy token
  return localStorage.getItem('access_token') || 
         localStorage.getItem('token') ||
         document.cookie.split('; ')
           .find(row => row.startsWith('access_token='))
           ?.split('=')[1] ||
         null
}

// Base fetch function with authentication and error handling
async function granularPermissionsFetch(
  endpoint: string, 
  options: RequestInit = {}
): Promise<any> {
  try {
    const token = getAuthToken()
    
    if (!token) {
      throw new Error('No authentication token available')
    }

    const headers = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
      ...options.headers,
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      const errorBody = await response.text().catch(() => 'Unknown error')
      
      // Handle specific error types
      if (response.status === 401) {
        // Token expired or invalid
        localStorage.removeItem('access_token')
        localStorage.removeItem('token')
        throw new Error('Authentication expired. Please log in again.')
      }
      
      if (response.status === 403) {
        throw new Error('Insufficient permissions for this operation')
      }
      
      if (response.status === 429) {
        throw new Error('Rate limit exceeded. Please try again later.')
      }

      let errorMessage = `API Error: ${response.status} ${response.statusText}`
      try {
        const errorJson = JSON.parse(errorBody)
        if (errorJson.message) {
          errorMessage = errorJson.message
        }
      } catch {
        // Use default error message if can't parse JSON
      }

      throw new Error(errorMessage)
    }

    // Handle updated token in response headers
    const updatedToken = response.headers.get('X-Updated-Token')
    if (updatedToken) {
      localStorage.setItem('access_token', updatedToken)
    }

    return response.json()
    
  } catch (error) {
    console.error('Granular Permissions API Error:', {
      endpoint,
      error: error instanceof Error ? error.message : error
    })
    throw error
  }
}

// User Permission Status API
export class UserPermissionAPI {
  /**
   * Get current user's permission status
   */
  static async getPermissionStatus(): Promise<PermissionStatusResponse> {
    return granularPermissionsFetch('/api/v1/permissions/status')
  }

  /**
   * Get permission health information
   */
  static async getPermissionHealth(): Promise<PermissionHealthInfo> {
    const status = await this.getPermissionStatus()
    return status.health
  }

  /**
   * Refresh user's JWT token with updated permissions
   */
  static async refreshToken(): Promise<TokenValidationResult> {
    return granularPermissionsFetch('/api/v1/permissions/refresh-token', {
      method: 'POST'
    })
  }

  /**
   * Validate permission hash for instant revocation check
   */
  static async validatePermissionHash(hash: string): Promise<HashValidationResult> {
    return granularPermissionsFetch('/api/v1/permissions/validate-hash', {
      method: 'POST',
      body: JSON.stringify({ hash })
    })
  }

  /**
   * Get detailed expiry information for specific permissions
   */
  static async getPermissionExpiry(permissions: string[]): Promise<{
    permission: string
    expires_at?: number
    is_expired: boolean
    time_remaining?: number
    expires_in?: string
  }[]> {
    return granularPermissionsFetch('/api/v1/permissions/expiry', {
      method: 'POST',
      body: JSON.stringify({ permissions })
    })
  }

  /**
   * Check if specific permissions are valid and not expired
   */
  static async validatePermissions(permissions: string[]): Promise<{
    valid: string[]
    expired: string[]
    missing: string[]
  }> {
    return granularPermissionsFetch('/api/v1/permissions/validate', {
      method: 'POST',
      body: JSON.stringify({ permissions })
    })
  }

  /**
   * Get permission usage statistics for the current user
   */
  static async getPermissionUsage(): Promise<{
    total_requests_last_24h: number
    top_used_permissions: Array<{ permission: string; usage_count: number }>
    last_permission_check: string
  }> {
    return granularPermissionsFetch('/api/v1/permissions/usage')
  }
}

// Permission Notifications API
export class PermissionNotificationAPI {
  /**
   * Get recent permission-related notifications
   */
  static async getPermissionNotifications(limit: number = 50): Promise<PermissionNotificationEvent[]> {
    return granularPermissionsFetch(`/api/v1/permissions/notifications?limit=${limit}`)
  }

  /**
   * Mark permission notifications as read
   */
  static async markNotificationsRead(notificationIds: string[]): Promise<void> {
    return granularPermissionsFetch('/api/v1/permissions/notifications/mark-read', {
      method: 'POST',
      body: JSON.stringify({ notification_ids: notificationIds })
    })
  }

  /**
   * Subscribe to permission expiry alerts
   */
  static async subscribeToExpiryAlerts(hoursBeforeExpiry: number = 24): Promise<void> {
    return granularPermissionsFetch('/api/v1/permissions/notifications/subscribe-expiry', {
      method: 'POST',
      body: JSON.stringify({ hours_before_expiry: hoursBeforeExpiry })
    })
  }

  /**
   * Unsubscribe from permission expiry alerts
   */
  static async unsubscribeFromExpiryAlerts(): Promise<void> {
    return granularPermissionsFetch('/api/v1/permissions/notifications/unsubscribe-expiry', {
      method: 'POST'
    })
  }
}

// Feature Access API
export class FeatureAccessAPI {
  /**
   * Check access to specific features with permission validation
   */
  static async checkFeatureAccess(features: string[]): Promise<{
    [feature: string]: {
      has_access: boolean
      required_permissions: string[]
      missing_permissions: string[]
      expiry_info?: {
        expires_at?: number
        is_expiring_soon: boolean
        time_remaining?: number
      }
    }
  }> {
    return granularPermissionsFetch('/api/v1/permissions/feature-access', {
      method: 'POST',
      body: JSON.stringify({ features })
    })
  }

  /**
   * Get available features for the current user
   */
  static async getAvailableFeatures(): Promise<{
    features: Array<{
      feature_name: string
      required_permissions: string[]
      description?: string
      is_available: boolean
      expires_at?: number
    }>
  }> {
    return granularPermissionsFetch('/api/v1/permissions/available-features')
  }

  /**
   * Get feature usage recommendations based on permissions
   */
  static async getFeatureRecommendations(): Promise<{
    recommendations: Array<{
      feature_name: string
      reason: string
      potential_benefit: string
      required_upgrade?: string
    }>
  }> {
    return granularPermissionsFetch('/api/v1/permissions/feature-recommendations')
  }
}

// Route Access Validation API
export class RouteAccessAPI {
  /**
   * Validate access to specific routes
   */
  static async validateRouteAccess(routes: string[]): Promise<{
    [route: string]: {
      has_access: boolean
      required_permissions: string[]
      redirect_url?: string
    }
  }> {
    return granularPermissionsFetch('/api/v1/permissions/route-access', {
      method: 'POST',
      body: JSON.stringify({ routes })
    })
  }

  /**
   * Get navigation menu based on current permissions
   */
  static async getNavigationMenu(): Promise<{
    menu_items: Array<{
      label: string
      url: string
      icon?: string
      children?: Array<{
        label: string
        url: string
        icon?: string
      }>
      requires_permissions: string[]
      is_available: boolean
    }>
  }> {
    return granularPermissionsFetch('/api/v1/permissions/navigation')
  }
}

// Error handling utilities
export function isPermissionError(error: any): error is PermissionError {
  return error && typeof error === 'object' && 'code' in error && 'message' in error
}

export function handlePermissionError(error: any): PermissionError {
  if (isPermissionError(error)) {
    return error
  }

  if (error instanceof Error) {
    if (error.message.includes('Authentication expired')) {
      return {
        code: 'TOKEN_INVALID',
        message: error.message,
        details: 'Please log in again'
      }
    }

    if (error.message.includes('Insufficient permissions')) {
      return {
        code: 'INSUFFICIENT_PERMISSION',
        message: error.message,
        details: 'Contact your administrator to request additional permissions'
      }
    }

    if (error.message.includes('expired') || error.message.includes('Expired')) {
      return {
        code: 'PERMISSION_EXPIRED',
        message: error.message,
        details: 'Some of your permissions have expired'
      }
    }
  }

  return {
    code: 'VALIDATION_ERROR',
    message: error?.message || 'Unknown permission error',
    details: error?.toString()
  }
}

// Real-time permission updates (if WebSocket is available)
export class PermissionRealtimeAPI {
  private static eventSource: EventSource | null = null

  /**
   * Subscribe to real-time permission updates
   */
  static subscribeToPermissionUpdates(
    onUpdate: (event: PermissionNotificationEvent) => void,
    onError: (error: Error) => void = console.error
  ): () => void {
    const token = getAuthToken()
    if (!token) {
      onError(new Error('No authentication token for real-time updates'))
      return () => {}
    }

    try {
      const url = `${API_BASE_URL}/api/v1/permissions/events?token=${encodeURIComponent(token)}`
      this.eventSource = new EventSource(url)

      this.eventSource.onmessage = (event) => {
        try {
          const permissionEvent: PermissionNotificationEvent = JSON.parse(event.data)
          onUpdate(permissionEvent)
        } catch (err) {
          onError(new Error('Failed to parse permission update event'))
        }
      }

      this.eventSource.onerror = () => {
        onError(new Error('Permission updates connection failed'))
      }

      return () => {
        if (this.eventSource) {
          this.eventSource.close()
          this.eventSource = null
        }
      }
    } catch (err) {
      onError(new Error('Failed to establish real-time permission updates'))
      return () => {}
    }
  }

  /**
   * Check if real-time updates are supported
   */
  static isRealtimeSupported(): boolean {
    return typeof EventSource !== 'undefined'
  }
}

// Export main API classes
export {
  UserPermissionAPI,
  PermissionNotificationAPI,
  FeatureAccessAPI,
  RouteAccessAPI,
  PermissionRealtimeAPI
}