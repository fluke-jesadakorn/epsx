import { 
  createPayment,
  getPaymentStatus as getPaymentStatusAction,
  verifyPayment
} from '@/app/actions/payment-server';
import type { PaymentStatus, PaymentTransaction } from '@/types/payment-types';

// Simple ID generator to replace nanoid
const generateId = () => Math.random().toString(36).substring(2) + Date.now().toString(36);

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
      console.error('Error recording payment:', error);
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
      console.error('Error confirming payment:', error);
      return { success: false, message: 'Failed to confirm payment' };
    }
  };

  const getPaymentStatus = async (): Promise<PaymentStatus | null> => {
    try {
      return await getPaymentStatusAction();
    } catch (error) {
      console.error('Error getting payment status:', error);
      return null;
    }
  };

  const getTxHistory = async (): Promise<PaymentTx[]> => {
    try {
      return await getTransactionHistory();
    } catch (error) {
      console.error('Error fetching transaction history:', error);
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
      console.error('Error initializing QR payment:', error);
      throw error;
    }
  };

  const getPlanDetailsLocal = async (planId: string) => {
    try {
      return await getPlanDetails(planId);
    } catch (error) {
      console.error('Error getting plan details:', error);
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
      console.error('Error fetching transaction history for new user:', error);
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