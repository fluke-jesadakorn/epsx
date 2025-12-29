/**
 * CONSOLIDATED PAYMENT TYPES
 * Unified payment type definitions shared across admin-frontend and frontend
 * Replaces duplicate payment.d.ts files in both applications
 */

// ============================================================================
// PERMISSION TEMPLATE TYPES
// ============================================================================

export type PermissionTemplateName =
  | 'FREE'
  | 'BASIC'
  | 'PRO'
  | 'ENTERPRISE'
  | 'WHALE'
  | 'CUSTOM';

// ============================================================================
// PAYMENT CONTEXT TYPES (V2 Dynamic Payments)
// ============================================================================

/**
 * Context types for dynamic payments
 * Maps to smart contract ContextType enum
 */
export type PaymentContextType = 'plan' | 'group' | 'product' | 'campaign' | 'custom';

/**
 * Context type to enum value mapping (matches smart contract)
 */
export const CONTEXT_TYPE_VALUES: Record<PaymentContextType, number> = {
  plan: 0,
  group: 1,
  product: 2,
  campaign: 3,
  custom: 4,
};

/**
 * Payment context entity - represents a dynamic payment link
 */
export interface PaymentContext {
  id: string;
  context_type: PaymentContextType;
  context_id?: string;           // UUID of plan, group, etc.
  slug: string;                   // URL-friendly identifier
  name: string;                   // Display name
  description?: string;
  amount: number;
  currency: string;
  expires_at?: string;            // Default: 24 hours from creation
  max_uses?: number;              // Default: null (multi-use)
  current_uses: number;
  metadata: Record<string, any>;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * Request to create a new payment link
 */
export interface CreatePaymentLinkRequest {
  context_type: PaymentContextType;
  context_id?: string;            // Required for plan/group, optional for custom
  slug?: string;                  // Auto-generated if not provided
  name: string;
  description?: string;
  amount: number;
  currency: string;
  expires_at?: string;            // Default: 24 hours from creation
  max_uses?: number;              // Default: null (multi-use)
  metadata?: Record<string, any>;
}

/**
 * Request to update a payment link
 */
export interface UpdatePaymentLinkRequest {
  name?: string;
  description?: string;
  amount?: number;
  currency?: string;
  expires_at?: string | null;
  max_uses?: number | null;
  is_active?: boolean;
  metadata?: Record<string, any>;
}

/**
 * Payment link with full URL
 */
export interface PaymentLink extends PaymentContext {
  url: string;                    // Full payment URL
  qr_code?: string;              // Base64 QR code image
}

// ============================================================================
// PAYMENT REQUEST/RESPONSE TYPES
// ============================================================================

export interface CreatePaymentRequest {
  currency: string;
  amount: string;
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  permission_template: PermissionTemplateName;
  notify_url?: string;
}

export interface CreatePaymentResponse extends PaymentResponse {
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  order_no: string;
  order_amount: number;
  receive_address?: string;
  checkout_url?: string;
}

export interface PaymentResponse {
  id: string;
  amount: number;
  currency: string;
  status: "Pending" | "Processing" | "Succeeded" | "Failed" | "Cancelled" | "Expired" | "RequiresAction";
  created_at: string;
  updated_at: string;
  expiration_date: string;
  permission_template: PermissionTemplateName;
  permissions: string[];
  display_tier?: string;
  qr_code?: string;
  checkout_url?: string;
  payment_method: string;
  retry_count: number;
  error_message?: string;
}

// ============================================================================
// ASSET INFORMATION TYPES
// ============================================================================

export interface AssetInfo {
  currency: string;
  name: string;
  symbol: string;
  decimals: number;
  contract_address?: string;
  chain?: string;
  depositThreshold?: number;
  addressFormat?: string;
}

// ============================================================================
// SUBSCRIPTION TYPES (from both apps)
// ============================================================================

export interface UserSubscription {
  id: string;
  user_id: string;
  plan_id: string;
  plan_name: string;
  status: string;
  started_at?: string;
  expires_at?: string;
  created_at: string;
  updated_at: string;
  auto_renew: boolean;
  current_usage: Record<string, any>;
  quota_limits: Record<string, any>;
  metadata?: Record<string, any>;
  access_context: string;
  api_key?: string;
  api_key_name?: string;
  last_billed_at?: string;
  next_billing_date?: string;
}

// ============================================================================
// PAYMENT VALIDATION TYPES
// ============================================================================

export interface PaymentValidationResult {
  valid: boolean;
  errors?: string[];
  warnings?: string[];
}

// ============================================================================
// PAYMENT STATUS TYPES
// ============================================================================

export type PaymentStatusType =
  | "Pending"
  | "Processing"
  | "Succeeded"
  | "Failed"
  | "Cancelled"
  | "Expired"
  | "RequiresAction";

export interface PaymentStatusInfo {
  status: PaymentStatusType;
  description: string;
  canRetry: boolean;
  nextAction?: string;
}

// ============================================================================
// PAYMENT METHOD TYPES
// ============================================================================

export type PaymentMethodType = 'on_line' | 'on_chain';

export interface PaymentMethod {
  type: PaymentMethodType;
  name: string;
  description: string;
  supported_currencies: string[];
  enabled: boolean;
}

// ============================================================================
// TYPE UTILITIES AND GUARDS
// ============================================================================

export function isSuccessfulPayment(payment: PaymentResponse): boolean {
  return payment.status === "Succeeded";
}

export function isPendingPayment(payment: PaymentResponse): boolean {
  return payment.status === "Pending" || payment.status === "Processing";
}

export function isFailedPayment(payment: PaymentResponse): boolean {
  return ["Failed", "Cancelled", "Expired"].includes(payment.status);
}

export function getPaymentStatusColor(status: PaymentStatusType): string {
  switch (status) {
    case "Succeeded":
      return "text-green-600 dark:text-green-400";
    case "Pending":
    case "Processing":
      return "text-yellow-600 dark:text-yellow-400";
    case "Failed":
    case "Cancelled":
    case "Expired":
      return "text-red-600 dark:text-red-400";
    case "RequiresAction":
      return "text-blue-600 dark:text-blue-400";
    default:
      return "text-gray-600 dark:text-gray-400";
  }
}

// ============================================================================
// EXPORT COMPATIBILITY ALIASES
// ============================================================================

// For backward compatibility with existing imports
export type { PaymentResponse as Payment, CreatePaymentRequest as PaymentRequest, UserSubscription as Subscription };
