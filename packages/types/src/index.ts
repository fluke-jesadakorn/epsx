// ============================================================================
// CONSOLIDATED TYPES - Single Entry Point for All Types
// ============================================================================

// Core Domain Types (consolidated from all domains)
export * from './domains/api';
export * from './domains/user';
export * from './domains/payment';
export * from './domains/analytics';
export * from './domains/permissions';
export * from './domains/common';

// Validation schemas
export * from './schemas';

// Legacy exports (backward compatibility)
export * from './chat';
export * from './auth/roles';
export * from './auth/request';
export * from './pagination';

// ============================================================================
// IMPORT GUIDANCE - Use these imports to minimize dependencies:
// 
// Basic types:     import type { User, AdminUser } from '@epsx/types';
// Payment types:   import type { PaymentTier, PaymentPlan } from '@epsx/types';
// API types:       import type { ApiResponse, RequestConfig } from '@epsx/types';
// Auth types:      import type { AuthenticatedUser, UserRole } from '@epsx/types';
// All types:       import type * as Types from '@epsx/types'; (discouraged)
// ============================================================================