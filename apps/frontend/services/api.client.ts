import { createApiClient, isApiError } from '@epsx/api-client';

// For client-side, use relative URLs through Next.js API routes
// The actual backend URL will be handled server-side
const API_BASE_URL = '/api';

// Create API client instance that routes through Next.js API
const apiClient = createApiClient(API_BASE_URL);

// Legacy wrapper class for backward compatibility
class ApiService {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const method = (options.method || 'GET') as 'GET' | 'POST' | 'PUT' | 'DELETE';
    const data = options.body ? JSON.parse(options.body as string) : undefined;
    
    let response;
    switch (method) {
      case 'GET':
        response = await apiClient.get<T>(endpoint, options.headers as Record<string, string>);
        break;
      case 'POST':
        response = await apiClient.post<T>(endpoint, data, options.headers as Record<string, string>);
        break;
      case 'PUT':
        response = await apiClient.put<T>(endpoint, data, options.headers as Record<string, string>);
        break;
      case 'DELETE':
        response = await apiClient.delete<T>(endpoint, data, options.headers as Record<string, string>);
        break;
      default:
        throw new Error(`Unsupported method: ${method}`);
    }

    if (isApiError(response)) {
      throw new Error(response.error || 'Request failed');
    }

    return response.data;
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET' });
  }

  async post<T>(endpoint: string, data?: any): Promise<T> {
    const options: RequestInit = {
      method: 'POST',
    };
    if (data) {
      options.body = JSON.stringify(data);
    }
    return this.request<T>(endpoint, options);
  }

  async put<T>(endpoint: string, data?: any): Promise<T> {
    const options: RequestInit = {
      method: 'PUT',
    };
    if (data) {
      options.body = JSON.stringify(data);
    }
    return this.request<T>(endpoint, options);
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

export const api = new ApiService();

// Specific API endpoints using the unified API client directly
export const stockApi = {
  getStocks: async () => {
    const response = await apiClient.getStocks();
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get stocks');
    }
    return response.data;
  },
  getSymbols: async () => {
    const response = await apiClient.get(`/api/v1/market-data/symbols`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get symbols');
    }
    return response.data;
  },
  getStock: async (symbol: string) => {
    const response = await apiClient.get(`/api/v1/stock/financial-data/${symbol}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get stock');
    }
    return response.data;
  },
  searchStocks: async (query: string) => {
    const response = await apiClient.get(`/api/v1/stock/screener/search?q=${query}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to search stocks');
    }
    return response.data;
  },
  getStockHistory: async (symbol: string, period = '1y') => {
    const response = await apiClient.get(`/api/v1/stock/price-data/${symbol}/history?period=${period}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get stock history');
    }
    return response.data;
  },
};

export const rankingApi = {
  getRankings: async () => {
    const response = await apiClient.get('/api/v1/premium/rankings');
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get rankings');
    }
    return response.data;
  },
  getUserRanking: async (userId: string) => {
    const response = await apiClient.get(`/api/v1/premium/rankings/user/${userId}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get user ranking');
    }
    return response.data;
  },
  updateRanking: async (data: any) => {
    const response = await apiClient.post('/api/v1/premium/rankings', data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to update ranking');
    }
    return response.data;
  },
};

export const paymentApi = {
  createPayment: async (data: any) => {
    const response = await apiClient.post('/api/v1/payments/musepay/create', data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to create payment');
    }
    return response.data;
  },
  getPaymentStatus: async (paymentId: string) => {
    const response = await apiClient.get(`/api/v1/payments/musepay/${paymentId}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get payment status');
    }
    return response.data;
  },
  verifyPayment: async (paymentId: string) => {
    const response = await apiClient.get(`/api/v1/payments/musepay/${paymentId}/validate`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to verify payment');
    }
    return response.data;
  },
  getCryptoDepositAddress: async () => {
    const response = await apiClient.get('/api/v1/payments/crypto/deposit-address');
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get crypto deposit address');
    }
    return response.data;
  },
  getQrCode: async (paymentId: string) => {
    const response = await apiClient.get(`/api/v1/payments/musepay/${paymentId}/qrcode`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get QR code');
    }
    return response.data;
  },
};

export const userApi = {
  getProfile: async () => {
    const response = await apiClient.getCurrentUser();
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get profile');
    }
    return response.data;
  },
  updateProfile: async (data: any) => {
    const response = await apiClient.updateProfile(data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to update profile');
    }
    return response.data;
  },
  getUserData: async () => {
    const response = await apiClient.get('/api/v1/users/profile');
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get user data');
    }
    return response.data;
  },
  updateUserData: async (data: any) => {
    const response = await apiClient.put('/api/v1/users/profile', data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to update user data');
    }
    return response.data;
  },
  getUserById: async (id: string) => {
    const response = await apiClient.get(`/api/v1/users/${id}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get user by id');
    }
    return response.data;
  },
  listUsers: async () => {
    const response = await apiClient.get('/api/v1/users');
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to list users');
    }
    return response.data;
  },
  deleteUser: async (id: string) => {
    const response = await apiClient.delete(`/api/v1/users/${id}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to delete user');
    }
    return response.data;
  },
};

// Export the API client directly for advanced usage
export { apiClient };