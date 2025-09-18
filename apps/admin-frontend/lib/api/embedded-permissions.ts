// Embedded timestamp permissions API client

import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';
import type {
  EmbeddedPermissionRequest,
  EmbeddedPermissionResponse,
  BulkEmbeddedPermissionRequest,
  BulkPermissionResponse,
  ValidatePermissionsRequest,
  ValidationResult,
  ExpiryStatusResponse,
  RevokePermissionRequest,
  ExtendPermissionRequest,
  ExtendPermissionResponse,
  CleanupExpiredRequest,
  CleanupResponse,
  ApiErrorResponse
} from '@/types/admin/embedded-permissions';

const API_BASE = URL.get(Service.BACKEND, URLContext.CLIENT);

class EmbeddedPermissionError extends Error {
  constructor(
    message: string,
    public status: number,
    public details?: string
  ) {
    super(message);
    this.name = 'EmbeddedPermissionError';
  }
}

async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE}/api/v1/admin${endpoint}`;
  
  // Get auth token from cookies or session
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    credentials: 'include',
  });

  if (!response.ok) {
    const errorData: ApiErrorResponse = await response.json().catch(() => ({
      error: 'unknown_error',
      message: `HTTP ${response.status}: ${response.statusText}`,
    }));
    
    throw new EmbeddedPermissionError(
      errorData.message || 'An error occurred',
      response.status,
      errorData.details
    );
  }

  return response.json();
}

export const EmbeddedPermissionsApi = {
  // Grant embedded timestamp permission to a user
  // POST /admin/users/:user_id/embedded-permissions
  async grantPermission(
    userId: string,
    request: EmbeddedPermissionRequest
  ): Promise<EmbeddedPermissionResponse> {
    return apiRequest<EmbeddedPermissionResponse>(
      `/users/${userId}/embedded-permissions`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  },

  // Grant embedded timestamp permissions to multiple users
  // POST /admin/users/bulk/embedded-permissions
  async grantBulkPermissions(
    request: BulkEmbeddedPermissionRequest
  ): Promise<BulkPermissionResponse> {
    return apiRequest<BulkPermissionResponse>(
      '/users/bulk/embedded-permissions',
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  },

  // Validate embedded permissions for a user
  // POST /admin/users/:user_id/embedded-permissions/validate
  async validatePermissions(
    userId: string,
    request: ValidatePermissionsRequest
  ): Promise<ValidationResult> {
    return apiRequest<ValidationResult>(
      `/users/${userId}/embedded-permissions/validate`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  },

  // Get permission expiry status for a user
  // GET /admin/users/:user_id/permissions/expiry-status
  async getExpiryStatus(userId: string): Promise<ExpiryStatusResponse> {
    return apiRequest<ExpiryStatusResponse>(
      `/users/${userId}/permissions/expiry-status`
    );
  },

  // Revoke an embedded timestamp permission
  // POST /admin/users/:user_id/embedded-permissions/revoke
  async revokePermission(
    userId: string,
    request: RevokePermissionRequest
  ): Promise<void> {
    await apiRequest<void>(
      `/users/${userId}/embedded-permissions/revoke`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  },

  // Extend an embedded timestamp permission's expiry
  // POST /admin/users/:user_id/embedded-permissions/extend
  async extendPermission(
    userId: string,
    request: ExtendPermissionRequest
  ): Promise<ExtendPermissionResponse> {
    return apiRequest<ExtendPermissionResponse>(
      `/users/${userId}/embedded-permissions/extend`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  },

  // Cleanup expired embedded permissions across all users
  // POST /admin/embedded-permissions/cleanup-expired
  async cleanupExpired(
    request: CleanupExpiredRequest = {}
  ): Promise<CleanupResponse> {
    return apiRequest<CleanupResponse>(
      '/embedded-permissions/cleanup-expired',
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  },
};

// Helper functions for working with embedded permissions
export const EmbeddedPermissionHelpers = {
  // Parse permission string to extract base permission and timestamp
  parsePermission(permission: string): { base: string; timestamp?: number } {
    const parts = permission.split(':');
    
    if (parts.length >= 4) {
      const lastPart = parts[parts.length - 1];
      const timestamp = parseInt(lastPart, 10);
      
      if (!isNaN(timestamp)) {
        return {
          base: parts.slice(0, -1).join(':'),
          timestamp
        };
      }
    }
    
    return { base: permission };
  },

  // Create embedded permission string with timestamp
  createEmbeddedPermission(basePermission: string, expiryTimestamp: number): string {
    return `${basePermission}:${expiryTimestamp}`;
  },

  // Check if permission is expired
  isExpired(permission: string): boolean {
    const { timestamp } = this.parsePermission(permission);
    if (!timestamp) return false; // Permanent permissions never expire
    
    return Date.now() / 1000 > timestamp;
  },

  // Get time remaining until expiry (in milliseconds)
  getTimeRemaining(permission: string): number | null {
    const { timestamp } = this.parsePermission(permission);
    if (!timestamp) return null; // Permanent permissions
    
    return Math.max(0, (timestamp * 1000) - Date.now());
  },

  // Format time remaining as human readable string
  formatTimeRemaining(permission: string): string {
    const timeRemaining = this.getTimeRemaining(permission);
    if (timeRemaining === null) return 'Never expires';
    if (timeRemaining === 0) return 'Expired';
    
    const days = Math.floor(timeRemaining / (1000 * 60 * 60 * 24));
    const hours = Math.floor((timeRemaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((timeRemaining % (1000 * 60 * 60)) / (1000 * 60));
    
    if (days > 0) {
      return `${days} day${days !== 1 ? 's' : ''}`;
    } else if (hours > 0) {
      return `${hours} hour${hours !== 1 ? 's' : ''}`;
    } else {
      return `${minutes} minute${minutes !== 1 ? 's' : ''}`;
    }
  },

  // Calculate health status based on time remaining
  getHealthStatus(permission: string): 'healthy' | 'expiring' | 'expired' {
    const timeRemaining = this.getTimeRemaining(permission);
    if (timeRemaining === null) return 'healthy'; // Permanent permissions are always healthy
    if (timeRemaining === 0) return 'expired';
    
    const hoursRemaining = timeRemaining / (1000 * 60 * 60);
    if (hoursRemaining <= 24) return 'expiring'; // Less than 24 hours
    
    return 'healthy';
  },

  // Generate permission health score (0-100)
  calculateHealthScore(permissions: string[]): number {
    if (permissions.length === 0) return 100;
    
    const scores = permissions.map(perm => {
      const status = this.getHealthStatus(perm);
      switch (status) {
        case 'healthy': return 100;
        case 'expiring': return 50;
        case 'expired': return 0;
        default: return 100;
      }
    });
    
    return Math.round(scores.reduce((sum, score) => sum + score, 0) / scores.length);
  }
};

export { EmbeddedPermissionError };