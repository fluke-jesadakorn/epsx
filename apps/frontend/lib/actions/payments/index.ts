'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { requireAuth } from '../auth';

// Check if error is a Next.js redirect error
function isRedirectError(error: unknown): boolean {
  if (typeof error !== 'object' || error === null) return false;
  return (
    'digest' in error &&
    typeof error.digest === 'string' &&
    error.digest.startsWith('NEXT_REDIRECT')
  );
}

// ============================================================================
// Payment Server Actions
// ============================================================================

const getClient = () => createApiClient();

/**
 * Create a new payment
 */
export async function createPayment(planId: string, promoCode?: string): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverCreatePayment({ planId, promoCode });

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to create payment');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Create payment error:', error);
    throw error;
  }
}

/**
 * Get payment status
 */
export async function getPaymentStatus(paymentId: string): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverGetPaymentStatus(paymentId);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get payment status');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Get payment status error:', error);
    throw error;
  }
}

/**
 * Validate payment
 */
export async function validatePayment(paymentId: string): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverValidatePayment(paymentId);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to validate payment');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Validate payment error:', error);
    throw error;
  }
}

/**
 * Get available payment plans
 */
export async function getPaymentPlans(): Promise<any[]> {
  try {
    const client = getClient();
    const result = await client.serverGetPaymentPlans();
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get payment plans');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get payment plans error:', error);
    throw error;
  }
}

/**
 * Get specific payment plan
 */
export async function getPaymentPlan(planId: string): Promise<any> {
  try {
    const client = getClient();
    const result = await client.serverGetPaymentPlan(planId);
    
    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get payment plan');
    }
    
    return result.data!;
  } catch (error) {
    console.error('Get payment plan error:', error);
    throw error;
  }
}

/**
 * Get user subscription
 */
export async function getUserSubscription(): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverGetUserSubscription();

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get user subscription');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Get user subscription error:', error);
    throw error;
  }
}

/**
 * Cancel subscription
 */
export async function cancelSubscription(subscriptionId: string): Promise<void> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverCancelSubscription(subscriptionId);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to cancel subscription');
    }
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Cancel subscription error:', error);
    throw error;
  }
}

/**
 * Update subscription
 */
export async function updateSubscription(subscriptionId: string, planId: string): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverUpdateSubscription(subscriptionId, planId);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to update subscription');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Update subscription error:', error);
    throw error;
  }
}

/**
 * Get payment history
 */
export async function getPaymentHistory(): Promise<any[]> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverGetPaymentHistory();

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to get payment history');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Get payment history error:', error);
    throw error;
  }
}

/**
 * Download invoice
 */
export async function downloadInvoice(invoiceId: string): Promise<Blob> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverDownloadInvoice(invoiceId);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to download invoice');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Download invoice error:', error);
    throw error;
  }
}

/**
 * Apply promo code
 */
export async function applyPromoCode(promoCode: string): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverApplyPromoCode(promoCode);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to apply promo code');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Apply promo code error:', error);
    throw error;
  }
}

/**
 * Create legacy payment
 */
export async function createLegacyPayment(paymentData: any): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverCreateLegacyPayment(paymentData);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to create legacy payment');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Create legacy payment error:', error);
    throw error;
  }
}

/**
 * Initialize QR payment
 */
export async function initQRPayment(planId: string): Promise<any> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.serverInitQRPayment(planId);

    if (isApiError(result)) {
      throw new Error(result.message || 'Failed to initialize QR payment');
    }

    return result.data!;
  } catch (error) {
    if (isRedirectError(error)) throw error;
    console.error('Initialize QR payment error:', error);
    throw error;
  }
}