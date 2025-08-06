import { Environment, getApiBaseUrl } from '@epsx/shared-core';

import { CookieManager } from '../cookie-manager';

import type { ApiResponse, RequestConfig } from '@epsx/types';

export abstract class BaseHttpClient {
  protected readonly baseUrl: string;
  protected readonly isServer: boolean;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || getApiBaseUrl();
    this.isServer = Environment.isServer();
  }

  protected async request<T>(
    endpoint: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    try {
      console.debug('Making HTTP request', { url, method: config.method || 'GET', component: 'BaseHttpClient' });

      const authHeaders = this.isServer
        ? await CookieManager.buildAuthHeaders()
        : CookieManager.client.buildAuthHeaders();

      const headers = {
        'Content-Type': 'application/json',
        ...authHeaders,
        ...config.headers,
      };

      const requestConfig: RequestInit = {
        method: config.method || 'GET',
        credentials: 'include',
        ...config,
        headers,
      };

      const response = await fetch(url, requestConfig);
      const contentType = response.headers.get('content-type');
      
      let data: unknown;
      if (contentType?.includes('application/json')) {
        data = await response.json() as unknown;
      } else {
        data = await response.text();
      }

      if (!response.ok) {
        // Enhanced error handling for new backend responses
        let errorMessage: string;
        let errorDetails: string;
        
        if (typeof data === 'object' && data !== null) {
          const errorData = data as Record<string, unknown>;
          errorMessage = (errorData.error as string) || (errorData.message as string) || 'Request failed';
          errorDetails = (errorData.details as string) || (errorData.reason as string) || `HTTP ${response.status}`;
        } else if (typeof data === 'string') {
          errorMessage = data;
          errorDetails = `HTTP ${response.status}`;
        } else {
          // Fallback error messages based on status codes
          switch (response.status) {
            case 401:
              errorMessage = 'Authentication required or session expired';
              errorDetails = 'Please log in again';
              break;
            case 403:
              errorMessage = 'Access denied';
              errorDetails = 'You do not have permission to perform this action';
              break;
            case 429:
              errorMessage = 'Rate limit exceeded';
              errorDetails = 'Please wait before trying again';
              break;
            case 500:
              errorMessage = 'Server error';
              errorDetails = 'An internal server error occurred';
              break;
            default:
              errorMessage = 'Request failed';
              errorDetails = `HTTP ${response.status}`;
          }
        }

        console.warn('HTTP request failed', {
          url,
          status: response.status,
          statusText: response.statusText,
          errorMessage,
          errorDetails,
          responseData: data as unknown,
          component: 'BaseHttpClient'
        });

        return {
          error: errorMessage,
          details: errorDetails,
        };
      }

      console.debug('HTTP request successful', { url, status: response.status, component: 'BaseHttpClient' });

      return { data: data as T };
    } catch (error) {
      console.error('HTTP request exception', { 
        url, 
        error: error instanceof Error ? error.message : String(error),
        component: 'BaseHttpClient'
      });

      return {
        error: error instanceof Error ? error.message : 'Request failed',
        details: 'Network or processing error',
      };
    }
  }

  protected async get<T>(
    endpoint: string,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'GET', headers });
  }

  protected async post<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  protected async put<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  protected async patch<T>(
    endpoint: string,
    data?: unknown,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      headers,
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  protected async delete<T>(
    endpoint: string,
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'DELETE', headers });
  }
}