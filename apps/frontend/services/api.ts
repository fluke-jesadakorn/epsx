import { createApiClient, isApiError } from '@epsx/api-client';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL;

if (!API_BASE_URL) {
  throw new Error('NEXT_PUBLIC_API_URL environment variable is required');
}

// Create API client instance
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
  getStock: async (symbol: string) => {
    const response = await apiClient.get(`/api/v1/market-data/stocks/${symbol}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get stock');
    }
    return response.data;
  },
  searchStocks: async (query: string) => {
    const response = await apiClient.get(`/api/v1/market-data/stocks/search?q=${query}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to search stocks');
    }
    return response.data;
  },
  getStockHistory: async (symbol: string, period = '1y') => {
    const response = await apiClient.get(`/api/v1/market-data/stocks/${symbol}/history?period=${period}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get stock history');
    }
    return response.data;
  },
};

export const rankingApi = {
  getRankings: async () => {
    const response = await apiClient.get('/api/v1/rankings');
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get rankings');
    }
    return response.data;
  },
  getUserRanking: async (userId: string) => {
    const response = await apiClient.get(`/api/v1/rankings/user/${userId}`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get user ranking');
    }
    return response.data;
  },
  updateRanking: async (data: any) => {
    const response = await apiClient.post('/api/v1/rankings', data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to update ranking');
    }
    return response.data;
  },
};

export const paymentApi = {
  createPayment: async (data: any) => {
    const response = await apiClient.createPayment(data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to create payment');
    }
    return response.data;
  },
  getPaymentStatus: async (paymentId: string) => {
    const response = await apiClient.getPaymentStatus(paymentId);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get payment status');
    }
    return response.data;
  },
  verifyPayment: async (paymentId: string) => {
    const response = await apiClient.post(`/api/v1/payments/${paymentId}/verify`);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to verify payment');
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
    const response = await apiClient.get('/api/v1/users/data');
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to get user data');
    }
    return response.data;
  },
  updateUserData: async (data: any) => {
    const response = await apiClient.put('/api/v1/users/data', data);
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to update user data');
    }
    return response.data;
  },
};

// Export the API client directly for advanced usage
export { apiClient };