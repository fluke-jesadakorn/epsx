import { doc, getDoc, collection, query, where, getDocs, orderBy } from 'firebase/firestore';
import { db, auth } from '@/lib/firebase';
import { nanoid } from 'nanoid';

export interface PaymentStatus {
  paid: boolean;
  lastPayDate?: Date;
  expireDate?: Date;
  userLevel?: string;
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
        throw new Error('User not authenticated');
      }

      const userDoc = await getDoc(doc(db, 'users', user.uid));
      if (!userDoc.exists()) {
        return {
          paid: false,
          userLevel: 'BASIC',
        };
      }

      const userData = userDoc.data();
      return {
        paid: userData.paymentStatus?.hasPaid || false,
        lastPayDate: userData.paymentStatus?.lastPaymentDate?.toDate(),
        expireDate: userData.paymentStatus?.expirationDate?.toDate(),
        userLevel: userData.userLevel || 'BASIC',
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
        throw new Error('User not authenticated');
      }

      const txRef = collection(db, 'transactions');
      const q = query(
        txRef,
        where('userId', '==', user.uid),
        orderBy('finishTime', 'desc')
      );

      const querySnapshot = await getDocs(q);
      const txs: PaymentTx[] = [];

      querySnapshot.forEach((doc) => {
        const data = doc.data();
        txs.push({
          orderNo: data.orderNo,
          amount: data.actualAmount,
          currency: data.currency,
          status: data.status,
          finishTime: data.finishTime?.toDate?.().toISOString() || new Date().toISOString(),
          blockchainData: data.blockchainData || { txHash: '', network: '' },
          blockExplorerUrl: data.blockExplorerUrl || '',
        });
      });

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

  return {
    recordPayment,
    confirmPayment,
    getPaymentStatus,
    initQRPayment,
    getPlanDetails,
    getTxHistory,
  };
};
