// ============================================================================
// BACKEND PERMISSION AUTHORITY CLIENT (Phase 2.1)
// THE SINGLE SOURCE OF TRUTH for all permission validation
// Replaces ALL local permission validation with backend API calls
// ============================================================================

import { BACKEND_URL, NEXT_PUBLIC_BACKEND_URL } from '@/config/env';

// ============================================================================
// BACKEND PERMISSION AUTHORITY TYPES
// These match the backend permission error system exactly
// ============================================================================

export interface PermissionValidationRequest {
  user_id: string;
  permission: string;
  resource_path?: string;
  context?: Record<string, any>;
}

export interface PermissionValidationResponse {
  granted: boolean;
  reason?: string;
  expires_at?: string; // ISO timestamp
  usage_count?: number;
  usage_limit?: number;
  next_refresh?: string; // ISO timestamp
}

export interface BulkPermissionRequest {
  user_id: string;
  permissions: Array<{
    permission: string;
    resource_path?: string;
  }>;
}

export interface BulkPermissionResponse {
  results: Array<{
    permission: string;
    granted: boolean;
    reason?: string;
  }>;
  user_id: string;
  validated_at: string;
}

export interface UserPermissionsResponse {
  user_id: string;
  permissions: Array<{
    permission: string;
    granted: boolean;
    expires_at?: string;
    usage_count?: number;
    usage_limit?: number;
  }>;
  tier_info?: {
    current_tier: string;
    tier_permissions: string[];
  };
  last_updated: string;
}

// Backend permission error response structure
export interface PermissionErrorResponse {
  error_type: string;
  status_code: number;
  message: string;
  details: {
    permission?: string;
    user_id?: string;
    resource_path?: string;
    current_tier?: string;
    required_tier?: string;
    usage_info?: {
      current_usage: number;
      limit: number;
      period: string;
      reset_at?: string;
      usage_percentage: number;
    };
    upgrade_info?: {
      current_tier: string;
      required_tier: string;
      upgrade_url?: string;
      benefits: string[];
      trial_available: boolean;
    };
  };
  suggested_actions: string[];
  user_message: string;
  timestamp: string;
  error_id: string;
  context: Record<string, any>;
}

// ============================================================================
// BACKEND PERMISSION AUTHORITY CLIENT CLASS
// THE SINGLE SOURCE OF TRUTH for all permission decisions
// ============================================================================

export class BackendPermissionAuthorityClient {
  private baseUrl: string;
  private authHeaders: () => Record<string, string>;

  constructor() {
    // Use appropriate backend URL based on environment
    this.baseUrl = typeof window !== 'undefined' 
      ? (NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080')
      : (BACKEND_URL || 'http://localhost:8080');
      
    // Setup authentication headers function
    this.authHeaders = () => {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        'x-api-version': '1.0',
        'x-client-version': 'frontend-v1.0',
      };

      if (typeof window !== 'undefined') {
        // Try to get authentication from various sources
        const walletAddress = localStorage.getItem('wallet_address');
        const signature = localStorage.getItem('wallet_signature');
        const authToken = localStorage.getItem('auth_token');

        // Web3 authentication (preferred)
        if (walletAddress && signature) {
          headers['x-wallet-address'] = walletAddress;
          headers['x-signature'] = signature;
          headers['x-chain-id'] = localStorage.getItem('chain_id') || '56';
          headers['x-timestamp'] = localStorage.getItem('auth_timestamp') || Date.now().toString();
        }
        // Fallback to Bearer token
        else if (authToken) {
          headers['Authorization'] = `Bearer ${authToken}`;
        }
      }

      return headers;
    };
  }

  // ⚡ CRITICAL: Real-time permission validation - THE AUTHORITY
  async validatePermission(
    userId: string,
    permission: string,
    resourcePath?: string,
    context?: Record<string, any>
  ): Promise<PermissionValidationResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/permissions/validate`, {
        method: 'POST',
        headers: this.authHeaders(),
        body: JSON.stringify({
          user_id: userId,
          permission,
          resource_path: resourcePath,
          context: context || {},
        } as PermissionValidationRequest),
        credentials: 'include',
      });

      if (!response.ok) {
        await this.handlePermissionError(response);
      }

      return await response.json() as PermissionValidationResponse;
    } catch (error) {
      console.error('Permission validation failed:', error);
      throw new PermissionValidationError('Permission validation service unavailable', {
        permission,
        userId,
        originalError: error,
      });
    }
  }

  // ⚡ CRITICAL: Bulk permission validation for performance
  async validateBulkPermissions(
    userId: string,
    permissions: Array<{ permission: string; resource_path?: string }>
  ): Promise<BulkPermissionResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/permissions/validate-bulk`, {
        method: 'POST',
        headers: this.authHeaders(),
        body: JSON.stringify({
          user_id: userId,
          permissions,
        } as BulkPermissionRequest),
        credentials: 'include',
      });

      if (!response.ok) {
        await this.handlePermissionError(response);
      }

      return await response.json() as BulkPermissionResponse;
    } catch (error) {
      console.error('Bulk permission validation failed:', error);
      throw new PermissionValidationError('Bulk permission validation service unavailable', {
        permissions,
        userId,
        originalError: error,
      });
    }
  }

  // ⚡ CRITICAL: Get user's effective permissions
  async getUserPermissions(userId: string): Promise<UserPermissionsResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/permissions/user/${userId}`, {
        method: 'GET',
        headers: this.authHeaders(),
        credentials: 'include',
      });

      if (!response.ok) {
        await this.handlePermissionError(response);
      }

      return await response.json() as UserPermissionsResponse;
    } catch (error) {
      console.error('Get user permissions failed:', error);
      throw new PermissionValidationError('User permissions service unavailable', {
        userId,
        originalError: error,
      });
    }
  }

  // Handle structured permission errors from backend
  private async handlePermissionError(response: Response): Promise<never> {
    try {
      const errorData = await response.json() as PermissionErrorResponse;
      
      // Create specific error types based on backend error classification
      switch (errorData.error_type) {
        case 'authentication_required':
          throw new AuthenticationRequiredError(errorData);
          
        case 'permission_denied':
          throw new PermissionDeniedError(errorData);
          
        case 'permission_expired':
          throw new PermissionExpiredError(errorData);
          
        case 'usage_limit_exceeded':
          throw new UsageLimitExceededError(errorData);
          
        case 'insufficient_tier':
          throw new InsufficientTierError(errorData);
          
        case 'security_restriction':
          throw new SecurityRestrictionError(errorData);
          
        case 'system_error':
          throw new SystemError(errorData);
          
        default:
          throw new PermissionValidationError(errorData.message, {
            errorType: errorData.error_type,
            errorData,
          });
      }
    } catch (parseError) {
      // If we can't parse the error response, create a generic error
      throw new PermissionValidationError(`Permission validation failed with status ${response.status}`, {
        status: response.status,
        statusText: response.statusText,
        originalError: parseError,
      });
    }
  }
}

// ============================================================================
// PERMISSION ERROR CLASSES
// Structured errors that match backend error system
// ============================================================================

export class PermissionValidationError extends Error {
  public readonly context?: Record<string, any>;
  public readonly isPermissionError = true;

  constructor(message: string, context?: Record<string, any>) {
    super(message);
    this.name = 'PermissionValidationError';
    this.context = context;
  }
}

export class AuthenticationRequiredError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Authentication required');
    this.name = 'AuthenticationRequiredError';
    this.errorData = errorData;
  }
}

export class PermissionDeniedError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;
  public readonly permission?: string;
  public readonly upgradeInfo?: any;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Permission denied');
    this.name = 'PermissionDeniedError';
    this.errorData = errorData;
    this.permission = errorData.details.permission;
    this.upgradeInfo = errorData.details.upgrade_info;
  }
}

export class PermissionExpiredError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;
  public readonly permission?: string;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Permission has expired');
    this.name = 'PermissionExpiredError';
    this.errorData = errorData;
    this.permission = errorData.details.permission;
  }
}

export class UsageLimitExceededError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;
  public readonly usageInfo?: any;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Usage limit exceeded');
    this.name = 'UsageLimitExceededError';
    this.errorData = errorData;
    this.usageInfo = errorData.details.usage_info;
  }
}

export class InsufficientTierError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;
  public readonly currentTier?: string;
  public readonly requiredTier?: string;
  public readonly upgradeInfo?: any;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Insufficient tier level');
    this.name = 'InsufficientTierError';
    this.errorData = errorData;
    this.currentTier = errorData.details.current_tier;
    this.requiredTier = errorData.details.required_tier;
    this.upgradeInfo = errorData.details.upgrade_info;
  }
}

export class SecurityRestrictionError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Access temporarily restricted');
    this.name = 'SecurityRestrictionError';
    this.errorData = errorData;
  }
}

export class SystemError extends PermissionValidationError {
  public readonly errorData: PermissionErrorResponse;
  public readonly retryAfter?: number;

  constructor(errorData: PermissionErrorResponse) {
    super(errorData.user_message || 'Service temporarily unavailable');
    this.name = 'SystemError';
    this.errorData = errorData;
    this.retryAfter = errorData.context?.retry_after;
  }
}

// ============================================================================
// SINGLETON INSTANCE - THE AUTHORITY
// ============================================================================

export const permissionAuthority = new BackendPermissionAuthorityClient();

// ============================================================================
// UTILITY FUNCTIONS FOR COMMON PERMISSION PATTERNS
// ============================================================================

// Quick permission check for single permission
export async function hasPermission(
  userId: string,
  permission: string,
  resourcePath?: string
): Promise<boolean> {
  try {
    const result = await permissionAuthority.validatePermission(userId, permission, resourcePath);
    return result.granted;
  } catch (error) {
    console.error('Permission check failed:', error);
    return false; // Fail closed for security
  }
}

// Check multiple permissions at once
export async function hasAnyPermission(
  userId: string,
  permissions: string[]
): Promise<boolean> {
  try {
    const permissionRequests = permissions.map(permission => ({ permission }));
    const result = await permissionAuthority.validateBulkPermissions(userId, permissionRequests);
    return result.results.some(r => r.granted);
  } catch (error) {
    console.error('Multi-permission check failed:', error);
    return false; // Fail closed for security
  }
}

// Check if user has all required permissions
export async function hasAllPermissions(
  userId: string,
  permissions: string[]
): Promise<boolean> {
  try {
    const permissionRequests = permissions.map(permission => ({ permission }));
    const result = await permissionAuthority.validateBulkPermissions(userId, permissionRequests);
    return result.results.every(r => r.granted);
  } catch (error) {
    console.error('Multi-permission check failed:', error);
    return false; // Fail closed for security
  }
}

// ============================================================================
// MIGRATION HELPERS
// These help transition from old local validation to new backend validation
// ============================================================================

// Legacy permission format converter
export function convertLegacyPermission(legacyPermission: string): string {
  // Convert old permission formats to new structured format
  // e.g., "user_management" -> "admin:users:manage"
  const permissionMap: Record<string, string> = {
    'user_management': 'admin:users:manage',
    'analytics_access': 'epsx:analytics:read',
    'premium_features': 'epsx:premium:access',
    'admin_access': 'admin:general:access',
    // Add more mappings as needed during migration
  };

  return permissionMap[legacyPermission] || legacyPermission;
}

// Batch convert legacy permissions
export function convertLegacyPermissions(legacyPermissions: string[]): string[] {
  return legacyPermissions.map(convertLegacyPermission);
}