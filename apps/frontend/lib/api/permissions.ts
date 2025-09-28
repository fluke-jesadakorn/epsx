/**
 * Granular Permissions API Client
 * Provides client-side integration with the granular permission system
 */

import { 
  PermissionStatusResponse,
  PermissionHealthInfo,
  TokenValidationResult,
  HashValidationResult,
  PermissionNotificationEvent,
  GranularPermissionError as PermissionError
} from '@/shared/permissions/types';
import { getBackendUrl } from '../../../../shared/utils/url-resolver';
import { apiLogger, safeError } from '@/lib/utils/logging';

// Base configuration using centralized URL resolution
const API_BASE_URL = getBackendUrl('client');

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get authentication token from various sources
 */
function getAuthToken(): string | null {
  if (typeof window === 'undefined') return null;
  
  // Try access_token first (OIDC), fallback to legacy token
  return localStorage.getItem('access_token') || 
         localStorage.getItem('token') ||
         document.cookie.split('; ')
           .find(row => row.startsWith('access_token='))
           ?.split('=')[1] ||
         null;
}

/**
 * Create headers for API requests
 */
function createHeaders(includeAuth: boolean = true): HeadersInit {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    'Accept': 'application/json'
  };

  if (includeAuth) {
    const token = getAuthToken();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
  }

  return headers;
}

/**
 * Handle API response errors
 */
function handleApiError(error: any, context: string): never {
  const errorMessage = safeError(error).message;
  apiLogger.error(`Permissions API error in ${context}`, { error: errorMessage });
  
  const permissionError = {
    code: 'NETWORK_ERROR',
    message: errorMessage,
    details: context,
    timestamp: Date.now()
  };

  throw permissionError;
}

// ============================================================================
// Permissions API Client Class
// ============================================================================

export class PermissionsApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || API_BASE_URL;
  }

  /**
   * Get current user's permission status
   */
  async getPermissionStatus(userId?: string): Promise<PermissionStatusResponse> {
    try {
      const endpoint = userId ? `/api/permissions/status/${userId}` : '/api/permissions/status';
      
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'GET',
        headers: createHeaders(),
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'getPermissionStatus');
    }
  }

  /**
   * Validate a specific permission for current user
   */
  async validatePermission(permission: string, userId?: string): Promise<TokenValidationResult> {
    try {
      const endpoint = userId 
        ? `/api/permissions/validate/${userId}`
        : '/api/permissions/validate';

      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: createHeaders(),
        credentials: 'include',
        body: JSON.stringify({ permission })
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'validatePermission');
    }
  }

  /**
   * Validate multiple permissions at once
   */
  async validatePermissions(permissions: string[], userId?: string): Promise<TokenValidationResult[]> {
    try {
      const endpoint = userId 
        ? `/api/permissions/validate-batch/${userId}`
        : '/api/permissions/validate-batch';

      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: createHeaders(),
        credentials: 'include',
        body: JSON.stringify({ permissions })
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'validatePermissions');
    }
  }

  /**
   * Get health information about the permission system
   */
  async getHealthInfo(): Promise<PermissionHealthInfo> {
    try {
      const response = await fetch(`${this.baseUrl}/api/permissions/health`, {
        method: 'GET',
        headers: createHeaders(false), // Health endpoint doesn't require auth
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'getHealthInfo');
    }
  }

  /**
   * Refresh user permissions from the server
   */
  async refreshPermissions(userId?: string): Promise<PermissionStatusResponse> {
    try {
      const endpoint = userId 
        ? `/api/permissions/refresh/${userId}`
        : '/api/permissions/refresh';

      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: createHeaders(),
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'refreshPermissions');
    }
  }

  /**
   * Subscribe to permission change notifications
   */
  async subscribeToNotifications(callback: (event: PermissionNotificationEvent) => void): Promise<void> {
    try {
      const token = getAuthToken();
      if (!token) {
        throw new Error('Authentication token required for notifications');
      }

      const eventSource = new EventSource(
        `${this.baseUrl}/api/permissions/notifications?token=${encodeURIComponent(token)}`
      );

      eventSource.onmessage = (event) => {
        try {
          const notificationEvent: PermissionNotificationEvent = JSON.parse(event.data);
          callback(notificationEvent);
        } catch (error) {
          apiLogger.error('Failed to parse permission notification', { error: safeError(error).message });
        }
      };

      eventSource.onerror = (error) => {
        apiLogger.error('Permission notification stream error', { error });
        eventSource.close();
      };

    } catch (error) {
      handleApiError(error, 'subscribeToNotifications');
    }
  }

  /**
   * Validate permission hash (for embedded timestamp permissions)
   */
  async validatePermissionHash(hash: string): Promise<HashValidationResult> {
    try {
      const response = await fetch(`${this.baseUrl}/api/permissions/validate-hash`, {
        method: 'POST',
        headers: createHeaders(),
        credentials: 'include',
        body: JSON.stringify({ hash })
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'validatePermissionHash');
    }
  }

  /**
   * Get permission audit log for user
   */
  async getPermissionAuditLog(userId?: string, limit: number = 50): Promise<any[]> {
    try {
      const endpoint = userId 
        ? `/api/permissions/audit/${userId}?limit=${limit}`
        : `/api/permissions/audit?limit=${limit}`;

      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'GET',
        headers: createHeaders(),
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      handleApiError(error, 'getPermissionAuditLog');
    }
  }
}

// ============================================================================
// Permission Helper Functions
// ============================================================================

/**
 * Check if user has a specific permission (client-side check only)
 */
export async function hasPermission(permission: string, userId?: string): Promise<boolean> {
  try {
    const client = new PermissionsApiClient();
    const result = await client.validatePermission(permission, userId);
    return result.valid_permissions.includes(permission);
  } catch (error) {
    apiLogger.error('Permission check failed', { permission, error: safeError(error).message });
    return false;
  }
}

/**
 * Check if user has any of the specified permissions
 */
export async function hasAnyPermission(permissions: string[], userId?: string): Promise<boolean> {
  try {
    const client = new PermissionsApiClient();
    const results = await client.validatePermissions(permissions, userId);
    return results.some(result => result.valid_permissions.length > 0);
  } catch (error) {
    apiLogger.error('Batch permission check failed', { permissions, error: safeError(error).message });
    return false;
  }
}

/**
 * Check if user has all specified permissions
 */
export async function hasAllPermissions(permissions: string[], userId?: string): Promise<boolean> {
  try {
    const client = new PermissionsApiClient();
    const results = await client.validatePermissions(permissions, userId);
    return results.every(result => result.valid_permissions.length > 0);
  } catch (error) {
    apiLogger.error('Batch permission check failed', { permissions, error: safeError(error).message });
    return false;
  }
}

/**
 * Get current user's permission status
 */
export async function getPermissionStatus(userId?: string): Promise<PermissionStatusResponse | null> {
  try {
    const client = new PermissionsApiClient();
    return await client.getPermissionStatus(userId);
  } catch (error) {
    apiLogger.error('Failed to get permission status', { error: safeError(error).message });
    return null;
  }
}

// ============================================================================
// Exports
// ============================================================================

// Export singleton instance
export const permissionsApiClient = new PermissionsApiClient();

// Re-export types
export type {
  PermissionStatusResponse,
  PermissionHealthInfo,
  TokenValidationResult,
  HashValidationResult,
  PermissionNotificationEvent,
  PermissionError
};