'use server';

import type {
  ApiResponse,
  RegisterRequest,
  UserProfile,
} from './types';

/**
 * Server-only API methods that can only be used in Next.js server components
 * and API routes. These methods use the 'use server' directive.
 */

export async function serverRegister(userData: RegisterRequest): Promise<
  ApiResponse<{
    user_id: string;
    email: string;
    verification_sent: boolean;
    message: string;
  }>
> {
  try {
    const response = await fetch(
      'http://localhost:8080/api/v1/auth/register',
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(userData),
      }
    );

    const data = await response.json();

    if (!response.ok) {
      return {
        error: data?.error || 'Request failed',
        details: data?.details || `HTTP ${response.status}`,
      };
    }

    return { data };
  } catch (error) {
    return {
      error: 'Network error',
      details: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

export async function serverGetAdminConfig(): Promise<ApiResponse<{ adminUrl: string }>> {
  const adminFrontendUrl =
    process.env.ADMIN_FRONTEND_URL || 'http://localhost:3001';
  return { data: { adminUrl: adminFrontendUrl } };
}

export async function serverGetVapidKey(): Promise<ApiResponse<{ vapidPublicKey: string }>> {
  const vapidPublicKey = process.env.VAPID_PUBLIC_KEY || '';

  if (!vapidPublicKey) {
    return {
      error: 'VAPID key not configured',
      details: 'HTTP 503',
    };
  }

  return { data: { vapidPublicKey } };
}

export async function serverGetStockSymbols(): Promise<ApiResponse<any>> {
  return serverRequest('/api/market-data/symbols');
}

export async function serverGetIndividualStock(symbol: string): Promise<ApiResponse<any>> {
  return serverRequest(
    `/api/market-data/stocks/individual?symbol=${symbol}`
  );
}

export async function serverBatchStocks(symbols: string[]): Promise<ApiResponse<any>> {
  return serverRequest('/api/market-data/stocks/batch', 'POST', {
    symbols,
  });
}

export async function serverGetPremiumRankings(): Promise<ApiResponse<any>> {
  return serverRequest('/api/premium/rankings');
}

export async function serverGetSystemCache(): Promise<ApiResponse<any>> {
  return serverRequest('/api/system/cache');
}

export async function serverGetCurrentUser(): Promise<ApiResponse<UserProfile>> {
  return serverRequest('/auth/me');
}

export async function serverGetAuditLogs(endpoint: string): Promise<ApiResponse<any>> {
  return serverRequest(endpoint);
}

export async function serverCreateMusePayPayment(data: any): Promise<ApiResponse<any>> {
  return serverRequest('/payments/musepay/create', 'POST', data);
}

export async function serverCreateCryptoPayment(data: any): Promise<ApiResponse<any>> {
  return serverRequest(
    '/api/payments/crypto/deposit-address',
    'POST',
    data
  );
}

export async function serverCreateCryptoDepositAddress(data: any): Promise<ApiResponse<any>> {
  return serverRequest(
    '/api/v1/payments/crypto/deposit-address',
    'POST',
    data
  );
}

export async function serverLogin(credentials: {
  token: string;
}): Promise<ApiResponse<UserProfile>> {
  return serverRequest('/api/v1/auth/login', 'POST', credentials);
}

// Generic server request helper function
async function serverRequest<T>(
  endpoint: string,
  method: string = 'GET',
  data?: any
): Promise<ApiResponse<T>> {
  try {
    const requestConfig: RequestInit = {
      method,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    if (data && method !== 'GET') {
      requestConfig.body = JSON.stringify(data);
    }

    const response = await fetch(
      `http://localhost:8080${endpoint}`,
      requestConfig
    );

    let responseData;
    const contentType = response.headers.get('content-type');

    if (contentType?.includes('application/json')) {
      responseData = await response.json();
    } else {
      responseData = await response.text();
    }

    if (!response.ok) {
      return {
        error: responseData?.error || responseData || 'Request failed',
        details: responseData?.details || `HTTP ${response.status}`,
      };
    }

    return { data: responseData };
  } catch (error) {
    return {
      error: 'Network error',
      details: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}