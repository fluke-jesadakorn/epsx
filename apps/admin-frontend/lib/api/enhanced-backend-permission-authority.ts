// Enhanced Backend Permission Authority for Admin Frontend
// Simplified version for admin authentication and permission validation

export interface PermissionValidationResult {
  success: boolean
  data?: {
    hasPermission: boolean
    reason?: string
    expires_at?: string
  }
  error?: {
    type: string
    code: string
    message: string
  }
}

export interface PermissionValidationOptions {
  component: string
  context?: Record<string, any>
}

export class EnhancedBackendPermissionAuthority {
  private static instance: EnhancedBackendPermissionAuthority
  private baseUrl: string
  
  constructor() {
    this.baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  }
  
  static getInstance(): EnhancedBackendPermissionAuthority {
    if (!EnhancedBackendPermissionAuthority.instance) {
      EnhancedBackendPermissionAuthority.instance = new EnhancedBackendPermissionAuthority()
    }
    return EnhancedBackendPermissionAuthority.instance
  }
  
  async validatePermission(
    userId: string,
    permission: string,
    options?: PermissionValidationOptions
  ): Promise<PermissionValidationResult> {
    try {
      const response = await fetch(`${this.baseUrl}/api/admin/permissions/validate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${this.getAuthToken()}`
        },
        body: JSON.stringify({
          user_id: userId,
          permission,
          component: options?.component,
          context: options?.context
        }),
        credentials: 'include'
      })
      
      if (!response.ok) {
        return {
          success: false,
          error: {
            type: 'PERMISSION_VALIDATION_ERROR',
            code: 'HTTP_ERROR',
            message: `HTTP ${response.status}: ${response.statusText}`
          }
        }
      }
      
      const data = await response.json()
      return {
        success: true,
        data: {
          hasPermission: data.granted || false,
          reason: data.reason,
          expires_at: data.expires_at
        }
      }
      
    } catch (error) {
      return {
        success: false,
        error: {
          type: 'NETWORK_ERROR',
          code: 'FETCH_FAILED',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      }
    }
  }
  
  async validateMultiplePermissions(
    userId: string,
    permissions: string[],
    options?: PermissionValidationOptions
  ): Promise<PermissionValidationResult> {
    try {
      const response = await fetch(`${this.baseUrl}/api/admin/permissions/validate-bulk`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${this.getAuthToken()}`
        },
        body: JSON.stringify({
          user_id: userId,
          permissions,
          component: options?.component,
          context: options?.context
        }),
        credentials: 'include'
      })
      
      if (!response.ok) {
        return {
          success: false,
          error: {
            type: 'PERMISSION_VALIDATION_ERROR',
            code: 'HTTP_ERROR', 
            message: `HTTP ${response.status}: ${response.statusText}`
          }
        }
      }
      
      const data = await response.json()
      return {
        success: true,
        data: {
          hasPermission: data.results?.every((r: any) => r.granted) || false,
          reason: 'Multiple permissions validated'
        }
      }
      
    } catch (error) {
      return {
        success: false,
        error: {
          type: 'NETWORK_ERROR',
          code: 'FETCH_FAILED',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      }
    }
  }
  
  private getAuthToken(): string {
    // In a real implementation, get from cookies/session
    return 'placeholder-token'
  }
}

export const enhancedPermissionAuthority = EnhancedBackendPermissionAuthority.getInstance()