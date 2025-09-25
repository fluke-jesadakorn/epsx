/**
 * Enterprise API Client
 * TypeScript client for Web3 Enterprise API endpoints
 * Handles authentication, marketplace, billing, and enterprise features
 */

import { enterpriseUrls } from '@/config/env';

// Enterprise API Types
export interface EnterpriseUser {
  wallet_address: string;
  enterprise_tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale';
  permissions: string[];
  has_api_access: boolean;
  verified_tokens_usd: number;
  nft_collections: string[];
  dao_memberships: string[];
}

export interface MarketplaceProduct {
  id: string;
  name: string;
  description: string;
  category: string;
  price_usd: number;
  billing_type: 'one_time' | 'monthly' | 'annual';
  features: string[];
  tier_requirement: 'Starter' | 'Business' | 'Enterprise' | 'Whale';
  is_featured: boolean;
  tags: string[];
}

export interface IntegrationProduct {
  id: string;
  name: string;
  provider: string;
  description: string;
  setup_complexity: 'easy' | 'medium' | 'complex';
  integration_time_hours: number;
  price_usd: number;
  supported_networks: string[];
}

export interface ProfessionalService {
  id: string;
  name: string;
  description: string;
  service_type: 'consultation' | 'implementation' | 'audit' | 'training';
  duration_weeks: number;
  price_usd: number;
  expertise_required: string[];
  deliverables: string[];
}

export interface ShoppingCart {
  id: string;
  wallet_address: string;
  items: CartItem[];
  total_usd: number;
  estimated_gas_fee: number;
  currency: 'USDC' | 'USDT' | 'ETH';
  created_at: string;
  expires_at: string;
}

export interface CartItem {
  product_id: string;
  product_type: 'product' | 'integration' | 'service';
  quantity: number;
  unit_price_usd: number;
  total_price_usd: number;
  customizations?: Record<string, any>;
}

export interface BillingSubscription {
  id: string;
  wallet_address: string;
  product_id: string;
  status: 'active' | 'paused' | 'cancelled' | 'expired';
  billing_cycle: 'monthly' | 'annual';
  next_billing_date: string;
  amount_usd: number;
  payment_token: string;
  auto_renewal: boolean;
}

export interface PaymentHistory {
  id: string;
  wallet_address: string;
  transaction_hash: string;
  amount_usd: number;
  token_symbol: string;
  network: string;
  status: 'pending' | 'confirmed' | 'failed';
  created_at: string;
  items: CartItem[];
}

export interface ApiKey {
  id: string;
  name: string;
  key: string;
  tier: string;
  permissions: string[];
  rate_limit_per_minute: number;
  created_at: string;
  last_used_at?: string;
  expires_at?: string;
}

// API Response Types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
  has_next: boolean;
  has_prev: boolean;
}

// Enterprise API Client Class
export class EnterpriseApiClient {
  private baseUrl: string;
  private accessToken: string | null = null;

  constructor() {
    this.baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  }

  // Authentication Management
  setAccessToken(token: string) {
    this.accessToken = token;
  }

  clearAccessToken() {
    this.accessToken = null;
  }

  private async getAccessToken(): Promise<string | null> {
    if (this.accessToken) {
      return this.accessToken;
    }

    // Try to get token from cookies via session API
    try {
      const response = await fetch('/api/auth/session', {
        credentials: 'include',
      });
      
      if (response.ok) {
        const session = await response.json();
        if (session.isAuthenticated) {
          // Extract token from cookie or session
          return 'session_token'; // Placeholder - actual implementation would extract real token
        }
      }
    } catch (error) {
      console.warn('Failed to get access token from session:', error);
    }

    return null;
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const accessToken = await this.getAccessToken();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...options.headers as Record<string, string>,
    };

    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        ...options,
        headers,
        credentials: 'include',
      });

      const data = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: data.error || `HTTP ${response.status}`,
        };
      }

      return {
        success: true,
        data,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  // Authentication Methods
  async getEnterprisePermissions(): Promise<ApiResponse<EnterpriseUser>> {
    return this.makeRequest<EnterpriseUser>('/api/v1/enterprise/auth/permissions');
  }

  async challengeWallet(walletAddress: string): Promise<ApiResponse<{ message: string; nonce: string }>> {
    return this.makeRequest('/api/v1/enterprise/auth/challenge', {
      method: 'POST',
      body: JSON.stringify({ wallet_address: walletAddress }),
    });
  }

  async verifySignature(
    walletAddress: string,
    signature: string,
    message: string,
    nonce: string
  ): Promise<ApiResponse<EnterpriseUser>> {
    return this.makeRequest<EnterpriseUser>('/api/v1/enterprise/auth/verify', {
      method: 'POST',
      body: JSON.stringify({
        wallet_address: walletAddress,
        signature,
        message,
        nonce,
      }),
    });
  }

  // Marketplace Methods
  async getMarketplaceCatalog(params?: {
    category?: string;
    tier?: string;
    search?: string;
    page?: number;
    per_page?: number;
  }): Promise<ApiResponse<PaginatedResponse<MarketplaceProduct>>> {
    const queryString = params ? new URLSearchParams(params as any).toString() : '';
    return this.makeRequest<PaginatedResponse<MarketplaceProduct>>(
      `/api/v1/enterprise/marketplace/catalog${queryString ? `?${queryString}` : ''}`
    );
  }

  async getProduct(productId: string): Promise<ApiResponse<MarketplaceProduct>> {
    return this.makeRequest<MarketplaceProduct>(`/api/v1/enterprise/marketplace/products/${productId}`);
  }

  async getIntegrations(params?: {
    provider?: string;
    complexity?: string;
    network?: string;
  }): Promise<ApiResponse<IntegrationProduct[]>> {
    const queryString = params ? new URLSearchParams(params as any).toString() : '';
    return this.makeRequest<IntegrationProduct[]>(
      `/api/v1/enterprise/marketplace/integrations${queryString ? `?${queryString}` : ''}`
    );
  }

  async getProfessionalServices(params?: {
    service_type?: string;
    expertise?: string;
  }): Promise<ApiResponse<ProfessionalService[]>> {
    const queryString = params ? new URLSearchParams(params as any).toString() : '';
    return this.makeRequest<ProfessionalService[]>(
      `/api/v1/enterprise/marketplace/services${queryString ? `?${queryString}` : ''}`
    );
  }

  async getRecommendations(): Promise<ApiResponse<{
    products: MarketplaceProduct[];
    integrations: IntegrationProduct[];
    services: ProfessionalService[];
  }>> {
    return this.makeRequest('/api/v1/enterprise/marketplace/recommendations');
  }

  // Shopping Cart Methods
  async getCart(): Promise<ApiResponse<ShoppingCart>> {
    return this.makeRequest<ShoppingCart>('/api/v1/enterprise/marketplace/cart');
  }

  async addToCart(item: {
    product_id: string;
    product_type: 'product' | 'integration' | 'service';
    quantity?: number;
    customizations?: Record<string, any>;
  }): Promise<ApiResponse<ShoppingCart>> {
    return this.makeRequest<ShoppingCart>('/api/v1/enterprise/marketplace/cart/add', {
      method: 'POST',
      body: JSON.stringify(item),
    });
  }

  async removeFromCart(productId: string): Promise<ApiResponse<ShoppingCart>> {
    return this.makeRequest<ShoppingCart>('/api/v1/enterprise/marketplace/cart/remove', {
      method: 'POST',
      body: JSON.stringify({ product_id: productId }),
    });
  }

  async updateCartItem(item: {
    product_id: string;
    quantity: number;
    customizations?: Record<string, any>;
  }): Promise<ApiResponse<ShoppingCart>> {
    return this.makeRequest<ShoppingCart>('/api/v1/enterprise/marketplace/cart/update', {
      method: 'POST',
      body: JSON.stringify(item),
    });
  }

  async clearCart(): Promise<ApiResponse<{ message: string }>> {
    return this.makeRequest('/api/v1/enterprise/marketplace/cart/clear', {
      method: 'POST',
    });
  }

  // Checkout and Payment Methods
  async processCheckout(params: {
    payment_token: 'USDC' | 'USDT' | 'ETH';
    network: string;
    gas_price?: number;
  }): Promise<ApiResponse<{
    transaction_hash: string;
    payment_id: string;
    total_amount: number;
    gas_estimate: number;
  }>> {
    return this.makeRequest('/api/v1/enterprise/marketplace/checkout', {
      method: 'POST',
      body: JSON.stringify(params),
    });
  }

  async confirmPayment(paymentId: string, transactionHash: string): Promise<ApiResponse<{
    status: string;
    subscriptions_created: BillingSubscription[];
  }>> {
    return this.makeRequest('/api/v1/enterprise/marketplace/confirm-payment', {
      method: 'POST',
      body: JSON.stringify({
        payment_id: paymentId,
        transaction_hash: transactionHash,
      }),
    });
  }

  // Billing Methods
  async getBillingOverview(): Promise<ApiResponse<{
    total_spent_usd: number;
    active_subscriptions: number;
    next_payment_due: string;
    billing_status: string;
  }>> {
    return this.makeRequest('/api/v1/enterprise/billing/overview');
  }

  async getSubscriptions(): Promise<ApiResponse<BillingSubscription[]>> {
    return this.makeRequest<BillingSubscription[]>('/api/v1/enterprise/billing/subscriptions');
  }

  async getPaymentHistory(params?: {
    limit?: number;
    offset?: number;
    status?: string;
  }): Promise<ApiResponse<PaginatedResponse<PaymentHistory>>> {
    const queryString = params ? new URLSearchParams(params as any).toString() : '';
    return this.makeRequest<PaginatedResponse<PaymentHistory>>(
      `/api/v1/enterprise/billing/payments${queryString ? `?${queryString}` : ''}`
    );
  }

  async pauseSubscription(subscriptionId: string): Promise<ApiResponse<BillingSubscription>> {
    return this.makeRequest<BillingSubscription>(`/api/v1/enterprise/billing/subscriptions/${subscriptionId}/pause`, {
      method: 'POST',
    });
  }

  async resumeSubscription(subscriptionId: string): Promise<ApiResponse<BillingSubscription>> {
    return this.makeRequest<BillingSubscription>(`/api/v1/enterprise/billing/subscriptions/${subscriptionId}/resume`, {
      method: 'POST',
    });
  }

  async cancelSubscription(subscriptionId: string): Promise<ApiResponse<BillingSubscription>> {
    return this.makeRequest<BillingSubscription>(`/api/v1/enterprise/billing/subscriptions/${subscriptionId}/cancel`, {
      method: 'POST',
    });
  }

  // API Key Management
  async getApiKeys(): Promise<ApiResponse<ApiKey[]>> {
    return this.makeRequest<ApiKey[]>('/api/v1/enterprise/billing/api-keys');
  }

  async createApiKey(name: string): Promise<ApiResponse<ApiKey>> {
    return this.makeRequest<ApiKey>('/api/v1/enterprise/billing/api-keys', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  async deleteApiKey(keyId: string): Promise<ApiResponse<{ message: string }>> {
    return this.makeRequest(`/api/v1/enterprise/billing/api-keys/${keyId}`, {
      method: 'DELETE',
    });
  }

  // Analytics Methods
  async getAnalyticsOverview(): Promise<ApiResponse<{
    total_api_calls: number;
    total_data_points: number;
    success_rate: number;
    average_response_time: number;
    top_endpoints: Array<{ endpoint: string; calls: number }>;
  }>> {
    return this.makeRequest('/api/v1/enterprise/analytics/overview');
  }

  async getUsageMetrics(params?: {
    start_date?: string;
    end_date?: string;
    granularity?: 'hour' | 'day' | 'week' | 'month';
  }): Promise<ApiResponse<{
    api_calls: Array<{ timestamp: string; count: number }>;
    data_transfer: Array<{ timestamp: string; bytes: number }>;
    error_rate: Array<{ timestamp: string; rate: number }>;
  }>> {
    const queryString = params ? new URLSearchParams(params as any).toString() : '';
    return this.makeRequest(
      `/api/v1/enterprise/analytics/usage${queryString ? `?${queryString}` : ''}`
    );
  }

  // Enterprise Status Methods
  async getEnterpriseStatus(): Promise<ApiResponse<{
    api_status: string;
    supported_chains: string[];
    supported_tiers: string[];
    features: Record<string, boolean>;
    rate_limits: Record<string, string>;
  }>> {
    return this.makeRequest('/api/v1/enterprise/status');
  }

  async getEnterpriseTiers(): Promise<ApiResponse<{
    tiers: Record<string, {
      name: string;
      description: string;
      minimum_token_value_usd: number;
      rate_limit_per_minute: number;
      features: string[];
      support: string;
    }>;
    compliance_requirements: Record<string, string[]>;
    supported_tokens: Record<string, string[]>;
  }>> {
    return this.makeRequest('/api/v1/enterprise/tiers');
  }
}

// Global enterprise API client instance
export const enterpriseApi = new EnterpriseApiClient();

// Hook for using enterprise API client in React components
export function useEnterpriseApi() {
  return enterpriseApi;
}

// Helper functions for common operations
export async function fetchUserTierStatus(): Promise<{
  tier: string;
  tokenValue: number;
  hasApiAccess: boolean;
  permissions: string[];
} | null> {
  const response = await enterpriseApi.getEnterprisePermissions();
  
  if (response.success && response.data) {
    return {
      tier: response.data.enterprise_tier,
      tokenValue: response.data.verified_tokens_usd,
      hasApiAccess: response.data.has_api_access,
      permissions: response.data.permissions,
    };
  }
  
  return null;
}

export async function checkEnterpriseAccess(requiredTier: string): Promise<boolean> {
  const tierStatus = await fetchUserTierStatus();
  if (!tierStatus) return false;
  
  const tierHierarchy = { 'Starter': 1, 'Business': 2, 'Enterprise': 3, 'Whale': 4 };
  const userLevel = tierHierarchy[tierStatus.tier as keyof typeof tierHierarchy] || 0;
  const requiredLevel = tierHierarchy[requiredTier as keyof typeof tierHierarchy] || 1;
  
  return userLevel >= requiredLevel;
}