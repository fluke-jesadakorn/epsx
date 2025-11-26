/**
 * ADMIN FRONTEND PAYMENT TYPES - MIGRATED TO SHARED
 * All payment types moved to shared/types/payment with compatibility layer
 * This file now re-exports shared types for backward compatibility
 */

// Re-export everything from shared payment types
export * from '../../../shared/types/payment';

// Import for local re-export with legacy names
import type { 
  CreatePaymentRequest as SharedCreatePaymentRequest,
  CreatePaymentResponse as SharedCreatePaymentResponse,
  PaymentResponse as SharedPaymentResponse,
  AssetInfo as SharedAssetInfo,
  UserSubscription as SharedUserSubscription,
  PermissionTemplateName as SharedPermissionTemplateName
} from '../../../shared/types/payment';

// Re-export with exact same names for backward compatibility
export type CreatePaymentRequest = SharedCreatePaymentRequest;
export type CreatePaymentResponse = SharedCreatePaymentResponse; 
export type PaymentResponse = SharedPaymentResponse;
export type AssetInfo = SharedAssetInfo;
export type UserSubscription = SharedUserSubscription;
export type PermissionTemplateName = SharedPermissionTemplateName;

// Legacy compatibility (admin-frontend was importing PermissionTemplateName from permission-templates)
export type { PermissionTemplateName } from '../permission-templates';