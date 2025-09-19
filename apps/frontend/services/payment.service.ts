import { 
  createPayment,
  getPaymentStatus as getPaymentStatusAction,
  verifyPayment
} from '@/app/actions/payment-server';
import { getTransactionHistory } from '@/lib/server-actions';
import { initQRPayment } from '@/lib/actions/payments.server';
import type { PaymentStatus, PaymentTransaction } from '@/types/api';
import { logger, safeError } from '@/lib/utils/logging';

// Simple ID generator to replace nanoid
const generateId = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

// Stub implementation for getPlanDetails - TODO: implement proper plan fetching
const getPlanDetails = async (planId: string) => {
  // This is a stub implementation - replace with actual API call
  return {
    id: planId,
    name: 'Basic Plan',
    price: 9.99,
    currency: 'USD',
    features: ['Basic Analytics', 'Email Support']
  };
};

// Re-export types for compatibility
export type { PaymentStatus };
export type PaymentTx = PaymentTransaction;

export const createPaymentService = () => {
  const recordPayment = async (
    amount: number,
    currency: string,
    description?: string,
  ): Promise<string | null> => {
    try {
      const response = await createPayment({
        currency,
        amount: amount.toString(),
        payment_method: 'on_chain',
        product_name: description || 'Payment',
      });

      if (response) {
        return response.paymentId || response.orderNo;
      }

      return null;
    } catch (error) {
      logger.error('Payment recording failed', safeError(error));
      return null;
    }
  };

  const confirmPayment = async (
    txId: string,
    _payMethod: string,
    _userLevel: string,
  ): Promise<{
    success: boolean;
    message: string;
  }> => {
    try {
      const response = await verifyPayment(txId);

      if (response) {
        return { success: true, message: 'Payment confirmed successfully' };
      }

      return { 
        success: false, 
        message: 'Failed to confirm payment' 
      };
    } catch (error) {
      logger.error('Payment confirmation failed', safeError(error));
      return { success: false, message: 'Failed to confirm payment' };
    }
  };

  const getPaymentStatus = async (): Promise<PaymentStatus | null> => {
    try {
      return await getPaymentStatusAction();
    } catch (error) {
      logger.error('Payment status retrieval failed', safeError(error));
      return null;
    }
  };

  const getTxHistory = async (): Promise<PaymentTx[]> => {
    try {
      return await getTransactionHistory();
    } catch (error) {
      logger.error('Transaction history fetch failed', safeError(error));
      return [];
    }
  };

  const initQRPaymentLocal = async (amount: number, currency: string) => {
    try {
      const response = await initQRPayment({
        amount,
        currency,
      });

      return response;
    } catch (error) {
      logger.error('QR payment initialization failed', safeError(error));
      throw error;
    }
  };

  const getPlanDetailsLocal = async (planId: string) => {
    try {
      return await getPlanDetails(planId);
    } catch (error) {
      logger.error('Plan details retrieval failed', safeError(error));
      throw error;
    }
  };

  const getTxHistoryForNewUser = async (): Promise<PaymentTx[]> => {
    try {
      const paymentStatus = await getPaymentStatus();
      if (!paymentStatus?.isNewUser) {
        // If not a new user, return all transactions
        return getTxHistory();
      }

      // Use server action with excludePending parameter
      const transactions = await getTransactionHistory(true);
      return transactions;
    } catch (error) {
      logger.error('New user transaction history fetch failed', safeError(error));
      return [];
    }
  };

  return {
    recordPayment,
    confirmPayment,
    getPaymentStatus,
    initQRPayment: initQRPaymentLocal,
    getPlanDetails: getPlanDetailsLocal,
    getTxHistory,
    getTxHistoryForNewUser,
  };
};