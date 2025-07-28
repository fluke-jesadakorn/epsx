/**
 * Client-side payment service
 * This service now uses server actions directly instead of API routes
 */

import { 
  createPayment, 
  validatePayment, 
  getPaymentStatus, 
  getTransactionHistory, 
  initQRPayment, 
  getPlanDetails 
} from '@epsx/server-actions';

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
      console.error('Error creating payment:', error);
      throw error;
    }
  },

  async validatePayment(data: ValidatePaymentRequest) {
    try {
      return await validatePayment(data);
    } catch (error) {
      console.error('Error validating payment:', error);
      throw error;
    }
  },

  async getPaymentStatus(paymentId: string) {
    try {
      return await getPaymentStatus(paymentId);
    } catch (error) {
      console.error('Error getting payment status:', error);
      throw error;
    }
  },

  async getTransactionHistory() {
    try {
      return await getTransactionHistory();
    } catch (error) {
      console.error('Error getting transaction history:', error);
      throw error;
    }
  },

  async initQRPayment(data: QRPaymentRequest) {
    try {
      return await initQRPayment(data);
    } catch (error) {
      console.error('Error initializing QR payment:', error);
      throw error;
    }
  },

  async getPlanDetails(planId?: string) {
    try {
      return await getPlanDetails(planId);
    } catch (error) {
      console.error('Error getting plan details:', error);
      throw error;
    }
  },
};