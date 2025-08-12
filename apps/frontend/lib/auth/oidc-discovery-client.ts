// OIDC Discovery Client for Backend Integration
// Connects to the sophisticated OIDC backend at /oauth/v2/*

export interface OIDCConfiguration {
  issuer: string;
  authorization_endpoint: string;
  token_endpoint: string;
  userinfo_endpoint: string;
  jwks_uri: string;
  end_session_endpoint: string;
  scopes_supported: string[];
  response_types_supported: string[];
  grant_types_supported: string[];
  code_challenge_methods_supported: string[];
  subject_types_supported: string[];
  id_token_signing_alg_values_supported: string[];
}

export interface TenantInfo {
  tenant_id: string;
  domain: string;
  name: string;
  provider_type: 'firebase' | 'oauth2' | 'oidc';
  discovery_endpoint?: string;
}

/**
 * OIDC Discovery Client
 * Integrates with backend's sophisticated OIDC provider system
 */
export class OIDCDiscoveryClient {
  private baseUrl: string;
  private configCache = new Map<string, { config: OIDCConfiguration; expires: number }>();
  private tenantCache = new Map<string, { tenants: TenantInfo[]; expires: number }>();
  
  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  }

  /**
   * Discover OIDC configuration for a tenant
   */
  async discoverConfiguration(tenantId?: string): Promise<OIDCConfiguration> {
    const cacheKey = tenantId || 'default';
    const cached = this.configCache.get(cacheKey);
    
    if (cached && cached.expires > Date.now()) {
      return cached.config;
    }

    try {
      // Use backend's OIDC discovery endpoint
      const discoveryUrl = tenantId 
        ? `${this.baseUrl}/oauth/v2/${tenantId}/.well-known/openid-configuration`
        : `${this.baseUrl}/oauth/v2/.well-known/openid-configuration`;
      
      console.log('🔍 Discovering OIDC configuration from:', discoveryUrl);
      
      const response = await fetch(discoveryUrl, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
        cache: 'no-cache'
      });

      if (!response.ok) {
        throw new Error(`OIDC discovery failed: ${response.status} ${response.statusText}`);
      }

      const config: OIDCConfiguration = await response.json();
      
      // Validate required endpoints
      this.validateConfiguration(config);
      
      // Cache for 1 hour
      this.configCache.set(cacheKey, {
        config,
        expires: Date.now() + 60 * 60 * 1000
      });
      
      console.log('✅ OIDC configuration discovered:', config.issuer);
      return config;
      
    } catch (error) {
      console.error('❌ OIDC discovery failed:', error);
      throw error;
    }
  }

  /**
   * Detect available tenants
   */
  async discoverTenants(domain?: string): Promise<TenantInfo[]> {
    const cacheKey = domain || 'all';
    const cached = this.tenantCache.get(cacheKey);
    
    if (cached && cached.expires > Date.now()) {
      return cached.tenants;
    }

    try {
      let url = `${this.baseUrl}/oauth/v2/tenants`;
      if (domain) {
        url += `?domain=${encodeURIComponent(domain)}`;
      }
      
      console.log('🔍 Discovering tenants from:', url);
      
      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
        cache: 'no-cache'
      });

      if (!response.ok) {
        throw new Error(`Tenant discovery failed: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();
      const tenants: TenantInfo[] = Array.isArray(data) ? data : data.tenants || [];
      
      // Cache for 30 minutes
      this.tenantCache.set(cacheKey, {
        tenants,
        expires: Date.now() + 30 * 60 * 1000
      });
      
      console.log('✅ Discovered tenants:', tenants.length);
      return tenants;
      
    } catch (error) {
      console.error('❌ Tenant discovery failed:', error);
      // Return empty array rather than throwing to allow fallback
      return [];
    }
  }

  /**
   * Detect tenant from domain or email
   */
  async detectTenant(identifier: string): Promise<TenantInfo | null> {
    try {
      const url = `${this.baseUrl}/oauth/v2/tenant-detection`;
      
      console.log('🔍 Detecting tenant for:', identifier);
      
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ identifier }),
      });

      if (!response.ok) {
        if (response.status === 404) {
          return null; // No tenant found
        }
        throw new Error(`Tenant detection failed: ${response.status} ${response.statusText}`);
      }

      const tenant: TenantInfo = await response.json();
      console.log('✅ Detected tenant:', tenant.tenant_id, tenant.domain);
      
      return tenant;
      
    } catch (error) {
      console.error('❌ Tenant detection failed:', error);
      return null;
    }
  }

  /**
   * Get JWKS for token verification
   */
  async getJWKS(tenantId?: string): Promise<any> {
    const config = await this.discoverConfiguration(tenantId);
    
    try {
      console.log('🔑 Fetching JWKS from:', config.jwks_uri);
      
      const response = await fetch(config.jwks_uri, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
        cache: 'default' // Allow caching for JWKS
      });

      if (!response.ok) {
        throw new Error(`JWKS fetch failed: ${response.status} ${response.statusText}`);
      }

      return await response.json();
      
    } catch (error) {
      console.error('❌ JWKS fetch failed:', error);
      throw error;
    }
  }

  /**
   * Clear caches
   */
  clearCache(): void {
    this.configCache.clear();
    this.tenantCache.clear();
    console.log('🧹 OIDC discovery cache cleared');
  }

  /**
   * Validate OIDC configuration
   */
  private validateConfiguration(config: OIDCConfiguration): void {
    const required = [
      'issuer',
      'authorization_endpoint',
      'token_endpoint',
      'userinfo_endpoint',
      'jwks_uri'
    ];

    for (const field of required) {
      if (!config[field as keyof OIDCConfiguration]) {
        throw new Error(`Missing required OIDC configuration field: ${field}`);
      }
    }

    // Validate PKCE support
    if (!config.code_challenge_methods_supported?.includes('S256')) {
      console.warn('⚠️ Backend OIDC provider does not support PKCE S256');
    }
  }
}

// Singleton instance
let discoveryClient: OIDCDiscoveryClient | null = null;

export function getOIDCDiscoveryClient(): OIDCDiscoveryClient {
  if (!discoveryClient) {
    discoveryClient = new OIDCDiscoveryClient();
  }
  return discoveryClient;
}