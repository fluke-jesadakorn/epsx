import axios from 'axios';
import type { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

export interface ApiError {
  message: string;
  status: number;
  code?: string;
}

export interface ApiResponse<T = any> {
  data: T;
  status: number;
  success: boolean;
}

export interface WatchlistAddRequest {
  symbol: string;
  name?: string;
}

export interface PriceAlertCreateRequest {
  symbol: string;
  targetPrice: number;
  condition: 'above' | 'below';
}

export interface PushSubscriptionRequest {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}

export interface PaymentStatus {
  id: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled';
  amount: number;
  currency: string;
  createdAt: string;
}

export interface PaymentTransaction {
  id: string;
  amount: number;
  currency: string;
  status: string;
  createdAt: string;
  description?: string;
}

export class ApiClient {
  private instance: AxiosInstance;

  constructor(baseURL: string, token?: string) {
    this.instance = axios.create({
      baseURL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
      },
    });

    this.instance.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          // Handle unauthorized - redirect to login
          if (typeof window !== 'undefined') {
            window.location.href = '/login';
          }
        }
        return Promise.reject(error);
      }
    );
  }

  async get<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.get(url, config);
    return { data: response.data, status: response.status, success: true };
  }

  async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.post(url, data, config);
    return { data: response.data, status: response.status, success: true };
  }

  async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.put(url, data, config);
    return { data: response.data, status: response.status, success: true };
  }

  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.instance.delete(url, config);
    return { data: response.data, status: response.status, success: true };
  }

  setAuthToken(token: string) {
    this.instance.defaults.headers.Authorization = `Bearer ${token}`;
  }

  removeAuthToken() {
    delete this.instance.defaults.headers.Authorization;
  }
}

export function createApiClient(baseURL?: string, token?: string): ApiClient {
  const url = baseURL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
  return new ApiClient(url, token);
}

export function isApiError(error: any): error is ApiError {
  return error && typeof error.message === 'string' && typeof error.status === 'number';
}

// Analytics Client for unified analytics endpoints
export class AnalyticsClient {
  private client: AxiosInstance;

  constructor(baseURL?: string) {
    this.client = axios.create({
      baseURL: baseURL || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Add request interceptor for authentication
    this.client.interceptors.request.use((config) => {
      if (typeof window !== 'undefined') {
        const token = document.cookie
          .split('; ')
          .find(row => row.startsWith('epsx_frontend_jwt='))
          ?.split('=')[1];
        
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
      }
      return config;
    });
  }

  async getUnifiedAnalyticsRankings(params: {
    page: number;
    limit: number;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: number;
    min_growth?: number;
  }): Promise<ApiResponse<any>> {
    try {
      const response = await this.client.get('/api/analytics/unified', { params });
      return {
        data: response.data,
        status: response.status,
        success: true,
      };
    } catch (error: any) {
      throw {
        message: error.response?.data?.message || 'Failed to fetch analytics data',
        status: error.response?.status || 500,
        code: error.response?.data?.code,
      } as ApiError;
    }
  }
}

// Default instance
export const apiClient = createApiClient();