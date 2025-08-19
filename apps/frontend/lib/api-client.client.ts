// Unified API client using the refactored package structure
// This file provides backward compatibility and convenience exports

import { apiClient, createApiClient, AnalyticsClient } from './api-client';

// Re-export the main API client instance and classes
export { apiClient, AnalyticsClient };

// Re-export types for backward compatibility
export interface ApiResponse<T = any> {
  data: T;
  status: number;
  success: boolean;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
}

export interface UserProfile {
  id: string;
  email: string;
  name?: string;
  createdAt: string;
}

export interface PasswordResetRequest {
  email: string;
}

export interface ProfileUpdateRequest {
  name?: string;
  email?: string;
}

export interface PasswordChangeRequest {
  currentPassword: string;
  newPassword: string;
}

// Utility functions for common patterns (using shared-core error handling)
import { ErrorHandler } from '@epsx/shared-core';

export const isApiError = (
  response: any
): response is { error: string; details?: string } => {
  return 'error' in response && !!response.error;
};

export const isApiSuccess = <T>(
  response: any
): response is { data: T } => {
  return 'data' in response && !response.error;
};

// Authentication helpers using the new client structure
export const loginWithCredentials = async (email: string, password: string) => {
  return apiClient.auth.login({ type: 'credentials', email, password });
};

// Export domain-specific clients for direct access
export const authClient = apiClient.auth;
export const paymentClient = apiClient.payments;
export const analyticsClient = apiClient.analytics;
export const permissionsClient = apiClient.permissions;

// Real-time communication interfaces (keeping existing implementation)
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
        console.error('WebSocket message parse error', {
          error: error instanceof Error ? error.message : error,
          customerRefId,
        });
      }
    };

    ws.onerror = error => {
      console.error('WebSocket error', {
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
        console.error('WebSocket notification parse error', {
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
        console.error('SSE message parse error', {
          error: error instanceof Error ? error.message : error,
          customerRefId,
        });
      }
    };

    eventSource.onerror = error => {
      console.error('SSE error', {
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