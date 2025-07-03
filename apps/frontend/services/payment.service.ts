import { doc, updateDoc, getDoc, collection, query, where, getDocs, orderBy } from 'firebase/firestore';
import { db, auth } from '@/lib/firebase';
import { nanoid } from 'nanoid';
import type { UserLevelType } from '@/app/constants/packages';

export interface PaymentStatus {
  hasPaid: boolean;
  lastPaymentDate?: Date;
  expirationDate?: Date;
  userLevel?: string;
}

export interface PaymentTransaction {
  orderNo: string;
  actualAmount: number;
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
      const transactionId = nanoid();
      // Implementation...
      return transactionId;
    } catch (error) {
      console.error('Error recording payment:', error);
      return null;
    }
  };

  const confirmPayment = async (
    transactionId: string,
    paymentMethod: string,
    userLevel: string,
  ): Promise<{
    success: boolean;
    message: string;
  }> => {
    try {
      // Implementation...
      return { success: true, message: 'Payment confirmed successfully' };
    } catch (error) {
      console.error('Error confirming payment:', error);
      return { success: false, message: 'Failed to confirm payment' };
    }
  };

  const getPaymentStatus = async (): Promise<PaymentStatus | null> => {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new Error('User not authenticated');
      }

      const userDoc = await getDoc(doc(db, 'users', user.uid));
      if (!userDoc.exists()) {
        return {
          hasPaid: false,
          userLevel: 'BASIC',
        };
      }

      const userData = userDoc.data();
      return {
        hasPaid: userData.paymentStatus?.hasPaid || false,
        lastPaymentDate: userData.paymentStatus?.lastPaymentDate?.toDate(),
        expirationDate: userData.paymentStatus?.expirationDate?.toDate(),
        userLevel: userData.userLevel || 'BASIC',
      };
    } catch (error) {
      console.error('Error getting payment status:', error);
      return null;
    }
  };

  const getTransactionHistory = async (): Promise<PaymentTransaction[]> => {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new Error('User not authenticated');
      }

      const transactionsRef = collection(db, 'transactions');
      const q = query(
        transactionsRef,
        where('userId', '==', user.uid),
        orderBy('finishTime', 'desc')
      );

      const querySnapshot = await getDocs(q);
      const transactions: PaymentTransaction[] = [];

      querySnapshot.forEach((doc) => {
        const data = doc.data();
        transactions.push({
          orderNo: data.orderNo,
          actualAmount: data.actualAmount,
          currency: data.currency,
          status: data.status,
          finishTime: data.finishTime?.toDate?.().toISOString() || new Date().toISOString(),
          blockchainData: data.blockchainData || { txHash: '', network: '' },
          blockExplorerUrl: data.blockExplorerUrl || '',
        });
      });

      return transactions;
    } catch (error) {
      console.error('Error fetching transaction history:', error);
      return [];
    }
  };

  const initiateQRCodePayment = async (amount: number, currency: string) => {
    // Implementation...
  };

  const getPlanDetails = async (planId: string) => {
    // Implementation...
  };

  return {
    recordPayment,
    confirmPayment,
    getPaymentStatus,
    initiateQRCodePayment,
    getPlanDetails,
    getTransactionHistory,
  };
};
