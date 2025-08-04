import { logger, Environment, getApiBaseUrl } from '@epsx/shared-core';
import type { Logger } from '@epsx/shared-core';
import type { ApiResponse, RequestConfig } from '@epsx/types';
import { CookieManager } from '../cookie-manager';

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
      (logger as Logger).debug('Making HTTP request', { url, method: config.method || 'GET' }, {
        component: 'BaseHttpClient',
        action: 'request'
      });

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
      
      let data;
      if (contentType?.includes('application/json')) {
        data = await response.json();
      } else {
        data = await response.text();
      }

      if (!response.ok) {
        (logger as Logger).warn('HTTP request failed', {
          url,
          status: response.status,
          statusText: response.statusText,
          responseData: data
        }, {
          component: 'BaseHttpClient',
          action: 'request'
        });

        return {
          error: data?.error || data || 'Request failed',
          details: data?.details || `HTTP ${response.status}`,
        };
      }

      (logger as Logger).debug('HTTP request successful', { url, status: response.status }, {
        component: 'BaseHttpClient',
        action: 'request'
      });

      return { data };
    } catch (error) {
      (logger as Logger).error('HTTP request exception', { 
        url, 
        error: error instanceof Error ? error.message : String(error) 
      }, {
        component: 'BaseHttpClient',
        action: 'request'
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
    data?: any,
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
    data?: any,
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
    data?: any,
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