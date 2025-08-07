// ============================================================================
// CONSOLIDATED API CLIENT - Single Entry Point for All API Operations
// ============================================================================

// Primary API client (most common usage)
export { ApiClientFactory, apiClient } from './ApiClientFactory';

// Specialized clients (for advanced usage)
export { PaymentClient } from './clients/PaymentClient';
export { AnalyticsClient } from './clients/AnalyticsClient';
export { PermissionsClient } from './clients/PermissionsClient';

// Base classes (for custom implementations)
export { BaseHttpClient } from './base/BaseHttpClient';
export { ClientApiClient } from './base/ClientApiClient';
export { ServerApiClient } from './base/ServerApiClient';

// Utilities
export { CookieManager } from './cookie-manager';
export { isApiError, isApiSuccess } from './types';

// Server-specific functions
export { 
  serverGetAdminProfile, 
  serverGetCasbinPolicies, 
  serverGetAdminPermissionProfiles,
  serverGetAdminUsers,
  serverSetUserRole,
  serverGetUserStats
} from './api-server';
export { ApiClient } from './api-client'; // Legacy compatibility

// Legacy createApiClient function for backward compatibility
import { ApiClient } from './api-client';
export const createApiClient = (baseUrl?: string): ApiClient => new ApiClient(baseUrl);

// Essential API-specific types (from local types.ts)
export type {
  ApiResponse,
  RequestConfig,
  PaginatedResponse,
  CountResponse,
  StockFinancialData,
  AdminUser,
  PermissionProfile,
  AssignmentResult,
  ActionResult
} from './types';

// Import only essential types from @epsx/types (to minimize bundle)
export type {
  User,
  AuthenticatedUser,
  PaymentTier,
  PaymentPlan
} from '@epsx/types';

// ============================================================================
// IMPORT GUIDANCE - Use these imports to minimize dependencies:
// 
// Standard usage:  import { apiClient } from '@epsx/api-client';
// Factory:         import { ApiClientFactory } from '@epsx/api-client';
// Specific client: import { AuthClient } from '@epsx/api-client';
// Type guards:     import { isApiError, isApiSuccess } from '@epsx/api-client';
// Types only:      import type { ApiResponse } from '@epsx/api-client';
// 
// Avoid: import * from '@epsx/api-client' (pulls in all types)
// ============================================================================