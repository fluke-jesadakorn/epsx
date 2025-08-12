// Cross-app authentication synchronization
// Handles real-time auth state sync between frontend and admin-frontend

export interface CrossAppAuthMessage {
  type: 'auth-login' | 'auth-logout' | 'auth-refresh' | 'auth-ping';
  data?: {
    token?: string;
    userId?: string;
    expiresAt?: number;
    timestamp: number;
  };
  source: 'frontend' | 'admin-frontend';
  targetOrigin: string;
}

export interface CrossAppSyncConfig {
  frontendOrigin: string;
  adminOrigin: string;
  pingInterval: number;
  maxRetries: number;
}

/**
 * Cross-app authentication synchronizer
 */
export class CrossAppAuthSync {
  private config: CrossAppSyncConfig;
  private pingInterval?: NodeJS.Timeout;
  private messageHandlers: Map<string, (data: any) => void> = new Map();
  private currentApp: 'frontend' | 'admin-frontend';

  constructor(config: CrossAppSyncConfig) {
    this.config = config;
    this.currentApp = window.location.port === '3001' ? 'admin-frontend' : 'frontend';
    this.setupMessageListeners();
  }

  /**
   * Initialize cross-app sync
   */
  initialize(): void {
    this.startPing();
    this.sendMessage('auth-ping', {
      timestamp: Date.now()
    });
  }

  /**
   * Send authentication login event to other app
   */
  notifyLogin(token: string, userId: string, expiresAt: number): void {
    this.sendMessage('auth-login', {
      token,
      userId,
      expiresAt,
      timestamp: Date.now()
    });
  }

  /**
   * Send authentication logout event to other app
   */
  notifyLogout(): void {
    this.sendMessage('auth-logout', {
      timestamp: Date.now()
    });
  }

  /**
   * Send token refresh event to other app
   */
  notifyTokenRefresh(token: string, expiresAt: number): void {
    this.sendMessage('auth-refresh', {
      token,
      expiresAt,
      timestamp: Date.now()
    });
  }

  /**
   * Register message handler
   */
  onMessage(type: string, handler: (data: any) => void): void {
    this.messageHandlers.set(type, handler);
  }

  /**
   * Send message to other app
   */
  private sendMessage(type: CrossAppAuthMessage['type'], data?: any): void {
    const message: CrossAppAuthMessage = {
      type,
      data,
      source: this.currentApp,
      targetOrigin: this.getTargetOrigin()
    };

    // Try iframe communication first (if available)
    this.sendViaIframe(message);

    // Also use localStorage as backup
    this.sendViaLocalStorage(message);

    // Use BroadcastChannel API if available
    this.sendViaBroadcastChannel(message);
  }

  /**
   * Get target origin for the other app
   */
  private getTargetOrigin(): string {
    return this.currentApp === 'frontend' 
      ? this.config.adminOrigin 
      : this.config.frontendOrigin;
  }

  /**
   * Send message via iframe (for same-domain communication)
   */
  private sendViaIframe(message: CrossAppAuthMessage): void {
    try {
      // Look for existing iframe to other app
      const iframe = document.querySelector(`iframe[data-auth-sync="${this.getTargetApp()}"]`) as HTMLIFrameElement;
      
      if (iframe && iframe.contentWindow) {
        iframe.contentWindow.postMessage(message, message.targetOrigin);
      } else {
        // Create hidden iframe if not exists (only in development)
        if (process.env.NODE_ENV === 'development') {
          this.createSyncIframe();
        }
      }
    } catch (error) {
      console.debug('Iframe sync failed:', error);
    }
  }

  /**
   * Send message via localStorage (cross-tab communication)
   */
  private sendViaLocalStorage(message: CrossAppAuthMessage): void {
    try {
      const key = `cross_app_auth_${Date.now()}_${Math.random()}`;
      localStorage.setItem(key, JSON.stringify(message));
      
      // Clean up after short delay
      setTimeout(() => {
        try {
          localStorage.removeItem(key);
        } catch {}
      }, 1000);
    } catch (error) {
      console.debug('LocalStorage sync failed:', error);
    }
  }

  /**
   * Send message via BroadcastChannel API
   */
  private sendViaBroadcastChannel(message: CrossAppAuthMessage): void {
    try {
      if (typeof BroadcastChannel !== 'undefined') {
        const channel = new BroadcastChannel('epsx-auth-sync');
        channel.postMessage(message);
        channel.close();
      }
    } catch (error) {
      console.debug('BroadcastChannel sync failed:', error);
    }
  }

  /**
   * Setup message listeners for all communication methods
   */
  private setupMessageListeners(): void {
    // PostMessage listener (for iframe communication)
    window.addEventListener('message', (event) => {
      try {
        if (!this.isValidOrigin(event.origin)) return;
        
        const message = event.data as CrossAppAuthMessage;
        if (message && message.source !== this.currentApp) {
          this.handleMessage(message);
        }
      } catch (error) {
        console.debug('PostMessage handling failed:', error);
      }
    });

    // LocalStorage listener (for cross-tab communication)
    window.addEventListener('storage', (event) => {
      try {
        if (event.key?.startsWith('cross_app_auth_') && event.newValue) {
          const message = JSON.parse(event.newValue) as CrossAppAuthMessage;
          if (message.source !== this.currentApp) {
            this.handleMessage(message);
          }
        }
      } catch (error) {
        console.debug('LocalStorage message handling failed:', error);
      }
    });

    // BroadcastChannel listener
    try {
      if (typeof BroadcastChannel !== 'undefined') {
        const channel = new BroadcastChannel('epsx-auth-sync');
        channel.addEventListener('message', (event) => {
          try {
            const message = event.data as CrossAppAuthMessage;
            if (message && message.source !== this.currentApp) {
              this.handleMessage(message);
            }
          } catch (error) {
            console.debug('BroadcastChannel message handling failed:', error);
          }
        });
      }
    } catch (error) {
      console.debug('BroadcastChannel setup failed:', error);
    }
  }

  /**
   * Handle incoming message
   */
  private handleMessage(message: CrossAppAuthMessage): void {
    const handler = this.messageHandlers.get(message.type);
    if (handler) {
      handler(message.data);
    }

    // Also emit a general auth-sync event
    window.dispatchEvent(new CustomEvent('cross-app-auth-sync', {
      detail: message
    }));
  }

  /**
   * Check if origin is valid for communication
   */
  private isValidOrigin(origin: string): boolean {
    return origin === this.config.frontendOrigin || origin === this.config.adminOrigin;
  }

  /**
   * Get the other app identifier
   */
  private getTargetApp(): string {
    return this.currentApp === 'frontend' ? 'admin-frontend' : 'frontend';
  }

  /**
   * Create hidden iframe for cross-app communication (development only)
   */
  private createSyncIframe(): void {
    if (process.env.NODE_ENV !== 'development') return;
    
    try {
      const iframe = document.createElement('iframe');
      iframe.src = `${this.getTargetOrigin()}/auth-sync-endpoint`;
      iframe.style.display = 'none';
      iframe.style.position = 'absolute';
      iframe.style.left = '-9999px';
      iframe.setAttribute('data-auth-sync', this.getTargetApp());
      
      document.body.appendChild(iframe);
      
      // Clean up after timeout
      setTimeout(() => {
        try {
          if (iframe.parentNode) {
            iframe.parentNode.removeChild(iframe);
          }
        } catch {}
      }, 30000);
    } catch (error) {
      console.debug('Iframe creation failed:', error);
    }
  }

  /**
   * Start periodic ping to check connection
   */
  private startPing(): void {
    this.pingInterval = setInterval(() => {
      this.sendMessage('auth-ping', {
        timestamp: Date.now()
      });
    }, this.config.pingInterval);
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
    }
    this.messageHandlers.clear();
  }
}

/**
 * Create cross-app sync instance based on environment
 */
export function createCrossAppSync(): CrossAppAuthSync {
  const isDevelopment = process.env.NODE_ENV === 'development';
  
  const config: CrossAppSyncConfig = {
    frontendOrigin: isDevelopment 
      ? 'http://localhost:3000'
      : (process.env.NEXT_PUBLIC_FRONTEND_URL || 'https://app.epsx.com'),
    adminOrigin: isDevelopment
      ? 'http://localhost:3001' 
      : (process.env.NEXT_PUBLIC_ADMIN_URL || 'https://admin.epsx.com'),
    pingInterval: 30000, // 30 seconds
    maxRetries: 3
  };

  return new CrossAppAuthSync(config);
}

/**
 * Singleton instance
 */
let crossAppSyncInstance: CrossAppAuthSync | null = null;

export function getCrossAppSync(): CrossAppAuthSync {
  if (!crossAppSyncInstance) {
    crossAppSyncInstance = createCrossAppSync();
  }
  return crossAppSyncInstance;
}