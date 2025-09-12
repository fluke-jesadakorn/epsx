/**
 * Client-side payment service
 * This service now uses server actions directly instead of API routes
 */

import { 
  createPayment,
  getPaymentStatus,
  verifyPayment,
  cancelPayment
} from '@/app/actions/payment-server';
import { logger, safeError } from '@/lib/logger';

interface CreatePaymentRequest {
  amount: number;
  currency: string;
  description?: string;
  orderNo: string;
}

interface ValidatePaymentRequest {
  paymentId: string;
  signature?: string;
}

interface QRPaymentRequest {
  amount: number;
  currency: string;
  orderNo?: string;
  description?: string;
}

export const paymentClient = {
  async createPayment(data: CreatePaymentRequest) {
    try {
      return await createPayment(data);
    } catch (error) {
      logger.error('Payment creation failed', safeError(error));
      throw error;
    }
  },

  async validatePayment(data: ValidatePaymentRequest) {
    try {
      return await verifyPayment(data.paymentId);
    } catch (error) {
      logger.error('Payment validation failed', safeError(error));
      throw error;
    }
  },

  async getPaymentStatus(paymentId?: string) {
    try {
      return await getPaymentStatus();
    } catch (error) {
      logger.error('Payment status retrieval failed', safeError(error));
      throw error;
    }
  },

  async getTransactionHistory() {
    try {
      // This would need to be implemented
      throw new Error('getTransactionHistory needs to be implemented');
    } catch (error) {
      logger.error('Transaction history retrieval failed', safeError(error));
      throw error;
    }
  },

  async initQRPayment(data: QRPaymentRequest) {
    try {
      // This would need to be implemented
      throw new Error('initQRPayment needs to be implemented');
    } catch (error) {
      logger.error('QR payment initialization failed', safeError(error));
      throw error;
    }
  },

  async getPlanDetails(planId?: string) {
    try {
      // This would need to be implemented
      throw new Error('getPlanDetails needs to be implemented');
    } catch (error) {
      logger.error('Plan details retrieval failed', safeError(error));
      throw error;
    }
  },
};