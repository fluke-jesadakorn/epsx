import { API_ROUTES } from '../../../shared/config/route-constants';
import { getBackendUrl } from '../../../shared/utils/url-resolver';
import { PureWeb3AuthStore } from './types';

// Define a minimal interface for the store to avoid heavy type deps
type AuthStoreGetter = () => PureWeb3AuthStore;
type AuthStore = { getState: AuthStoreGetter };

export class PureWeb3ApiClient {
    protected baseUrl: string;
    protected authStore: AuthStore;

    constructor(authStore: AuthStore) {
        this.baseUrl = getBackendUrl('client');
        this.authStore = authStore;
    }

    // Generic request method with automatic signing
    async request<T>(
        endpoint: string,
        options: {
            method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
            body?: any;
            headers?: Record<string, string>;
        } = {}
    ): Promise<T> {
        const { method = 'GET', body, headers = {} } = options;

        try {
            // Get signed headers
            const signedHeaders = await this.authStore.getState().signRequest(
                endpoint,
                method,
                body
            );

            // Prepare request
            const requestOptions: RequestInit = {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    ...headers,
                    ...signedHeaders,
                },
            };

            if (body && (method === 'POST' || method === 'PUT')) {
                requestOptions.body = JSON.stringify(body);
            }

            // Make request
            const response = await fetch(`${this.baseUrl}${endpoint}`, requestOptions);

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error(errorData.message || `Request failed: ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Pure Web3 API request failed [${method} ${endpoint}]:`, error);
            throw error;
        }
    }

    // Convenience methods
    async get<T>(endpoint: string, headers?: Record<string, string>): Promise<T> {
        return this.request<T>(endpoint, { method: 'GET', headers });
    }

    async post<T>(endpoint: string, body?: any, headers?: Record<string, string>): Promise<T> {
        return this.request<T>(endpoint, { method: 'POST', body, headers });
    }

    async put<T>(endpoint: string, body?: any, headers?: Record<string, string>): Promise<T> {
        return this.request<T>(endpoint, { method: 'PUT', body, headers });
    }

    async delete<T>(endpoint: string, headers?: Record<string, string>): Promise<T> {
        return this.request<T>(endpoint, { method: 'DELETE', headers });
    }

    // Common API methods
    async getWalletStatus() {
        return this.get(API_ROUTES.PUBLIC.STATUS); // Or auth status?
    }

    async getWalletPermissions(includeExpired = false) {
        return this.get(`${API_ROUTES.AUTH.PERMISSIONS}?include_expired=${includeExpired}`);
    }

    async getWalletProfile() {
        return this.get(API_ROUTES.AUTH.PROFILE);
    }
}
