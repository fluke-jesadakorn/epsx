import type { USDTDetails } from '@/types/userLevel';
import type { PaymentResponse } from '@/types/payment.d.ts';

interface PaymentConfig {
  apiUrl: string;
  endpoints: {
    createPayment: string;
    validatePayment: string;
    getPayment: string;
    getQrCode: string;
  };
}

interface ApiClient {
  get: (url: string) => Promise<any>;
  post: (url: string, data: any) => Promise<any>;
}

export function createPaymentService(
  config: PaymentConfig,
  apiClient: ApiClient,
) {
  const createPayment = async (
    amount: number,
    currency: string,
  ): Promise<PaymentResponse | null> => {
    try {
      const fullUrl = `${config.apiUrl}${config.endpoints.createPayment}`;
      const response = await apiClient.post(fullUrl, {
        amount,
        currency,
        description: 'Subscription payment',
      });
      return response.data;
    } catch (error) {
      console.error('Failed to create payment:', error);
      return null;
    }
  };

  const validatePayment = async (paymentId: string): Promise<boolean> => {
    try {
      const fullUrl = `${config.apiUrl}${config.endpoints.validatePayment}/${paymentId}`;
      const response = await apiClient.get(fullUrl);
      return response.data.is_valid;
    } catch (error) {
      console.error('Failed to validate payment:', error);
      return false;
    }
  };

  const getPaymentDetails = async (
    paymentId: string,
  ): Promise<USDTDetails | null> => {
    try {
      const fullUrl = `${config.apiUrl}${config.endpoints.getPayment}/${paymentId}`;
      const response = await apiClient.get(fullUrl);
      return mapPaymentResponse(response.data);
    } catch (error) {
      console.error('Failed to fetch payment details:', error);
      return null;
    }
  };

  const getQrCode = async (paymentId: string): Promise<string | null> => {
    try {
      const fullUrl = `${config.apiUrl}${config.endpoints.getQrCode}/${paymentId}`;
      const response = await apiClient.get(fullUrl);
      return response.data;
    } catch (error) {
      console.error('Failed to get QR code:', error);
      return null;
    }
  };

  const mapPaymentResponse = (response: PaymentResponse): USDTDetails => {
    return {
      network: 'TRC20',
      walletAddress: 'TXYZ1234567890',
      paymentStatus: {
        lastPaymentDate: new Date(response.created_at),
        expirationDate: new Date(response.expiration_date),
        paymentMethod: 'USDT',
        transactionId: response.id,
        amount: response.amount,
      },
      userLevel: response.user_level,
    };
  };

  return {
    createPayment,
    validatePayment,
    getPaymentDetails,
    getQrCode,
  };
}
