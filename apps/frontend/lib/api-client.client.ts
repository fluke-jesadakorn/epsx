// Unified API client to replace Firebase SDK
// Handles all backend communication with cookie-based authentication

import { logger } from './logger';

interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  details?: string;
}

interface LoginRequest {
  type: 'credentials';
  email: string;
  password: string;
}

interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
  package_tier?: string;
}

interface UserProfile {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
}

interface PasswordResetRequest {
  email: string;
}

interface ProfileUpdateRequest {
  displayName?: string;
  photoURL?: string;
}

interface PasswordChangeRequest {
  currentPassword: string;
  newPassword: string;
}

export interface ApiClient {
  // Core HTTP methods
  get<T>(
    endpoint: string,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>>;
  post<T, TData = unknown>(
    endpoint: string,
    data: TData,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>>;
  put<T, TData = unknown>(
    endpoint: string,
    data: TData,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>>;
  delete<T>(
    endpoint: string,
    headers?: Record<string, string>
  ): Promise<ApiResponse<void>>;

  // Authentication methods
  login(credentials: LoginRequest): Promise<ApiResponse<UserProfile>>;
  register(
    userData: RegisterRequest
  ): Promise<
    ApiResponse<{
      user_id: string;
      email: string;
      verification_sent: boolean;
      message: string;
    }>
  >;
  logout(): Promise<ApiResponse<void>>;
  getCurrentUser(): Promise<ApiResponse<UserProfile>>;
  refreshSession(): Promise<ApiResponse<{ expires_at: string }>>;
  resetPassword(
    request: PasswordResetRequest
  ): Promise<ApiResponse<{ message: string; reset_sent: boolean }>>;
  updateProfile(
    request: ProfileUpdateRequest
  ): Promise<ApiResponse<UserProfile>>;
  changePassword(
    request: PasswordChangeRequest
  ): Promise<ApiResponse<{ message: string }>>;
}

class ApiClientImpl implements ApiClient {
  private baseUrl = '';

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        credentials: 'include', // Include cookies
        ...options,
      });

      const data = await response.json();

      if (!response.ok) {
        return {
          error: data.error || 'Request failed',
          details: data.details,
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

  // Core HTTP methods
  async get<T>(
    endpoint: string,
    headers: Record<string, string> = {}
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'GET', headers });
  }

  async post<T, TData = unknown>(
    endpoint: string,
    data: TData,
    headers: Record<string, string> = {}
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(data),
    });
  }

  async put<T, TData = unknown>(
    endpoint: string,
    data: TData,
    headers: Record<string, string> = {}
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      headers,
      body: JSON.stringify(data),
    });
  }

  async delete<T>(
    endpoint: string,
    headers: Record<string, string> = {}
  ): Promise<ApiResponse<void>> {
    return this.request<void>(endpoint, { method: 'DELETE', headers });
  }

  // Authentication methods
  async login(credentials: LoginRequest): Promise<ApiResponse<UserProfile>> {
    return this.request<UserProfile>('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });
  }

  async register(
    userData: RegisterRequest
  ): Promise<
    ApiResponse<{
      user_id: string;
      email: string;
      verification_sent: boolean;
      message: string;
    }>
  > {
    return this.request('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify(userData),
    });
  }

  async logout(): Promise<ApiResponse<void>> {
    return this.request('/api/v1/auth/logout', {
      method: 'POST',
    });
  }

  async getCurrentUser(): Promise<ApiResponse<UserProfile>> {
    return this.request<UserProfile>('/api/v1/auth/profile');
  }

  async refreshSession(): Promise<ApiResponse<{ expires_at: string }>> {
    return this.request('/api/v1/auth/refresh', {
      method: 'POST',
    });
  }

  async resetPassword(
    request: PasswordResetRequest
  ): Promise<ApiResponse<{ message: string; reset_sent: boolean }>> {
    return this.request('/api/v1/auth/password-reset', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async updateProfile(
    request: ProfileUpdateRequest
  ): Promise<ApiResponse<UserProfile>> {
    return this.request<UserProfile>('/api/v1/auth/profile', {
      method: 'PATCH',
      body: JSON.stringify(request),
    });
  }

  async changePassword(
    request: PasswordChangeRequest
  ): Promise<ApiResponse<{ message: string }>> {
    return this.request('/api/v1/auth/password', {
      method: 'PATCH',
      body: JSON.stringify(request),
    });
  }
}

// Create singleton instance
export const apiClient = new ApiClientImpl();

// Export types for use in components
export type {
  ApiResponse,
  LoginRequest,
  PasswordChangeRequest,
  PasswordResetRequest,
  ProfileUpdateRequest,
  RegisterRequest,
  UserProfile,
};

// Utility functions for common patterns
export const isApiError = (
  response: ApiResponse
): response is { error: string; details?: string } => {
  return 'error' in response && !!response.error;
};

export const isApiSuccess = <T>(
  response: ApiResponse<T>
): response is { data: T } => {
  return 'data' in response && !response.error;
};

// Authentication helpers
export const loginWithCredentials = async (email: string, password: string) => {
  return apiClient.login({ type: 'credentials', email, password });
};

// Real-time communication interfaces
interface PaymentStatusUpdate {
  customerRefId: string;
  status: string;
  amount?: number;
  timestamp: string;
}

interface RealtimeClient {
  connectToPaymentUpdates(
    customerRefId: string,
    callback: (update: PaymentStatusUpdate) => void
  ): () => void;
  connectToNotifications(callback: (notification: any) => void): () => void;
}

class RealtimeClientImpl implements RealtimeClient {
  private baseWsUrl =
    process.env.NODE_ENV === 'production'
      ? 'wss://api.epsx.com'
      : 'ws://localhost:8000';

  connectToPaymentUpdates(
    customerRefId: string,
    callback: (update: PaymentStatusUpdate) => void
  ): () => void {
    // Try WebSocket first, fallback to SSE
    const ws = new WebSocket(`${this.baseWsUrl}/realtime/ws`);

    ws.onopen = () => {
      // Send subscription message for payment updates
      ws.send(
        JSON.stringify({
          type: 'subscribe',
          topic: 'payment_updates',
          customerRefId,
        })
      );
    };

    ws.onmessage = event => {
      try {
        const data = JSON.parse(event.data);
        if (
          data.type === 'payment_update' &&
          data.customerRefId === customerRefId
        ) {
          callback(data as PaymentStatusUpdate);
        }
      } catch (error) {
        logger.error('WebSocket message parse error', {
          error: error instanceof Error ? error.message : error,
          customerRefId,
        });
      }
    };

    ws.onerror = error => {
      logger.error('WebSocket error', {
        error: error instanceof Error ? error.message : error,
        customerRefId,
      });
      // Fallback to SSE
      this.fallbackToSSE(customerRefId, callback);
    };

    // Return cleanup function
    return () => {
      if (
        ws.readyState === WebSocket.OPEN ||
        ws.readyState === WebSocket.CONNECTING
      ) {
        ws.close();
      }
    };
  }

  connectToNotifications(callback: (notification: any) => void): () => void {
    const ws = new WebSocket(`${this.baseWsUrl}/realtime/ws`);

    ws.onopen = () => {
      ws.send(
        JSON.stringify({
          type: 'subscribe',
          topic: 'notifications',
        })
      );
    };

    ws.onmessage = event => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'notification') {
          callback(data);
        }
      } catch (error) {
        logger.error('WebSocket notification parse error', {
          error: error instanceof Error ? error.message : error,
        });
      }
    };

    return () => {
      if (
        ws.readyState === WebSocket.OPEN ||
        ws.readyState === WebSocket.CONNECTING
      ) {
        ws.close();
      }
    };
  }

  private fallbackToSSE(
    customerRefId: string,
    callback: (update: PaymentStatusUpdate) => void
  ): () => void {
    const eventSource = new EventSource(
      `${this.baseWsUrl.replace('ws://', 'http://').replace('wss://', 'https://')}/realtime/events?customerRefId=${customerRefId}`,
      {
        withCredentials: true,
      }
    );

    eventSource.onmessage = event => {
      try {
        const data = JSON.parse(event.data);
        if (
          data.type === 'payment_update' &&
          data.customerRefId === customerRefId
        ) {
          callback(data as PaymentStatusUpdate);
        }
      } catch (error) {
        logger.error('SSE message parse error', {
          error: error instanceof Error ? error.message : error,
          customerRefId,
        });
      }
    };

    eventSource.onerror = error => {
      logger.error('SSE error', {
        error: error instanceof Error ? error.message : error,
        customerRefId,
      });
    };

    return () => {
      eventSource.close();
    };
  }
}

// Create singleton instances
export const realtimeClient = new RealtimeClientImpl();

// Export real-time types
export type { PaymentStatusUpdate, RealtimeClient };
