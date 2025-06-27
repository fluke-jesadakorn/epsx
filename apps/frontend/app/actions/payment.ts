'use server';

import { apiClient } from '@/lib/api-client';
import { createPaymentService } from '@/services/payment.service';

import type { PaymentResponse } from '@/types/payment.d.ts';

const paymentService = createPaymentService({
  apiUrl: process.env.NEXT_PUBLIC_API_URL || '',
  endpoints: {
    createPayment: '/payment',
    validatePayment: '/payment/validate',
    getPayment: '/payment',
    getQrCode: '/payment/qrcode'
  }
}, apiClient);

export async function getPaymentDetails(userId: string) {
  try {
    const response = await paymentService.getPaymentDetails(userId);
    if (!response) return undefined;
    
    const paymentResponse = response as unknown as PaymentResponse;
    return {
      network: 'TRC20' as const,
      walletAddress: 'TXYZ1234567890',
      paymentStatus: {
        lastPaymentDate: new Date(paymentResponse.created_at),
        expirationDate: new Date(paymentResponse.expiration_date),
        paymentMethod: 'USDT' as const,
        transactionId: paymentResponse.id,
        amount: paymentResponse.amount
      },
      userLevel: paymentResponse.user_level
    };
  } catch (error) {
    console.error('Failed to get payment details:', error);
    return undefined;
  }
}
