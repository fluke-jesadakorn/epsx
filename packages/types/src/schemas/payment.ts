import { z } from 'zod';

export const PaymentTierSchema = z.enum(['BRONZE', 'SILVER', 'GOLD', 'PLATINUM']);

export const PaymentLimitsSchema = z.object({
  requestsPerMinute: z.number().positive(),
  requestsPerDay: z.number().positive(),
  maxRankings: z.number().positive(),
  maxFileSize: z.number().positive(),
});

export const PaymentPlanSchema = z.object({
  id: z.string(),
  tier: PaymentTierSchema,
  name: z.string(),
  price: z.number().nonnegative(),
  currency: z.string(),
  features: z.array(z.string()),
  apiLimits: PaymentLimitsSchema,
  duration: z.number().positive(),
  numericLevel: z.number().nonnegative(),
  color: z.string(),
});

export const UserSubscriptionSchema = z.object({
  userId: z.string().uuid(),
  tier: PaymentTierSchema,
  subscriptionId: z.string().optional(),
  validUntil: z.date().optional(),
  isActive: z.boolean(),
  features: z.array(z.string()),
  lastPaymentDate: z.date().optional(),
  paymentMethod: z.string().optional(),
  transactionId: z.string().optional(),
  amount: z.number().nonnegative().optional(),
});

export const BillingAddressSchema = z.object({
  line1: z.string(),
  line2: z.string().optional(),
  city: z.string(),
  state: z.string(),
  postalCode: z.string(),
  country: z.string(),
});

export const CreatePaymentRequestSchema = z.object({
  planId: z.string(),
  paymentMethod: z.string(),
  billingAddress: BillingAddressSchema.optional(),
  couponCode: z.string().optional(),
});

export const PaymentStatusSchema = z.enum(['PENDING', 'SUCCEEDED', 'FAILED', 'CANCELLED', 'REFUNDED']);

export const CreatePaymentResponseSchema = z.object({
  paymentIntentId: z.string(),
  clientSecret: z.string(),
  subscriptionId: z.string(),
  status: PaymentStatusSchema,
});

export const PaymentStatusResponseSchema = z.object({
  paymentIntentId: z.string(),
  subscriptionId: z.string(),
  status: PaymentStatusSchema,
  amount: z.number().nonnegative(),
  currency: z.string(),
  nextBillingDate: z.date().optional(),
});