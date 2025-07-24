const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002';

interface ApiResponse<T = any> {
  data: T;
  success: boolean;
  message?: string;
}

class ApiService {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`;
    const config: RequestInit = {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    };

    const response = await fetch(url, config);
    const result: ApiResponse<T> = await response.json();

    if (!response.ok || !result.success) {
      throw new Error(result.message || 'Request failed');
    }

    return result.data;
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

// Specific API endpoints
export const stockApi = {
  getStocks: () => api.get('/api/v1/market-data/stocks'),
  getStock: (symbol: string) => api.get(`/api/v1/market-data/stocks/${symbol}`),
  searchStocks: (query: string) => api.get(`/api/v1/market-data/stocks/search?q=${query}`),
  getStockHistory: (symbol: string, period = '1y') => 
    api.get(`/api/v1/market-data/stocks/${symbol}/history?period=${period}`),
};

export const rankingApi = {
  getRankings: () => api.get('/api/v1/rankings'),
  getUserRanking: (userId: string) => api.get(`/api/v1/rankings/user/${userId}`),
  updateRanking: (data: any) => api.post('/api/v1/rankings', data),
};

export const paymentApi = {
  createPayment: (data: any) => api.post('/api/v1/payments', data),
  getPaymentStatus: (paymentId: string) => api.get(`/api/v1/payments/${paymentId}`),
  verifyPayment: (paymentId: string) => api.post(`/api/v1/payments/${paymentId}/verify`),
};

export const userApi = {
  getProfile: () => api.get('/api/v1/users/profile'),
  updateProfile: (data: any) => api.put('/api/v1/users/profile', data),
  getUserData: () => api.get('/api/v1/users/data'),
  updateUserData: (data: any) => api.put('/api/v1/users/data', data),
};
