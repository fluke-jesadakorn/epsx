// Advanced OIDC Client Wrapper
// Enterprise-grade client with circuit breaker, retry logic, health monitoring, adaptive flows

import { UserManager, WebStorageStateStore, User, UserManagerSettings } from 'oidc-client-ts';
import { getOIDCDiscoveryClient, type OIDCConfiguration, type TenantInfo } from './oidc-discovery-client';
import { getTenantDetectionService, type TenantDetectionResult } from './tenant-detection-service';

export interface OIDCClientOptions {
  client_id?: string;
  redirect_uri?: string;
  post_logout_redirect_uri?: string;
  scope?: string;
  response_type?: string;
  tenant_id?: string;
  automaticSilentRenew?: boolean;
  includeIdTokenInSilentRenew?: boolean;
  monitorSession?: boolean;
  checkSessionInterval?: number;
  revokeTokensOnSignout?: boolean;
  loadUserInfo?: boolean;
}

export interface AuthenticationState {
  isLoading: boolean;
  isAuthenticated: boolean;
  user: User | null;
  error: string | null;
  tenant: TenantInfo | null;
  healthStatus: 'healthy' | 'degraded' | 'unhealthy';
  lastActivity: number;
  sessionTimeoutWarning: boolean;
}

export interface CircuitBreakerState {
  state: 'closed' | 'open' | 'half-open';
  failureCount: number;
  lastFailureTime: number;
  nextAttemptTime: number;
}

export interface RetryConfig {
  maxAttempts: number;
  baseDelay: number;
  maxDelay: number;
  backoffMultiplier: number;
  retryableErrors: string[];
}

export interface HealthMetrics {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  lastHealthCheck: number;
  uptime: number;
}

export interface AdaptiveAuthConfig {
  riskBasedAuth: boolean;
  deviceFingerprinting: boolean;
  behaviorAnalysis: boolean;
  locationTracking: boolean;
  stepUpAuthThreshold: number;
}

export type AuthenticationMethod = 'redirect' | 'popup' | 'silent' | 'iframe' | 'device_flow';
export type AuthenticationRisk = 'low' | 'medium' | 'high' | 'critical';

/**
 * Advanced OIDC Client Wrapper with Enterprise Features
 */
export class OIDCClientWrapper {
  private userManager: UserManager | null = null;
  private discoveryClient = getOIDCDiscoveryClient();
  private tenantService = getTenantDetectionService();
  private config: OIDCConfiguration | null = null;
  private tenant: TenantInfo | null = null;
  
  // State management
  private state: AuthenticationState = {
    isLoading: false,
    isAuthenticated: false,
    user: null,
    error: null,
    tenant: null,
    healthStatus: 'healthy',
    lastActivity: Date.now(),
    sessionTimeoutWarning: false
  };

  // Circuit breaker
  private circuitBreaker: CircuitBreakerState = {
    state: 'closed',
    failureCount: 0,
    lastFailureTime: 0,
    nextAttemptTime: 0
  };

  // Retry configuration
  private retryConfig: RetryConfig = {
    maxAttempts: 3,
    baseDelay: 1000,
    maxDelay: 30000,
    backoffMultiplier: 2,
    retryableErrors: ['network_error', 'timeout', 'server_error', 'rate_limit']
  };

  // Health metrics
  private healthMetrics: HealthMetrics = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    averageResponseTime: 0,
    lastHealthCheck: Date.now(),
    uptime: Date.now()
  };

  // Adaptive auth configuration
  private adaptiveConfig: AdaptiveAuthConfig = {
    riskBasedAuth: true,
    deviceFingerprinting: true,
    behaviorAnalysis: true,
    locationTracking: false, // Privacy consideration
    stepUpAuthThreshold: 0.7
  };

  // Event listeners
  private listeners = new Map<string, Function[]>();
  private sessionTimer: NodeJS.Timeout | null = null;
  private healthMonitor: NodeJS.Timeout | null = null;

  constructor(private options: OIDCClientOptions = {}) {
    this.startHealthMonitoring();
    this.startSessionMonitoring();
    console.log('🚀 Advanced OIDC Client Wrapper initialized');
  }

  /**
   * Initialize OIDC client with tenant detection
   */
  async initialize(userHint?: string): Promise<void> {
    try {
      this.setState({ isLoading: true, error: null });
      
      // Detect tenant if needed
      if (!this.options.tenant_id && userHint) {
        console.log('🔍 Detecting tenant for user hint:', userHint);
        const detection = await this.tenantService.detectTenant(userHint);
        if (detection.tenant) {
          this.tenant = detection.tenant;
          this.options.tenant_id = detection.tenant.tenant_id;
          console.log('✅ Tenant detected:', detection.tenant.tenant_id);
        }
      }

      // Discover OIDC configuration
      this.config = await this.executeWithCircuitBreaker(
        () => this.discoveryClient.discoverConfiguration(this.options.tenant_id)
      );

      // Create user manager
      const userManagerConfig = await this.buildUserManagerConfig();
      this.userManager = new UserManager(userManagerConfig);
      
      // Set up event handlers
      this.setupEventHandlers();
      
      // Check existing authentication
      const existingUser = await this.userManager.getUser();
      if (existingUser && !existingUser.expired) {
        this.setState({ 
          isAuthenticated: true, 
          user: existingUser,
          healthStatus: 'healthy'
        });
        console.log('✅ Existing valid session found');
      }

      this.setState({ isLoading: false });
      this.recordSuccessfulRequest(100); // Initialization time
      this.emit('initialized', { tenant: this.tenant });
      
    } catch (error) {
      console.error('❌ OIDC client initialization failed:', error);
      this.recordFailedRequest();
      this.setState({ 
        isLoading: false, 
        error: error instanceof Error ? error.message : 'Initialization failed',
        healthStatus: 'unhealthy'
      });
      throw error;
    }
  }

  /**
   * Adaptive authentication with risk assessment
   */
  async authenticateWithRiskAssessment(
    userInput: string,
    preferredMethod: AuthenticationMethod = 'redirect'
  ): Promise<User> {
    const startTime = Date.now();
    
    try {
      this.setState({ isLoading: true, error: null });
      
      // Assess authentication risk
      const riskLevel = await this.assessAuthenticationRisk(userInput);
      console.log('🎯 Authentication risk assessed:', riskLevel);
      
      // Adapt authentication method based on risk
      const authMethod = this.selectAuthenticationMethod(riskLevel, preferredMethod);
      
      // Select tenant detection strategy
      const tenantStrategy = this.selectTenantStrategy(riskLevel);
      const detection = await this.executeTenantDetection(userInput, tenantStrategy);
      
      if (detection.tenant && detection.tenant.tenant_id !== this.options.tenant_id) {
        console.log('🔄 Switching tenant:', detection.tenant.tenant_id);
        await this.switchTenant(detection.tenant);
      }

      // Execute authentication with selected method
      const user = await this.executeAuthentication(authMethod, { 
        userHint: userInput,
        riskLevel 
      });
      
      this.recordSuccessfulRequest(Date.now() - startTime);
      this.updateLastActivity();
      
      return user;
      
    } catch (error) {
      this.recordFailedRequest();
      console.error('❌ Adaptive authentication failed:', error);
      throw error;
    } finally {
      this.setState({ isLoading: false });
    }
  }

  /**
   * Standard redirect authentication
   */
  async signInRedirect(userHint?: string): Promise<void> {
    return this.executeWithRetry(async () => {
      if (!this.userManager) {
        await this.initialize(userHint);
      }
      
      const extraParams: any = {};
      if (userHint) {
        extraParams.login_hint = userHint;
      }
      
      console.log('🚀 Initiating redirect authentication');
      await this.userManager!.signinRedirect({ extraQueryParams: extraParams });
    });
  }

  /**
   * Popup authentication
   */
  async signInPopup(userHint?: string): Promise<User> {
    return this.executeWithRetry(async () => {
      if (!this.userManager) {
        await this.initialize(userHint);
      }
      
      const extraParams: any = {};
      if (userHint) {
        extraParams.login_hint = userHint;
      }
      
      console.log('🚀 Initiating popup authentication');
      const user = await this.userManager!.signinPopup({ extraQueryParams: extraParams });
      
      this.setState({ isAuthenticated: true, user });
      this.updateLastActivity();
      
      return user;
    });
  }

  /**
   * Silent authentication
   */
  async signInSilent(): Promise<User | null> {
    return this.executeWithRetry(async () => {
      if (!this.userManager) {
        throw new Error('OIDC client not initialized');
      }
      
      try {
        console.log('🔇 Attempting silent authentication');
        const user = await this.userManager.signinSilent();
        
        if (user) {
          this.setState({ isAuthenticated: true, user });
          this.updateLastActivity();
          console.log('✅ Silent authentication successful');
          return user;
        }
        
        return null;
        
      } catch (error) {
        console.log('🔇 Silent authentication failed:', error);
        return null;
      }
    });
  }

  /**
   * Handle redirect callback
   */
  async signInRedirectCallback(url?: string): Promise<User> {
    return this.executeWithRetry(async () => {
      if (!this.userManager) {
        throw new Error('OIDC client not initialized');
      }
      
      console.log('🔄 Processing redirect callback');
      const user = await this.userManager.signinRedirectCallback(url);
      
      this.setState({ isAuthenticated: true, user });
      this.updateLastActivity();
      this.emit('signin_callback_success', { user });
      
      return user;
    });
  }

  /**
   * Handle popup callback
   */
  async signInPopupCallback(url?: string): Promise<void> {
    return this.executeWithRetry(async () => {
      if (!this.userManager) {
        throw new Error('OIDC client not initialized');
      }
      
      console.log('🔄 Processing popup callback');
      await this.userManager.signinPopupCallback(url);
      this.emit('signin_popup_callback_success', {});
    });
  }

  /**
   * Sign out with cleanup
   */
  async signOut(): Promise<void> {
    return this.executeWithRetry(async () => {
      if (!this.userManager) {
        return;
      }
      
      console.log('🔒 Initiating sign out');
      
      // Clear state
      this.setState({ 
        isAuthenticated: false, 
        user: null, 
        sessionTimeoutWarning: false 
      });
      
      // Sign out from OIDC provider
      await this.userManager.signoutRedirect();
      
      this.emit('signout_success', {});
    });
  }

  /**
   * Get current user with validation
   */
  async getUser(): Promise<User | null> {
    if (!this.userManager) {
      return null;
    }
    
    try {
      const user = await this.userManager.getUser();
      
      if (user && !user.expired) {
        // Validate token freshness
        const tokenAge = Date.now() - (user.profile.iat * 1000);
        const maxAge = 24 * 60 * 60 * 1000; // 24 hours
        
        if (tokenAge > maxAge) {
          console.log('🔄 Token too old, attempting silent refresh');
          return await this.signInSilent();
        }
        
        this.setState({ isAuthenticated: true, user });
        return user;
      }
      
      // Try silent renewal
      return await this.signInSilent();
      
    } catch (error) {
      console.error('❌ Failed to get user:', error);
      return null;
    }
  }

  /**
   * Switch to different tenant
   */
  async switchTenant(newTenant: TenantInfo): Promise<void> {
    console.log('🔄 Switching tenant from', this.tenant?.tenant_id, 'to', newTenant.tenant_id);
    
    // Clear current session
    if (this.state.isAuthenticated) {
      await this.signOut();
    }
    
    // Update tenant configuration
    this.tenant = newTenant;
    this.options.tenant_id = newTenant.tenant_id;
    
    // Re-initialize with new tenant
    await this.initialize();
    
    this.emit('tenant_switched', { oldTenant: this.tenant, newTenant });
  }

  /**
   * Get authentication state
   */
  getState(): Readonly<AuthenticationState> {
    return { ...this.state };
  }

  /**
   * Get health metrics
   */
  getHealthMetrics(): Readonly<HealthMetrics> {
    return { ...this.healthMetrics };
  }

  /**
   * Add event listener
   */
  on(event: string, listener: Function): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(listener);
  }

  /**
   * Remove event listener
   */
  off(event: string, listener: Function): void {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      const index = eventListeners.indexOf(listener);
      if (index > -1) {
        eventListeners.splice(index, 1);
      }
    }
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    if (this.sessionTimer) {
      clearInterval(this.sessionTimer);
    }
    if (this.healthMonitor) {
      clearInterval(this.healthMonitor);
    }
    
    this.listeners.clear();
    this.userManager = null;
    console.log('🧹 OIDC client wrapper destroyed');
  }

  // Private methods

  private async buildUserManagerConfig(): Promise<UserManagerSettings> {
    const baseUrl = process.env.NEXT_PUBLIC_FRONTEND_URL || window.location.origin;
    
    return {
      authority: this.config!.issuer,
      client_id: this.options.client_id || process.env.NEXT_PUBLIC_OIDC_CLIENT_ID || 'epsx-frontend',
      redirect_uri: this.options.redirect_uri || `${baseUrl}/auth/callback`,
      post_logout_redirect_uri: this.options.post_logout_redirect_uri || `${baseUrl}/`,
      response_type: this.options.response_type || 'code',
      scope: this.options.scope || 'openid profile email',
      
      // PKCE
      code_challenge_method: 'S256',
      
      // Storage
      userStore: new WebStorageStateStore({ store: window.localStorage }),
      stateStore: new WebStorageStateStore({ store: window.sessionStorage }),
      
      // Token management
      automaticSilentRenew: this.options.automaticSilentRenew ?? true,
      includeIdTokenInSilentRenew: this.options.includeIdTokenInSilentRenew ?? true,
      silent_redirect_uri: `${baseUrl}/auth/silent-callback`,
      
      // Session monitoring
      monitorSession: this.options.monitorSession ?? true,
      checkSessionInterval: this.options.checkSessionInterval || 60000, // 1 minute
      
      // Security
      revokeTokensOnSignout: this.options.revokeTokensOnSignout ?? true,
      loadUserInfo: this.options.loadUserInfo ?? true,
      
      // Timeouts
      silentRequestTimeoutInSeconds: 30,
      
      // Metadata
      metadata: {
        issuer: this.config!.issuer,
        authorization_endpoint: this.config!.authorization_endpoint,
        token_endpoint: this.config!.token_endpoint,
        userinfo_endpoint: this.config!.userinfo_endpoint,
        jwks_uri: this.config!.jwks_uri,
        end_session_endpoint: this.config!.end_session_endpoint,
      }
    };
  }

  private setupEventHandlers(): void {
    if (!this.userManager) return;
    
    this.userManager.events.addUserLoaded((user) => {
      console.log('👤 User loaded:', user.profile.email);
      this.setState({ isAuthenticated: true, user });
      this.updateLastActivity();
      this.emit('user_loaded', { user });
    });
    
    this.userManager.events.addUserUnloaded(() => {
      console.log('👤 User unloaded');
      this.setState({ isAuthenticated: false, user: null });
      this.emit('user_unloaded', {});
    });
    
    this.userManager.events.addSilentRenewError((error) => {
      console.error('🔇 Silent renew error:', error);
      this.setState({ error: error.message });
      this.emit('silent_renew_error', { error });
    });
    
    this.userManager.events.addUserSignedIn(() => {
      console.log('✅ User signed in');
      this.updateLastActivity();
      this.emit('user_signed_in', {});
    });
    
    this.userManager.events.addUserSignedOut(() => {
      console.log('🔒 User signed out');
      this.setState({ isAuthenticated: false, user: null });
      this.emit('user_signed_out', {});
    });
    
    this.userManager.events.addUserSessionChanged(() => {
      console.log('🔄 User session changed');
      this.emit('session_changed', {});
    });
  }

  private async assessAuthenticationRisk(userInput: string): Promise<AuthenticationRisk> {
    if (!this.adaptiveConfig.riskBasedAuth) {
      return 'low';
    }
    
    let riskScore = 0;
    
    // Check for known risk patterns
    if (userInput.includes('admin') || userInput.includes('root')) {
      riskScore += 0.3;
    }
    
    // Device fingerprinting
    if (this.adaptiveConfig.deviceFingerprinting) {
      const isNewDevice = !localStorage.getItem('device_fingerprint');
      if (isNewDevice) {
        riskScore += 0.2;
        // Store device fingerprint
        localStorage.setItem('device_fingerprint', this.generateDeviceFingerprint());
      }
    }
    
    // Behavioral analysis
    if (this.adaptiveConfig.behaviorAnalysis) {
      const lastActivity = localStorage.getItem('last_activity');
      const currentTime = Date.now();
      
      if (lastActivity) {
        const timeSinceLastActivity = currentTime - parseInt(lastActivity);
        if (timeSinceLastActivity > 7 * 24 * 60 * 60 * 1000) { // 7 days
          riskScore += 0.2;
        }
      }
    }
    
    // Time-based risk
    const hour = new Date().getHours();
    if (hour < 6 || hour > 22) {
      riskScore += 0.1; // Off-hours access
    }
    
    // Convert score to risk level
    if (riskScore >= 0.8) return 'critical';
    if (riskScore >= 0.6) return 'high';
    if (riskScore >= 0.3) return 'medium';
    return 'low';
  }

  private selectAuthenticationMethod(
    riskLevel: AuthenticationRisk, 
    preferred: AuthenticationMethod
  ): AuthenticationMethod {
    // High-risk scenarios force more secure methods
    if (riskLevel === 'critical' || riskLevel === 'high') {
      return 'redirect'; // Most secure with full page context
    }
    
    return preferred;
  }

  private selectTenantStrategy(riskLevel: AuthenticationRisk): 'aggressive' | 'conservative' | 'balanced' {
    switch (riskLevel) {
      case 'critical':
      case 'high':
        return 'aggressive'; // Multiple detection methods
      case 'medium':
        return 'balanced'; // Standard detection
      default:
        return 'conservative'; // Basic detection
    }
  }

  private async executeTenantDetection(
    userInput: string, 
    strategy: 'aggressive' | 'conservative' | 'balanced'
  ): Promise<TenantDetectionResult> {
    switch (strategy) {
      case 'aggressive':
        // Multiple parallel detection methods
        const results = await Promise.allSettled([
          this.tenantService.detectTenant(userInput),
          this.tenantService.detectTenant(userInput.split('@')[1] || userInput),
        ]);
        
        const validResults = results
          .filter((r): r is PromiseFulfilledResult<TenantDetectionResult> => r.status === 'fulfilled')
          .map(r => r.value)
          .filter(r => r.tenant !== null);
        
        return validResults.length > 0 ? validResults[0] : { tenant: null, confidence: 'none', method: 'default' };
        
      case 'balanced':
        return this.tenantService.detectTenant(userInput);
        
      case 'conservative':
      default:
        // Only domain-based detection
        if (userInput.includes('@')) {
          const domain = userInput.split('@')[1];
          return this.tenantService.detectTenant(domain);
        }
        return { tenant: null, confidence: 'none', method: 'default' };
    }
  }

  private async executeAuthentication(
    method: AuthenticationMethod, 
    context: { userHint?: string; riskLevel: AuthenticationRisk }
  ): Promise<User> {
    switch (method) {
      case 'popup':
        return this.signInPopup(context.userHint);
      case 'silent':
        const silentUser = await this.signInSilent();
        if (!silentUser) {
          throw new Error('Silent authentication failed, user interaction required');
        }
        return silentUser;
      case 'redirect':
      default:
        await this.signInRedirect(context.userHint);
        throw new Error('Redirect initiated'); // This shouldn't return normally
    }
  }

  private async executeWithCircuitBreaker<T>(operation: () => Promise<T>): Promise<T> {
    // Check circuit breaker state
    if (this.circuitBreaker.state === 'open') {
      if (Date.now() < this.circuitBreaker.nextAttemptTime) {
        throw new Error('Circuit breaker is open - service temporarily unavailable');
      } else {
        this.circuitBreaker.state = 'half-open';
      }
    }
    
    try {
      const result = await operation();
      
      // Success - reset circuit breaker
      if (this.circuitBreaker.state === 'half-open' || this.circuitBreaker.failureCount > 0) {
        this.circuitBreaker.state = 'closed';
        this.circuitBreaker.failureCount = 0;
        console.log('🔄 Circuit breaker reset to closed state');
      }
      
      return result;
      
    } catch (error) {
      this.circuitBreaker.failureCount++;
      this.circuitBreaker.lastFailureTime = Date.now();
      
      // Trip circuit breaker after 3 failures
      if (this.circuitBreaker.failureCount >= 3) {
        this.circuitBreaker.state = 'open';
        this.circuitBreaker.nextAttemptTime = Date.now() + 30000; // 30 seconds
        console.log('⚡ Circuit breaker tripped to open state');
      }
      
      throw error;
    }
  }

  private async executeWithRetry<T>(operation: () => Promise<T>): Promise<T> {
    let lastError: Error | null = null;
    
    for (let attempt = 1; attempt <= this.retryConfig.maxAttempts; attempt++) {
      try {
        return await this.executeWithCircuitBreaker(operation);
        
      } catch (error) {
        lastError = error as Error;
        
        // Check if error is retryable
        const isRetryable = this.isRetryableError(lastError);
        if (!isRetryable || attempt === this.retryConfig.maxAttempts) {
          throw lastError;
        }
        
        // Calculate backoff delay
        const delay = Math.min(
          this.retryConfig.baseDelay * Math.pow(this.retryConfig.backoffMultiplier, attempt - 1),
          this.retryConfig.maxDelay
        );
        
        console.log(`🔄 Retrying operation in ${delay}ms (attempt ${attempt}/${this.retryConfig.maxAttempts})`);
        await this.sleep(delay);
      }
    }
    
    throw lastError;
  }

  private isRetryableError(error: Error): boolean {
    const errorMessage = error.message.toLowerCase();
    return this.retryConfig.retryableErrors.some(retryableError => 
      errorMessage.includes(retryableError)
    );
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private setState(updates: Partial<AuthenticationState>): void {
    this.state = { ...this.state, ...updates };
    this.emit('state_changed', { state: this.state });
  }

  private emit(event: string, data: any): void {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.forEach(listener => {
        try {
          listener(data);
        } catch (error) {
          console.error('❌ Event listener error:', error);
        }
      });
    }
  }

  private recordSuccessfulRequest(responseTime: number): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.successfulRequests++;
    this.updateAverageResponseTime(responseTime);
    
    // Update health status
    const successRate = this.healthMetrics.successfulRequests / this.healthMetrics.totalRequests;
    if (successRate > 0.9) {
      this.setState({ healthStatus: 'healthy' });
    } else if (successRate > 0.7) {
      this.setState({ healthStatus: 'degraded' });
    }
  }

  private recordFailedRequest(): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.failedRequests++;
    
    // Update health status
    const successRate = this.healthMetrics.successfulRequests / this.healthMetrics.totalRequests;
    if (successRate < 0.5) {
      this.setState({ healthStatus: 'unhealthy' });
    } else if (successRate < 0.9) {
      this.setState({ healthStatus: 'degraded' });
    }
  }

  private updateAverageResponseTime(responseTime: number): void {
    const currentAvg = this.healthMetrics.averageResponseTime;
    const totalRequests = this.healthMetrics.totalRequests;
    
    this.healthMetrics.averageResponseTime = 
      ((currentAvg * (totalRequests - 1)) + responseTime) / totalRequests;
  }

  private updateLastActivity(): void {
    this.state.lastActivity = Date.now();
    localStorage.setItem('last_activity', String(Date.now()));
  }

  private startHealthMonitoring(): void {
    this.healthMonitor = setInterval(() => {
      this.healthMetrics.lastHealthCheck = Date.now();
      this.emit('health_check', { metrics: this.healthMetrics });
    }, 60000); // Every minute
  }

  private startSessionMonitoring(): void {
    this.sessionTimer = setInterval(() => {
      if (this.state.isAuthenticated) {
        const inactiveTime = Date.now() - this.state.lastActivity;
        const warningThreshold = 25 * 60 * 1000; // 25 minutes
        const timeoutThreshold = 30 * 60 * 1000; // 30 minutes
        
        if (inactiveTime > timeoutThreshold) {
          console.log('🔒 Session timeout due to inactivity');
          this.signOut();
        } else if (inactiveTime > warningThreshold && !this.state.sessionTimeoutWarning) {
          this.setState({ sessionTimeoutWarning: true });
          this.emit('session_timeout_warning', { remainingTime: timeoutThreshold - inactiveTime });
        }
      }
    }, 60000); // Check every minute
  }

  private generateDeviceFingerprint(): string {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    ctx.textBaseline = 'top';
    ctx.font = '14px Arial';
    ctx.fillText('Device fingerprint', 2, 2);
    
    const fingerprint = [
      navigator.userAgent,
      navigator.language,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset(),
      canvas.toDataURL()
    ].join('|');
    
    return btoa(fingerprint).replace(/[^a-zA-Z0-9]/g, '').substring(0, 32);
  }
}

// Singleton instance
let oidcClient: OIDCClientWrapper | null = null;

export function getOIDCClient(options?: OIDCClientOptions): OIDCClientWrapper {
  if (!oidcClient) {
    oidcClient = new OIDCClientWrapper(options);
  }
  return oidcClient;
}

export function resetOIDCClient(): void {
  if (oidcClient) {
    oidcClient.destroy();
    oidcClient = null;
  }
}