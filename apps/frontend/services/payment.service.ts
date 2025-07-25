import { apiClient, isApiSuccess } from '@/lib/api-client.client';
import { nanoid } from 'nanoid';

export interface PaymentStatus {
  paid: boolean;
  lastPayDate?: Date;
  expireDate?: Date;
  userLevel?: string;
  isNewUser?: boolean;
}

export interface PaymentTx {
  orderNo: string;
  amount: number;
  currency: string;
  status: string;
  finishTime: string;
  blockchainData: {
    txHash: string;
    network: string;
  };
  blockExplorerUrl: string;
}

export const createPaymentService = () => {
  const recordPayment = async (
    amount: number,
    currency: string,
    description?: string,
  ): Promise<string | null> => {
    try {
      const response = await apiClient.post('/payments', {
        amount,
        currency,
        description,
        orderNo: nanoid(),
      });

      if (isApiSuccess(response) && response.data) {
        return response.data.paymentId || response.data.orderNo;
      }

      console.error('Failed to record payment:', response.error);
      return null;
    } catch (error) {
      console.error('Error recording payment:', error);
      return null;
    }
  };

  const confirmPayment = async (
    txId: string,
    payMethod: string,
    userLevel: string,
  ): Promise<{
    success: boolean;
    message: string;
  }> => {
    try {
      const response = await apiClient.post(`/payments/${txId}/confirm`, {
        payMethod,
        userLevel,
      });

      if (isApiSuccess(response)) {
        return { success: true, message: 'Payment confirmed successfully' };
      }

      return { 
        success: false, 
        message: response.error || 'Failed to confirm payment' 
      };
    } catch (error) {
      console.error('Error confirming payment:', error);
      return { success: false, message: 'Failed to confirm payment' };
    }
  };

  const getPaymentStatus = async (): Promise<PaymentStatus | null> => {
    try {
      const response = await apiClient.get('/user/payment-status');

      if (isApiSuccess(response) && response.data) {
        const data = response.data;
        return {
          paid: data.hasPaid || false,
          lastPayDate: data.lastPaymentDate ? new Date(data.lastPaymentDate) : undefined,
          expireDate: data.expirationDate ? new Date(data.expirationDate) : undefined,
          userLevel: data.userLevel || 'BRONZE',
          isNewUser: !data.hasPaid,
        };
      }

      // Return default status if API call fails
      return {
        paid: false,
        userLevel: 'BRONZE',
        isNewUser: true,
      };
    } catch (error) {
      console.error('Error getting payment status:', error);
      return null;
    }
  };

  const getTxHistory = async (): Promise<PaymentTx[]> => {
    try {
      const response = await apiClient.get('/user/transactions');

      if (isApiSuccess(response) && response.data) {
        const transactions = response.data.transactions || [];
        
        return transactions.map((tx: any) => ({
          orderNo: tx.orderNo,
          amount: tx.amount,
          currency: tx.currency,
          status: tx.status,
          finishTime: tx.finishTime || new Date().toISOString(),
          blockchainData: tx.blockchainData || { txHash: '', network: '' },
          blockExplorerUrl: tx.blockExplorerUrl || '',
        }));
      }

      return [];
    } catch (error) {
      console.error('Error fetching transaction history:', error);
      return [];
    }
  };

  const initQRPayment = async (amount: number, currency: string) => {
    try {
      const response = await apiClient.post('/payments/qr-init', {
        amount,
        currency,
      });

      if (isApiSuccess(response)) {
        return response.data;
      }

      throw new Error(response.error || 'Failed to initialize QR payment');
    } catch (error) {
      console.error('Error initializing QR payment:', error);
      throw error;
    }
  };

  const getPlanDetails = async (planId: string) => {
    try {
      const response = await apiClient.get(`/plans/${planId}`);

      if (isApiSuccess(response)) {
        return response.data;
      }

      throw new Error(response.error || 'Failed to get plan details');
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

      const response = await apiClient.get('/user/transactions?excludePending=true');

      if (isApiSuccess(response) && response.data) {
        const transactions = response.data.transactions || [];
        
        return transactions
          .filter((tx: any) => tx.status !== 'PENDING' && tx.status !== 'pending')
          .map((tx: any) => ({
            orderNo: tx.orderNo,
            amount: tx.amount,
            currency: tx.currency,
            status: tx.status,
            finishTime: tx.finishTime || new Date().toISOString(),
            blockchainData: tx.blockchainData || { txHash: '', network: '' },
            blockExplorerUrl: tx.blockExplorerUrl || '',
          }));
      }

      return [];
    } catch (error) {
      console.error('Error fetching transaction history for new user:', error);
      return [];
    }
  };

  return {
    recordPayment,
    confirmPayment,
    getPaymentStatus,
    initQRPayment,
    getPlanDetails,
    getTxHistory,
    getTxHistoryForNewUser,
  };
};