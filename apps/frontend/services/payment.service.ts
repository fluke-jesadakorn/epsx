import {
  doc,
  getDoc,
  collection,
  query,
  where,
  getDocs,
} from 'firebase/firestore';
import { db, auth } from '@/lib/firebase';
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
    _amount: number,
    _currency: string,
    _description?: string,
  ): Promise<string | null> => {
    try {
      const txId = nanoid();
      // Implementation...
      return txId;
    } catch (error) {
      console.error('Error recording payment:', error);
      return null;
    }
  };

  const confirmPayment = async (
    _txId: string,
    _payMethod: string,
    _userLevel: string,
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
        console.warn('User not authenticated yet, returning default status');
        return {
          paid: false,
          userLevel: 'BASIC',
          isNewUser: true,
        };
      }

      const userDoc = await getDoc(doc(db, 'users', user.uid));
      if (!userDoc.exists()) {
        return {
          paid: false,
          userLevel: 'BASIC',
          isNewUser: true,
        };
      }

      const userData = userDoc.data();
      const hasPaid = userData.paymentStatus?.hasPaid || false;
      
      return {
        paid: hasPaid,
        lastPayDate: userData.paymentStatus?.lastPaymentDate?.toDate(),
        expireDate: userData.paymentStatus?.expirationDate?.toDate(),
        userLevel: userData.userLevel || 'BASIC',
        isNewUser: !hasPaid,
      };
    } catch (error) {
      console.error('Error getting payment status:', error);
      return null;
    }
  };

  const getTxHistory = async (): Promise<PaymentTx[]> => {
    try {
      const user = auth.currentUser;
      if (!user) {
        console.warn('User not authenticated yet, returning empty transaction history');
        return [];
      }

      const txRef = collection(db, 'transactions');
      // Note: We don't use orderBy here to avoid requiring a composite index
      // Alternative: Create a composite index in Firebase Console for userId (ASC) + finishTime (DESC)
      // Index URL: https://console.firebase.google.com/v1/r/project/epsx-449804/firestore/indexes
      const q = query(txRef, where('userId', '==', user.uid));

      const querySnapshot = await getDocs(q);
      const txs: PaymentTx[] = [];

      querySnapshot.forEach((doc) => {
        const data = doc.data();
        txs.push({
          orderNo: data.orderNo,
          amount: data.actualAmount,
          currency: data.currency,
          status: data.status,
          finishTime:
            data.finishTime?.toDate?.().toISOString() ||
            new Date().toISOString(),
          blockchainData: data.blockchainData || { txHash: '', network: '' },
          blockExplorerUrl: data.blockExplorerUrl || '',
        });
      });

      // Sort by finishTime in descending order (most recent first) in JavaScript
      txs.sort(
        (a, b) =>
          new Date(b.finishTime).getTime() - new Date(a.finishTime).getTime(),
      );

      return txs;
    } catch (error) {
      console.error('Error fetching transaction history:', error);
      return [];
    }
  };

  const initQRPayment = async (_amount: number, _currency: string) => {
    // Implementation...
  };

  const getPlanDetails = async (_planId: string) => {
    // Implementation...
  };

  const getTxHistoryForNewUser = async (): Promise<PaymentTx[]> => {
    try {
      const paymentStatus = await getPaymentStatus();
      if (!paymentStatus?.isNewUser) {
        // If not a new user, return all transactions
        return getTxHistory();
      }

      const user = auth.currentUser;
      if (!user) {
        console.warn('User not authenticated yet, returning empty transaction history for new user');
        return [];
      }

      const txRef = collection(db, 'transactions');
      const q = query(txRef, where('userId', '==', user.uid));

      const querySnapshot = await getDocs(q);
      const txs: PaymentTx[] = [];

      querySnapshot.forEach((doc) => {
        const data = doc.data();
        
        // Skip pending transactions for new users
        if (data.status === 'PENDING' || data.status === 'pending') {
          return;
        }

        txs.push({
          orderNo: data.orderNo,
          amount: data.actualAmount,
          currency: data.currency,
          status: data.status,
          finishTime:
            data.finishTime?.toDate?.().toISOString() ||
            new Date().toISOString(),
          blockchainData: data.blockchainData || { txHash: '', network: '' },
          blockExplorerUrl: data.blockExplorerUrl || '',
        });
      });

      // Sort by finishTime in descending order (most recent first) in JavaScript
      txs.sort(
        (a, b) =>
          new Date(b.finishTime).getTime() - new Date(a.finishTime).getTime(),
      );

      return txs;
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
