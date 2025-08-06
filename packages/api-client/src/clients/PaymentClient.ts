import { BaseHttpClient } from '../base/BaseHttpClient';

import type {
  CreatePaymentRequest,
  CreatePaymentResponse,
  PaymentStatusResponse,
  PaymentPlan,
  UserSubscription,
  ApiResponse,
  PaginatedResponse,
} from '@epsx/types';

export class PaymentClient extends BaseHttpClient {
  async getPlans(): Promise<ApiResponse<PaymentPlan[]>> {
    return this.get<PaymentPlan[]>('/api/payments/plans');
  }

  async getPlan(id: string): Promise<ApiResponse<PaymentPlan>> {
    return this.get<PaymentPlan>(`/api/payments/plans/${id}`);
  }

  async createPayment(data: CreatePaymentRequest): Promise<ApiResponse<CreatePaymentResponse>> {
    return this.post<CreatePaymentResponse>('/api/payments/create', data);
  }

  async getPaymentStatus(paymentIntentId: string): Promise<ApiResponse<PaymentStatusResponse>> {
    return this.get<PaymentStatusResponse>(`/api/payments/status/${paymentIntentId}`);
  }

  async getUserSubscription(): Promise<ApiResponse<UserSubscription>> {
    return this.get<UserSubscription>('/api/payments/subscription');
  }

  async cancelSubscription(subscriptionId: string): Promise<ApiResponse<void>> {
    return this.post<void>(`/api/payments/subscription/${subscriptionId}/cancel`);
  }

  async updateSubscription(subscriptionId: string, planId: string): Promise<ApiResponse<UserSubscription>> {
    return this.put<UserSubscription>(`/api/payments/subscription/${subscriptionId}`, { planId });
  }

  async getPaymentHistory(page: number = 1, limit: number = 10): Promise<ApiResponse<PaginatedResponse<unknown>>> {
    return this.get<PaginatedResponse<unknown>>(`/api/payments/history?page=${page}&limit=${limit}`);
  }

  async downloadInvoice(invoiceId: string): Promise<ApiResponse<Blob>> {
    return this.get<Blob>(`/api/payments/invoice/${invoiceId}/download`);
  }

  async applyPromoCode(code: string): Promise<ApiResponse<{ discount: number; validUntil: Date }>> {
    return this.post<{ discount: number; validUntil: Date }>('/api/payments/promo', { code });
  }
}