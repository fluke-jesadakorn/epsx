'use client';

// Minimal admin auth API client that calls the centralized backend

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  session_type: string;
  expires_at: string;
}

interface LoginRequest {
  type: 'admin';
  email: string;
  password: string;
}

interface LoginResponse {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  expires_at: string;
  session_type: string;
  access_token: string;
  token_type: string;
}

interface PermissionResponse {
  feature: string;
  has_access: boolean;
  reason?: string;
}

interface NavigationResponse {
  items: Array<{
    name: string;
    path: string;
    enabled: boolean;
    required_permission?: string;
    required_role?: string;
  }>;
  user_role: string;
  permissions: string[];
}

interface FeaturesResponse {
  user_id: string;
  role: string;
  subscription_tier: string;
  features: Array<{
    feature: string;
    enabled: boolean;
    tier_required: string;
    permission_required?: string;
  }>;
  permissions: string[];
}

class AdminAuthAPI {
  private baseUrl: string;

  constructor() {
    this.baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  }

  private async getBearerToken(): Promise<string | null> {
    // Get token from server-side cookie via API route
    try {
      const response = await fetch('/api/auth/token', {
        credentials: 'same-origin',
      });
      
      if (!response.ok) {
        return null;
      }
      
      const data = await response.json();
      return data.token || null;
    } catch {
      return null;
    }
  }

  private async apiCall<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}/api/v1${endpoint}`;
    
    // Get bearer token for authorization
    const token = await this.getBearerToken();
    const authHeaders = token ? { 'Authorization': `Bearer ${token}` } : {};
    
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders,
        ...options.headers,
      },
      ...options,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`API Error ${response.status}: ${error}`);
    }

    return response.json();
  }

  // Authentication operations
  async login(email: string, password: string): Promise<LoginResponse> {
    const loginRequest: LoginRequest = {
      type: 'admin',
      email,
      password,
    };

    return this.apiCall<LoginResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(loginRequest),
    });
  }

  async logout(): Promise<void> {
    await this.apiCall<void>('/auth/logout', {
      method: 'POST',
    });
  }

  async getCurrentUser(): Promise<AuthUser> {
    return this.apiCall<AuthUser>('/auth/profile');
  }

  async refreshSession(): Promise<{ expires_at: string }> {
    return this.apiCall<{ expires_at: string }>('/auth/refresh', {
      method: 'POST',
    });
  }

  // Permission checking
  async checkPermission(feature: string): Promise<PermissionResponse> {
    return this.apiCall<PermissionResponse>(`/auth/permission?feature=${encodeURIComponent(feature)}`);
  }

  async getNavigation(): Promise<NavigationResponse> {
    return this.apiCall<NavigationResponse>('/auth/navigation');
  }

  async getFeatures(): Promise<FeaturesResponse> {
    return this.apiCall<FeaturesResponse>('/auth/features');
  }

  // Validation helpers (for middleware)
  async validateSession(): Promise<AuthUser> {
    return this.apiCall<AuthUser>('/auth/validate-session', {
      method: 'POST',
      body: JSON.stringify({ app_type: 'admin' }),
    });
  }

  async validateRoute(route: string, method: string = 'GET'): Promise<{
    allowed: boolean;
    reason?: string;
    required_permissions: string[];
    user_permissions: string[];
  }> {
    return this.apiCall('/auth/validate-access', {
      method: 'POST',
      body: JSON.stringify({
        route,
        method,
        app_type: 'admin',
      }),
    });
  }
}

// Export singleton instance
export const adminAuthAPI = new AdminAuthAPI();

// Utility functions for easy use in components
export const authUtils = {
  async isLoggedIn(): Promise<boolean> {
    try {
      await adminAuthAPI.getCurrentUser();
      return true;
    } catch {
      return false;
    }
  },

  async hasPermission(feature: string): Promise<boolean> {
    try {
      const result = await adminAuthAPI.checkPermission(feature);
      return result.has_access;
    } catch {
      return false;
    }
  },

  async canAccessRoute(route: string): Promise<boolean> {
    try {
      const result = await adminAuthAPI.validateRoute(route);
      return result.allowed;
    } catch {
      return false;
    }
  },
};