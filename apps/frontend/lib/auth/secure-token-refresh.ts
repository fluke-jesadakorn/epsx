// Secure Token Refresh Manager
// Automatic JWT token renewal with device fingerprinting and security monitoring

interface RefreshConfig {
  refreshThresholdMinutes: number;  // Minutes before expiry to refresh
  maxRetries: number;               // Maximum refresh attempts
  backoffMultiplier: number;        // Exponential backoff base (ms)
  monitoringEnabled: boolean;       // Enable security monitoring
}

interface TokenPayload {
  sub: string;
  exp: number;
  iat: number;
  permissions?: string[];
  device_fingerprint?: string;
  jti?: string;
}

interface RefreshResponse {
  access_token: string;
  expires_in: number;
  success: boolean;
  error?: string;
}

interface SecurityMetrics {
  refreshAttempts: number;
  failedRefreshes: number;
  lastSuccessfulRefresh: number;
  securityWarnings: string[];
}

/**
 * Secure Token Refresh Manager
 * 
 * Features:
 * - Automatic background token refresh 5 minutes before expiry
 * - Device fingerprinting for security
 * - Exponential backoff on failures
 * - Security monitoring and anomaly detection
 * - Seamless user experience with zero interruption
 * - Graceful fallback to login on persistent failures
 */
export class SecureTokenRefreshManager {
  private refreshTimer?: NodeJS.Timeout;
  private isRefreshing = false;
  private refreshPromise?: Promise<boolean>;
  private config: RefreshConfig;
  private deviceFingerprint?: string;
  private securityMetrics: SecurityMetrics;
  
  constructor(config: Partial<RefreshConfig> = {}) {
    this.config = {
      refreshThresholdMinutes: 5,
      maxRetries: 3,
      backoffMultiplier: 1000,
      monitoringEnabled: true,
      ...config
    };
    
    this.securityMetrics = {
      refreshAttempts: 0,
      failedRefreshes: 0,
      lastSuccessfulRefresh: 0,
      securityWarnings: []
    };
    
    // Initialize device fingerprint
    this.initializeDeviceFingerprint();
    
    // Listen for token refresh events from other tabs
    if (typeof window !== 'undefined') {
      window.addEventListener('storage', this.handleStorageChange.bind(this));
      window.addEventListener('beforeunload', this.cleanup.bind(this));
    }
  }
  
  /**
   * Schedule automatic token refresh based on token expiry
   */
  public scheduleRefresh(accessToken: string): void {
    const payload = this.parseJwtPayload(accessToken);
    if (!payload) {
      console.warn('🚨 Invalid JWT token format, cannot schedule refresh');
      return;
    }
    
    // Security check: Validate device fingerprint
    if (payload.device_fingerprint && this.deviceFingerprint) {
      if (payload.device_fingerprint !== this.deviceFingerprint) {
        this.addSecurityWarning('Device fingerprint mismatch detected');
        console.warn('🚨 Device fingerprint mismatch - possible token theft');
      }
    }
    
    const expiresAt = payload.exp * 1000; // Convert to milliseconds
    const refreshTime = expiresAt - (this.config.refreshThresholdMinutes * 60 * 1000);
    const now = Date.now();
    
    // If token expires very soon, refresh immediately
    if (refreshTime <= now + 30000) { // 30 seconds buffer
      console.log('🔄 Token expires very soon, refreshing immediately');
      this.performRefresh();
      return;
    }
    
    // Clear existing timer
    this.clearRefreshTimer();
    
    // Schedule refresh
    this.refreshTimer = setTimeout(() => {
      this.performRefresh();
    }, refreshTime - now);
    
    const refreshDate = new Date(refreshTime).toLocaleString();
    console.log(`🔄 Token refresh scheduled for ${refreshDate}`);
    
    // Store refresh schedule in sessionStorage for monitoring
    if (typeof window !== 'undefined') {
      sessionStorage.setItem('tokenRefreshScheduled', refreshTime.toString());
    }
  }
  
  /**
   * Perform secure token refresh with comprehensive error handling
   */
  private async performRefresh(): Promise<boolean> {
    // Prevent multiple concurrent refreshes
    if (this.isRefreshing) {
      console.log('🔄 Refresh already in progress, waiting...');
      return this.refreshPromise || Promise.resolve(false);
    }
    
    this.isRefreshing = true;
    this.refreshPromise = this.executeRefreshWithRetry();
    
    try {
      const result = await this.refreshPromise;
      return result;
    } finally {
      this.isRefreshing = false;
      this.refreshPromise = undefined;
    }
  }
  
  /**
   * Execute refresh with retry logic and security monitoring
   */
  private async executeRefreshWithRetry(): Promise<boolean> {
    for (let attempt = 1; attempt <= this.config.maxRetries; attempt++) {
      try {
        this.securityMetrics.refreshAttempts++;
        
        console.log(`🔄 Attempting token refresh (${attempt}/${this.config.maxRetries})`);
        
        const deviceFingerprint = await this.generateDeviceFingerprint();
        
        // Attempt refresh
        const response = await fetch('/api/auth/refresh', {
          method: 'POST',
          credentials: 'include', // Include httpOnly cookies
          headers: {
            'Content-Type': 'application/json',
            'X-Requested-With': 'XMLHttpRequest', // CSRF protection
          },
          body: JSON.stringify({
            device_fingerprint: deviceFingerprint,
            client_info: {
              user_agent: navigator.userAgent,
              timestamp: Date.now(),
              timezone: Intl.DateTimeFormat().resolvedOptions().timeZone
            }
          })
        });
        
        if (response.ok) {
          const data: RefreshResponse = await response.json();
          
          if (data.success && data.access_token) {
            // Update security metrics
            this.securityMetrics.lastSuccessfulRefresh = Date.now();
            
            // Schedule next refresh
            this.scheduleRefresh(data.access_token);
            
            // Notify application of successful refresh
            this.notifyRefreshSuccess(data);
            
            console.log('✅ Token refresh successful');
            return true;
          } else {
            throw new Error(data.error || 'Refresh failed with invalid response');
          }
        } else if (response.status === 401) {
          // Refresh token expired or invalid
          console.log('🚨 Refresh token expired, redirecting to login');
          this.handleRefreshTokenExpired();
          return false;
        } else if (response.status === 403) {
          // Security violation detected
          console.error('🚨 Security violation during refresh - possible attack');
          this.addSecurityWarning('Refresh blocked due to security violation');
          this.handleSecurityViolation();
          return false;
        } else {
          // Server error - retry with backoff
          const errorText = await response.text();
          throw new Error(`Refresh failed with status ${response.status}: ${errorText}`);
        }
        
      } catch (error) {
        this.securityMetrics.failedRefreshes++;
        
        console.error(`❌ Refresh attempt ${attempt} failed:`, error);
        
        if (attempt < this.config.maxRetries) {
          // Exponential backoff with jitter
          const baseDelay = this.config.backoffMultiplier * Math.pow(2, attempt - 1);
          const jitter = Math.random() * 1000; // Add random jitter
          const delay = baseDelay + jitter;
          
          console.log(`⏳ Retrying refresh in ${Math.round(delay)}ms...`);
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }
    
    // All retries failed
    console.error('🚨 All refresh attempts failed, redirecting to login');
    this.handleRefreshFailure();
    return false;
  }
  
  /**
   * Generate device fingerprint for security
   */
  private async generateDeviceFingerprint(): Promise<string> {
    if (this.deviceFingerprint) {
      return this.deviceFingerprint;
    }
    
    try {
      // Collect browser characteristics
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.textBaseline = 'top';
        ctx.font = '14px Arial';
        ctx.fillText('EPSX Security Fingerprint', 2, 2);
      }
      const canvasFingerprint = canvas.toDataURL();
      
      // Collect system information
      const fingerprint = {
        userAgent: navigator.userAgent,
        language: navigator.language,
        languages: navigator.languages?.join(',') || '',
        platform: navigator.platform,
        screen: `${screen.width}x${screen.height}x${screen.colorDepth}`,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        canvas: canvasFingerprint.slice(-50), // Last 50 chars
        memory: (navigator as any).deviceMemory || 'unknown',
        cores: navigator.hardwareConcurrency || 'unknown',
        touch: 'ontouchstart' in window ? 'yes' : 'no'
      };
      
      // Generate stable hash
      const fingerprintString = JSON.stringify(fingerprint);
      const hashBuffer = await crypto.subtle.digest('SHA-256', 
        new TextEncoder().encode(fingerprintString)
      );
      
      const hashArray = Array.from(new Uint8Array(hashBuffer));
      const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
      
      this.deviceFingerprint = hashHex.slice(0, 32); // First 32 chars
      return this.deviceFingerprint;
      
    } catch (error) {
      console.warn('Device fingerprint generation failed, using fallback:', error);
      // Fallback fingerprint
      this.deviceFingerprint = btoa(navigator.userAgent).slice(0, 32);
      return this.deviceFingerprint;
    }
  }
  
  /**
   * Parse JWT payload without validation (client-side only)
   */
  private parseJwtPayload(token: string): TokenPayload | null {
    try {
      const parts = token.split('.');
      if (parts.length !== 3) return null;
      
      // Decode base64url
      const payload = parts[1].replace(/-/g, '+').replace(/_/g, '/');
      const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4);
      
      const decoded = JSON.parse(atob(paddedPayload));
      return decoded as TokenPayload;
    } catch (error) {
      console.warn('Failed to parse JWT payload:', error);
      return null;
    }
  }
  
  /**
   * Initialize device fingerprint on startup
   */
  private async initializeDeviceFingerprint(): Promise<void> {
    if (typeof window !== 'undefined') {
      // Check if fingerprint is cached
      const cached = sessionStorage.getItem('deviceFingerprint');
      if (cached) {
        this.deviceFingerprint = cached;
      } else {
        const fingerprint = await this.generateDeviceFingerprint();
        sessionStorage.setItem('deviceFingerprint', fingerprint);
      }
    }
  }
  
  /**
   * Handle storage events from other tabs (token refresh coordination)
   */
  private handleStorageChange(event: StorageEvent): void {
    if (event.key === 'tokenRefreshed' && event.newValue) {
      const data = JSON.parse(event.newValue);
      if (data.success) {
        console.log('🔄 Token refreshed in another tab, updating local schedule');
        this.scheduleRefresh(data.access_token);
      }
    }
  }
  
  /**
   * Notify application of successful refresh
   */
  private notifyRefreshSuccess(data: RefreshResponse): void {
    // Dispatch custom event
    if (typeof window !== 'undefined') {
      const event = new CustomEvent('tokenRefreshed', {
        detail: { 
          success: true, 
          access_token: data.access_token,
          expires_in: data.expires_in
        }
      });
      window.dispatchEvent(event);
      
      // Store in localStorage for other tabs
      localStorage.setItem('tokenRefreshed', JSON.stringify({
        success: true,
        access_token: data.access_token,
        timestamp: Date.now()
      }));
      
      // Clean up after 1 second
      setTimeout(() => {
        localStorage.removeItem('tokenRefreshed');
      }, 1000);
    }
  }
  
  /**
   * Handle refresh token expiration
   */
  private handleRefreshTokenExpired(): void {
    this.cleanup();
    
    // Notify application
    if (typeof window !== 'undefined') {
      const event = new CustomEvent('authenticationExpired', {
        detail: { reason: 'refresh_token_expired' }
      });
      window.dispatchEvent(event);
      
      // Redirect to login after brief delay
      setTimeout(() => {
        window.location.href = '/login?reason=session_expired';
      }, 2000);
    }
  }
  
  /**
   * Handle security violations
   */
  private handleSecurityViolation(): void {
    this.addSecurityWarning('Security violation during token refresh');
    
    // Notify application
    if (typeof window !== 'undefined') {
      const event = new CustomEvent('securityViolation', {
        detail: { 
          type: 'refresh_blocked',
          metrics: this.securityMetrics 
        }
      });
      window.dispatchEvent(event);
    }
    
    // Force logout for security
    this.forceLogout('security_violation');
  }
  
  /**
   * Handle persistent refresh failures
   */
  private handleRefreshFailure(): void {
    this.cleanup();
    
    // Notify application
    if (typeof window !== 'undefined') {
      const event = new CustomEvent('refreshFailed', {
        detail: { 
          attempts: this.securityMetrics.refreshAttempts,
          failures: this.securityMetrics.failedRefreshes 
        }
      });
      window.dispatchEvent(event);
      
      // Redirect to login
      setTimeout(() => {
        window.location.href = '/login?reason=refresh_failed';
      }, 2000);
    }
  }
  
  /**
   * Force logout for security reasons
   */
  private forceLogout(reason: string): void {
    console.error(`🚨 Force logout due to: ${reason}`);
    
    // Clear all tokens and data
    if (typeof window !== 'undefined') {
      sessionStorage.clear();
      localStorage.removeItem('deviceFingerprint');
    }
    
    // Redirect to login immediately
    setTimeout(() => {
      window.location.href = `/login?reason=${reason}`;
    }, 100);
  }
  
  /**
   * Add security warning
   */
  private addSecurityWarning(warning: string): void {
    this.securityMetrics.securityWarnings.push(`${new Date().toISOString()}: ${warning}`);
    
    // Keep only last 10 warnings
    if (this.securityMetrics.securityWarnings.length > 10) {
      this.securityMetrics.securityWarnings = this.securityMetrics.securityWarnings.slice(-10);
    }
  }
  
  /**
   * Clear refresh timer
   */
  private clearRefreshTimer(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = undefined;
    }
  }
  
  /**
   * Get security metrics for monitoring
   */
  public getSecurityMetrics(): SecurityMetrics {
    return { ...this.securityMetrics };
  }
  
  /**
   * Manual refresh trigger (for testing or special cases)
   */
  public async manualRefresh(): Promise<boolean> {
    console.log('🔄 Manual token refresh triggered');
    return this.performRefresh();
  }
  
  /**
   * Check if refresh is currently in progress
   */
  public isRefreshInProgress(): boolean {
    return this.isRefreshing;
  }
  
  /**
   * Cleanup resources and timers
   */
  public cleanup(): void {
    this.clearRefreshTimer();
    
    if (typeof window !== 'undefined') {
      window.removeEventListener('storage', this.handleStorageChange);
      window.removeEventListener('beforeunload', this.cleanup);
      
      // Clear session data
      sessionStorage.removeItem('tokenRefreshScheduled');
      sessionStorage.removeItem('deviceFingerprint');
    }
  }
}

// Global instance for easy access
export const secureTokenRefreshManager = new SecureTokenRefreshManager();

// Auto-initialize if we have a token
if (typeof window !== 'undefined') {
  // Check for existing access token and schedule refresh
  const checkAndScheduleRefresh = () => {
    // This would typically get the token from your auth context/state
    // For now, we'll listen for the token to be set
    window.addEventListener('tokenSet', (event: any) => {
      if (event.detail?.access_token) {
        secureTokenRefreshManager.scheduleRefresh(event.detail.access_token);
      }
    });
  };
  
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', checkAndScheduleRefresh);
  } else {
    checkAndScheduleRefresh();
  }
}