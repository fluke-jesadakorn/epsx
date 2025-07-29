'use server';

import { 
  withServerAction,
  createServerAction, 
  createAuthenticatedAction,
  CommonSchemas,
  type ActionResult 
} from '../core/action-wrapper';
import { serverGet, serverPost } from '../core/enhanced-request';
import { z } from 'zod';
import type { 
  CreatePaymentRequest,
  CreatePaymentResponse,
  PaymentStatusResponse,
  PaymentPlan,
  UserSubscription
} from '@epsx/types';

// Enhanced Payment Schemas
const CreatePaymentSchema = z.object({
  planId: z.string().min(1, 'Plan ID is required'),
  paymentMethod: z.string().min(1, 'Payment method is required'),
  billingAddress: z.object({
    line1: z.string().min(1, 'Address line 1 is required'),
    line2: z.string().optional(),
    city: z.string().min(1, 'City is required'),
    state: z.string().min(1, 'State is required'),
    postalCode: z.string().min(1, 'Postal code is required'),
    country: z.string().min(2, 'Country is required'),
  }).optional(),
  couponCode: z.string().optional()
});

const PaymentStatusSchema = z.object({
  paymentIntentId: z.string(),
  subscriptionId: z.string(),
  status: z.enum(['PENDING', 'SUCCEEDED', 'FAILED', 'CANCELLED', 'REFUNDED']),
  amount: z.number().nonnegative(),
  currency: z.string().length(3),
  nextBillingDate: z.date().optional()
});

const PaymentPlanSchema = z.object({
  id: z.string(),
  tier: z.enum(['BRONZE', 'SILVER', 'GOLD', 'PLATINUM']),
  name: z.string(),
  price: z.number().nonnegative(),
  currency: z.string().length(3),
  features: z.array(z.string()),
  duration: z.number().positive(),
  numericLevel: z.number().nonnegative(),
  color: z.string()
});

// Enhanced Payment Actions
export const enhancedCreatePayment = createAuthenticatedAction(
  'payments.createPayment',
  async (data: CreatePaymentRequest, context) => {
    const result = await serverPost('/api/v1/payments/create', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
    return result;
  },
  {
    validateInput: CreatePaymentSchema,
    logLevel: 'info'
  }
);

export const enhancedGetPaymentStatus = createAuthenticatedAction(
  'payments.getPaymentStatus',
  async (paymentIntentId: string, context) => {
    return await serverGet(`/api/v1/payments/status/${paymentIntentId}`, undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.string().min(1, 'Payment intent ID is required'),
    validateOutput: PaymentStatusSchema,
    logLevel: 'info'
  }
);

export const enhancedValidatePayment = createAuthenticatedAction(
  'payments.validatePayment',
  async (data: { paymentId: string; signature?: string }, context) => {
    return await serverPost(`/api/v1/payments/${data.paymentId}/validate`, 
      data.signature ? { signature: data.signature } : {}, 
      {
        action: context.action,
        userId: context.userId,
        requestId: context.requestId
      }
    );
  },
  {
    validateInput: z.object({
      paymentId: z.string().min(1, 'Payment ID is required'),
      signature: z.string().optional()
    }),
    logLevel: 'warn' // Higher log level for payment validation
  }
);

export const enhancedGetPaymentPlans = createServerAction(
  'payments.getPaymentPlans',
  async (_, context) => {
    return await serverGet('/api/v1/payments/plans', undefined, {
      action: context.action,
      requestId: context.requestId
    });
  }
);

export const enhancedGetPaymentPlan = withServerAction(
  'payments.getPaymentPlan',
  async (planId: string, context) => {
    return await serverGet(`/api/v1/payments/plans/${planId}`, undefined, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.string().min(1, 'Plan ID is required'),
    validateOutput: PaymentPlanSchema
  }
);

export const enhancedGetUserSubscription = createAuthenticatedAction(
  'payments.getUserSubscription',
  async (_, context) => {
    return await serverGet('/api/v1/payments/subscription', undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

export const enhancedCancelSubscription = createAuthenticatedAction(
  'payments.cancelSubscription',
  async (subscriptionId: string, context) => {
    return await serverPost(`/api/v1/payments/subscription/${subscriptionId}/cancel`, undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.string().min(1, 'Subscription ID is required'),
    logLevel: 'warn' // Higher log level for subscription changes
  }
);

export const enhancedUpdateSubscription = createAuthenticatedAction(
  'payments.updateSubscription',
  async (data: { subscriptionId: string; planId: string }, context) => {
    return await serverPost(`/api/v1/payments/subscription/${data.subscriptionId}`, 
      { planId: data.planId }, 
      {
        action: context.action,
        userId: context.userId,
        requestId: context.requestId
      }
    );
  },
  {
    validateInput: z.object({
      subscriptionId: z.string().min(1, 'Subscription ID is required'),
      planId: z.string().min(1, 'Plan ID is required')
    }),
    logLevel: 'info'
  }
);

export const enhancedGetPaymentHistory = createAuthenticatedAction(
  'payments.getPaymentHistory',
  async (params: { page?: number; limit?: number }, context) => {
    const queryParams = {
      page: params.page || 1,
      limit: params.limit || 10
    };
    
    return await serverGet('/api/v1/payments/history', queryParams, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.object({
      page: z.number().positive().optional(),
      limit: z.number().positive().max(100).optional()
    })
  }
);

export const enhancedDownloadInvoice = createAuthenticatedAction(
  'payments.downloadInvoice',
  async (invoiceId: string, context) => {
    return await serverGet(`/api/v1/payments/invoice/${invoiceId}/download`, undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.string().min(1, 'Invoice ID is required'),
    logLevel: 'info'
  }
);

export const enhancedApplyPromoCode = createAuthenticatedAction(
  'payments.applyPromoCode',
  async (code: string, context) => {
    return await serverPost('/api/v1/payments/promo', { code }, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.string().min(1, 'Promo code is required'),
    logLevel: 'info'
  }
);

// Legacy payment functions (deprecated but maintained for backward compatibility)
export const enhancedCreateLegacyPayment = createAuthenticatedAction(
  'payments.createLegacyPayment',
  async (data: { amount: number; currency: string; description?: string; orderNo: string }, context) => {
    return await serverPost('/api/v1/payments/musepay/create', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: CommonSchemas.paymentData,
    logLevel: 'info'
  }
);

export const enhancedInitQRPayment = createAuthenticatedAction(
  'payments.initQRPayment',
  async (data: { amount: number; currency: string; orderNo: string }, context) => {
    return await serverPost('/api/v1/payments/qr/init', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.object({
      amount: z.number().positive(),
      currency: z.string().length(3),
      orderNo: z.string().min(1)
    }),
    logLevel: 'info'
  }
);

// Type exports for enhanced actions
export type EnhancedCreatePaymentResult = ActionResult<CreatePaymentResponse>;
export type EnhancedPaymentStatusResult = ActionResult<PaymentStatusResponse>;
export type EnhancedPaymentPlansResult = ActionResult<PaymentPlan[]>;
export type EnhancedUserSubscriptionResult = ActionResult<UserSubscription>;