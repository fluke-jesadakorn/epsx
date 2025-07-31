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
import { 
  CreatePaymentRequestSchema,
  CreatePaymentResponseSchema,
  PaymentStatusResponseSchema,
  PaymentPlanSchema,
  UserSubscriptionSchema
} from '@epsx/types';

// Using imported schemas from @epsx/types

// Enhanced Payment Actions
export const enhancedCreatePayment = createAuthenticatedAction(
  'payments.createPayment',
  async (data: z.infer<typeof CreatePaymentRequestSchema>, context) => {
    const result = await serverPost('/api/v1/payments/create', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
    return result;
  },
  {
    validateInput: CreatePaymentRequestSchema,
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
    validateOutput: PaymentStatusResponseSchema,
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

// Backward compatibility exports (legacy simple versions)
export const createPayment = enhancedCreatePayment;
export const getPaymentStatus = enhancedGetPaymentStatus;
export const validatePayment = enhancedValidatePayment;
export const getPaymentPlans = enhancedGetPaymentPlans;
export const getPaymentPlan = enhancedGetPaymentPlan;
export const getUserSubscription = enhancedGetUserSubscription;
export const cancelSubscription = enhancedCancelSubscription;
export const updateSubscription = enhancedUpdateSubscription;
export const getPaymentHistory = enhancedGetPaymentHistory;
export const downloadInvoice = enhancedDownloadInvoice;
export const applyPromoCode = enhancedApplyPromoCode;
export const createLegacyPayment = enhancedCreateLegacyPayment;
export const initQRPayment = enhancedInitQRPayment;

// Additional exports needed by index.ts
export const getTransactionHistory = enhancedGetPaymentHistory; // Transaction history is same as payment history
export const getPlanDetails = enhancedGetPaymentPlan; // Plan details is same as get payment plan

// Type exports for backward compatibility
export type PaymentStatus = 'pending' | 'completed' | 'failed' | 'cancelled';
export type PaymentTransaction = {
  id: string;
  amount: number;
  currency: string;
  status: PaymentStatus;
  createdAt: string;
};

// Type exports for enhanced actions
export type EnhancedCreatePaymentResult = ActionResult<z.infer<typeof CreatePaymentResponseSchema>>;
export type EnhancedPaymentStatusResult = ActionResult<z.infer<typeof PaymentStatusResponseSchema>>;
export type EnhancedPaymentPlansResult = ActionResult<z.infer<typeof PaymentPlanSchema>[]>;
export type EnhancedUserSubscriptionResult = ActionResult<z.infer<typeof UserSubscriptionSchema>>;