/**
 * Client-side payment service
 * This wraps server actions for use in client components
 */

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
  orderNo: string;
  description?: string;
}

export const paymentClient = {
  async createPayment(data: CreatePaymentRequest) {
    try {
      const response = await fetch('/api/payment/create', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Payment creation failed: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error creating payment:', error);
      throw error;
    }
  },

  async validatePayment(data: ValidatePaymentRequest) {
    try {
      const response = await fetch('/api/payment/validate', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Payment validation failed: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error validating payment:', error);
      throw error;
    }
  },

  async getPaymentStatus(paymentId: string) {
    try {
      const response = await fetch(`/api/payment/status?paymentId=${encodeURIComponent(paymentId)}`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Payment status check failed: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error getting payment status:', error);
      throw error;
    }
  },

  async getTransactionHistory() {
    try {
      const response = await fetch('/api/payment/history', {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Failed to get transaction history: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error getting transaction history:', error);
      throw error;
    }
  },

  async initQRPayment(data: QRPaymentRequest) {
    try {
      const response = await fetch('/api/payment/qr-init', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `QR payment init failed: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error initializing QR payment:', error);
      throw error;
    }
  },

  async getPlanDetails(planId?: string) {
    try {
      const url = planId ? `/api/payment/plan-details?planId=${encodeURIComponent(planId)}` : '/api/payment/plan-details';
      const response = await fetch(url, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Failed to get plan details: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error getting plan details:', error);
      throw error;
    }
  },
};