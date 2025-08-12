// Tenant Detection Service
// Smart tenant routing based on email domains and user input

import { getOIDCDiscoveryClient, type TenantInfo } from './oidc-discovery-client';

export interface TenantDetectionResult {
  tenant: TenantInfo | null;
  confidence: 'high' | 'medium' | 'low' | 'none';
  method: 'domain' | 'email' | 'manual' | 'default';
  suggestions?: TenantInfo[];
}

export interface TenantPreference {
  tenantId: string;
  domain: string;
  lastUsed: number;
  useCount: number;
}

/**
 * Tenant Detection Service
 * Intelligently routes users to the correct OIDC provider/tenant
 */
export class TenantDetectionService {
  private discoveryClient = getOIDCDiscoveryClient();
  private preferences: TenantPreference[] = [];
  private readonly STORAGE_KEY = 'epsx_tenant_preferences';

  constructor() {
    this.loadPreferences();
  }

  /**
   * Detect tenant from user input
   */
  async detectTenant(input: string): Promise<TenantDetectionResult> {
    const normalizedInput = input.trim().toLowerCase();
    
    // Try different detection methods
    const results = await Promise.allSettled([
      this.detectByEmail(normalizedInput),
      this.detectByDomain(normalizedInput),
      this.detectByPreferences(normalizedInput),
    ]);

    // Find the best result
    const validResults = results
      .filter((r): r is PromiseFulfilledResult<TenantDetectionResult> => r.status === 'fulfilled')
      .map(r => r.value)
      .filter(r => r.tenant !== null)
      .sort((a, b) => this.compareConfidence(b.confidence, a.confidence));

    if (validResults.length > 0) {
      const best = validResults[0];
      // Update preferences if we found a good match
      if (best.confidence === 'high' && best.tenant) {
        this.updatePreferences(best.tenant);
      }
      return {
        ...best,
        suggestions: validResults.slice(1, 4).map(r => r.tenant!).filter(Boolean)
      };
    }

    // Fallback: get available tenants as suggestions
    try {
      const allTenants = await this.discoveryClient.discoverTenants();
      return {
        tenant: null,
        confidence: 'none',
        method: 'default',
        suggestions: allTenants.slice(0, 3)
      };
    } catch (error) {
      console.error('Failed to get tenant suggestions:', error);
      return {
        tenant: null,
        confidence: 'none',
        method: 'default'
      };
    }
  }

  /**
   * Detect tenant by email address
   */
  private async detectByEmail(email: string): Promise<TenantDetectionResult> {
    if (!this.isValidEmail(email)) {
      return { tenant: null, confidence: 'none', method: 'email' };
    }

    try {
      const tenant = await this.discoveryClient.detectTenant(email);
      return {
        tenant,
        confidence: tenant ? 'high' : 'none',
        method: 'email'
      };
    } catch (error) {
      console.debug('Email-based tenant detection failed:', error);
      return { tenant: null, confidence: 'none', method: 'email' };
    }
  }

  /**
   * Detect tenant by domain
   */
  private async detectByDomain(domain: string): Promise<TenantDetectionResult> {
    try {
      // Extract domain from email if needed
      let cleanDomain = domain;
      if (domain.includes('@')) {
        cleanDomain = domain.split('@')[1];
      }

      // Remove common email suffixes for corporate matching
      cleanDomain = this.normalizeDomain(cleanDomain);

      const tenants = await this.discoveryClient.discoverTenants(cleanDomain);
      
      if (tenants.length === 0) {
        return { tenant: null, confidence: 'none', method: 'domain' };
      }

      // Exact domain match gets high confidence
      const exactMatch = tenants.find(t => t.domain.toLowerCase() === cleanDomain.toLowerCase());
      if (exactMatch) {
        return {
          tenant: exactMatch,
          confidence: 'high',
          method: 'domain'
        };
      }

      // Partial matches get medium confidence
      const partialMatches = tenants.filter(t => 
        t.domain.toLowerCase().includes(cleanDomain.toLowerCase()) ||
        cleanDomain.toLowerCase().includes(t.domain.toLowerCase())
      );

      if (partialMatches.length > 0) {
        return {
          tenant: partialMatches[0],
          confidence: 'medium',
          method: 'domain',
          suggestions: partialMatches.slice(1)
        };
      }

      return {
        tenant: tenants[0],
        confidence: 'low',
        method: 'domain',
        suggestions: tenants.slice(1)
      };

    } catch (error) {
      console.debug('Domain-based tenant detection failed:', error);
      return { tenant: null, confidence: 'none', method: 'domain' };
    }
  }

  /**
   * Detect tenant by user preferences
   */
  private async detectByPreferences(input: string): Promise<TenantDetectionResult> {
    if (!input || this.preferences.length === 0) {
      return { tenant: null, confidence: 'none', method: 'manual' };
    }

    try {
      // Check if input matches any preferred domain
      const domain = this.isValidEmail(input) ? input.split('@')[1] : input;
      const matchingPref = this.preferences.find(p => 
        p.domain.toLowerCase() === domain.toLowerCase()
      );

      if (matchingPref) {
        // Fetch the tenant details
        const tenants = await this.discoveryClient.discoverTenants(matchingPref.domain);
        const tenant = tenants.find(t => t.tenant_id === matchingPref.tenantId);
        
        if (tenant) {
          return {
            tenant,
            confidence: 'medium',
            method: 'manual'
          };
        }
      }

      return { tenant: null, confidence: 'none', method: 'manual' };

    } catch (error) {
      console.debug('Preference-based tenant detection failed:', error);
      return { tenant: null, confidence: 'none', method: 'manual' };
    }
  }

  /**
   * Get user's preferred tenants
   */
  getPreferredTenants(): TenantPreference[] {
    return [...this.preferences].sort((a, b) => {
      // Sort by recent usage and frequency
      const scoreA = a.lastUsed + (a.useCount * 1000);
      const scoreB = b.lastUsed + (b.useCount * 1000);
      return scoreB - scoreA;
    });
  }

  /**
   * Manually set preferred tenant
   */
  setPreferredTenant(tenant: TenantInfo): void {
    this.updatePreferences(tenant);
  }

  /**
   * Clear tenant preferences
   */
  clearPreferences(): void {
    this.preferences = [];
    this.savePreferences();
  }

  /**
   * Update tenant preferences
   */
  private updatePreferences(tenant: TenantInfo): void {
    const existing = this.preferences.find(p => p.tenantId === tenant.tenant_id);
    
    if (existing) {
      existing.lastUsed = Date.now();
      existing.useCount += 1;
    } else {
      this.preferences.push({
        tenantId: tenant.tenant_id,
        domain: tenant.domain,
        lastUsed: Date.now(),
        useCount: 1
      });
    }

    // Keep only the 10 most recent preferences
    this.preferences = this.preferences
      .sort((a, b) => b.lastUsed - a.lastUsed)
      .slice(0, 10);

    this.savePreferences();
  }

  /**
   * Load preferences from localStorage
   */
  private loadPreferences(): void {
    if (typeof window === 'undefined') return;

    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      if (stored) {
        this.preferences = JSON.parse(stored);
      }
    } catch (error) {
      console.debug('Failed to load tenant preferences:', error);
      this.preferences = [];
    }
  }

  /**
   * Save preferences to localStorage
   */
  private savePreferences(): void {
    if (typeof window === 'undefined') return;

    try {
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(this.preferences));
    } catch (error) {
      console.debug('Failed to save tenant preferences:', error);
    }
  }

  /**
   * Compare confidence levels
   */
  private compareConfidence(a: string, b: string): number {
    const levels = { high: 3, medium: 2, low: 1, none: 0 };
    return levels[a as keyof typeof levels] - levels[b as keyof typeof levels];
  }

  /**
   * Validate email format
   */
  private isValidEmail(email: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  }

  /**
   * Normalize domain for matching
   */
  private normalizeDomain(domain: string): string {
    return domain
      .toLowerCase()
      .replace(/^www\./, '') // Remove www prefix
      .replace(/\.com$|\.net$|\.org$/, '') // Remove common TLDs for corporate matching
      .trim();
  }
}

// Singleton instance
let tenantDetectionService: TenantDetectionService | null = null;

export function getTenantDetectionService(): TenantDetectionService {
  if (!tenantDetectionService) {
    tenantDetectionService = new TenantDetectionService();
  }
  return tenantDetectionService;
}