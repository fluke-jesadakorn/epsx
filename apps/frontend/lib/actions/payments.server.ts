'use server';

import { createApiClient, isApiError } from '@epsx/api-client';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const apiClient = createApiClient(BACKEND_URL);

export async function createMusePayPaymentAction(formData: FormData) {
  const amount = formData.get('amount') as string;
  const currency = formData.get('currency') as string;
  const packageType = formData.get('packageType') as string;

  try {
    const response = await apiClient.serverCreateMusePayPayment({
      amount: parseFloat(amount),
      currency,
      packageType,
    });

    if (isApiError(response)) {
      throw new Error(response.error || 'Payment creation failed');
    }

    return { success: true, data: response.data };
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Payment creation failed' 
    };
  }
}

export async function createCryptoPaymentAction(formData: FormData) {
  const currency = formData.get('currency') as string;
  const userId = formData.get('userId') as string;
  const packageId = formData.get('packageId') as string;
  const description = formData.get('description') as string;

  try {
    const response = await apiClient.serverCreateCryptoPayment({
      currency,
      userId,
      packageId,
      description,
    });

    if (isApiError(response)) {
      throw new Error(response.error || 'Crypto payment creation failed');
    }

    return { success: true, data: response.data };
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Crypto payment creation failed' 
    };
  }
}