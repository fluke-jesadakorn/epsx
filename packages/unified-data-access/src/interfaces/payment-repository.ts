import { BaseRepository, ListResult } from "./base-repository";

// Payment domain types
export interface Payment {
  id: string;
  userId: string;
  amount: number;
  currency: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled' | 'refunded';
  paymentMethod: string;
  description?: string;
  metadata?: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
}

export interface Subscription {
  id: string;
  userId: string;
  planId: string;
  status: 'active' | 'cancelled' | 'expired' | 'past_due';
  currentPeriodStart: Date;
  currentPeriodEnd: Date;
  cancelAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface PaymentPlan {
  id: string;
  name: string;
  description?: string;
  amount: number;
  currency: string;
  interval: 'month' | 'year' | 'one_time';
  features: string[];
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface CreatePaymentInput {
  userId: string;
  amount: number;
  currency: string;
  paymentMethod: string;
  description?: string;
  metadata?: Record<string, any>;
}

export interface UpdatePaymentInput {
  status?: 'pending' | 'completed' | 'failed' | 'cancelled' | 'refunded';
  metadata?: Record<string, any>;
}

export interface PaymentFilters {
  userId?: string;
  status?: 'pending' | 'completed' | 'failed' | 'cancelled' | 'refunded';
  paymentMethod?: string;
  amountMin?: number;
  amountMax?: number;
  createdAfter?: Date;
  createdBefore?: Date;
}

export interface SubscriptionFilters {
  userId?: string;
  planId?: string;
  status?: 'active' | 'cancelled' | 'expired' | 'past_due';
}

// Extended payment repository interface
export interface PaymentRepository extends BaseRepository<Payment, string, CreatePaymentInput, UpdatePaymentInput> {
  // Payment-specific operations
  findByUserId(userId: string): Promise<Payment[]>;
  findByStatus(status: Payment['status']): Promise<Payment[]>;
  updatePaymentStatus(id: string, status: Payment['status']): Promise<Payment>;
  
  // Transaction history
  getPaymentHistory(userId: string, filters?: PaymentFilters): Promise<ListResult<Payment>>;
  
  // Subscription management
  getUserSubscription(userId: string): Promise<Subscription | null>;
  createSubscription(data: Omit<Subscription, 'id' | 'createdAt' | 'updatedAt'>): Promise<Subscription>;
  updateSubscription(id: string, data: Partial<Subscription>): Promise<Subscription>;
  cancelSubscription(id: string, cancelAt?: Date): Promise<Subscription>;
  
  // Payment plan management
  getPaymentPlans(): Promise<PaymentPlan[]>;
  getPaymentPlan(id: string): Promise<PaymentPlan | null>;
  
  // Analytics and reporting
  getRevenueAnalytics(startDate: Date, endDate: Date): Promise<{
    totalRevenue: number;
    transactionCount: number;
    averageTransactionValue: number;
    revenueByMethod: Record<string, number>;
  }>;
  
  // Filtered operations
  list(filters?: PaymentFilters): Promise<ListResult<Payment>>;
  listSubscriptions(filters?: SubscriptionFilters): Promise<ListResult<Subscription>>;
}