// Main API client factory
export { ApiClientFactory, apiClient } from './ApiClientFactory';

// Individual clients for custom usage
export { AuthClient } from './clients/AuthClient';
export { PaymentClient } from './clients/PaymentClient';
export { AnalyticsClient } from './clients/AnalyticsClient';
export { PermissionsClient } from './clients/PermissionsClient';

// Base classes
export { BaseHttpClient } from './base/BaseHttpClient';
export { ClientApiClient } from './base/ClientApiClient';
export { ServerApiClient } from './base/ServerApiClient';

// Legacy exports (cookie manager still needed)
export { CookieManager } from './cookie-manager';

// Re-export types for convenience
export * from '@epsx/types';

// Legacy API client (for backward compatibility)
export { ApiClient } from './api-client';

// Server API functions
export { serverGetAdminProfile } from './api-server';

// Type guards and utilities
export { isApiError, isApiSuccess } from './types';