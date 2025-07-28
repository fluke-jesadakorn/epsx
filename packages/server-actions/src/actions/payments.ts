'use server';

import { serverGet, serverPost } from '../core/request';

// Payment Types
export interface PaymentStatus {
  paid: boolean;
  lastPayDate?: Date;
  expireDate?: Date;
  userLevel?: string;
  isNewUser?: boolean;
}

export interface PaymentTransaction {
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

// Payment Actions
export async function createPayment(data: {
  amount: number;
  currency: string;
  description?: string;
  orderNo: string;
}) {
  try {
    return await serverPost('/api/v1/payments/musepay/create', data);
  } catch (error) {
    console.error('Error creating payment:', error);
    throw error;
  }
}

export async function validatePayment(data: { paymentId: string; signature?: string }) {
  try {
    return await serverPost(`/api/v1/payments/${data.paymentId}/validate`, data.signature ? { signature: data.signature } : {});
  } catch (error) {
    console.error('Error validating payment:', error);
    throw error;
  }
}

export async function getPaymentStatus(paymentId?: string): Promise<PaymentStatus | null> {
  try {
    const endpoint = paymentId ? `/api/v1/payments/${paymentId}/status` : '/api/v1/users/payment-status';
    const response = await serverGet(endpoint);
    
    if (response) {
      return {
        paid: response.hasPaid || false,
        lastPayDate: response.lastPaymentDate ? new Date(response.lastPaymentDate) : undefined,
        expireDate: response.expirationDate ? new Date(response.expirationDate) : undefined,
        userLevel: response.userLevel || 'BRONZE',
        isNewUser: !response.hasPaid,
      };
    }

    return {
      paid: false,
      userLevel: 'BRONZE',
      isNewUser: true,
    };
  } catch (error) {
    console.error('Error getting payment status:', error);
    return null;
  }
}

export async function getTransactionHistory(excludePending?: boolean): Promise<PaymentTransaction[]> {
  try {
    const endpoint = excludePending ? '/user/transactions?excludePending=true' : '/user/transactions';
    const response = await serverGet(endpoint);
    
    if (response?.transactions) {
      return response.transactions.map((tx: any) => ({
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
}

export async function getPlanDetails(planId?: string) {
  try {
    const endpoint = planId ? `/api/v1/plans/${planId}` : '/api/v1/plans';
    return await serverGet(endpoint);
  } catch (error) {
    console.error('Error getting plan details:', error);
    throw error;
  }
}

export async function initQRPayment(data: {
  amount: number;
  currency: string;
  orderNo?: string;
  description?: string;
}) {
  try {
    return await serverPost('/api/v1/payments/musepay/create', data);
  } catch (error) {
    console.error('Error initializing QR payment:', error);
    throw error;
  }
}