'use client';

export interface FCMDiagnosticResult {
  component: string;
  status: 'pass' | 'fail' | 'warning' | 'info';
  message: string;
  details?: any;
  recommendation?: string;
}

export interface FCMDiagnosticReport {
  timestamp: Date;
  overall: 'healthy' | 'degraded' | 'unhealthy';
  score: number; // 0-100
  results: FCMDiagnosticResult[];
  environment: {
    userAgent: string;
    platform: string;
    url: string;
    timestamp: string;
  };
}

export class FCMDiagnostics {
  private diagnosticResults: FCMDiagnosticResult[] = [];

  /**
   * Run comprehensive FCM diagnostics
   */
  async runDiagnostics(): Promise<FCMDiagnosticReport> {
    this.diagnosticResults = [];

    // Run all diagnostic checks
    await Promise.all([
      this.checkBrowserSupport(),
      this.checkServiceWorkerSupport(),
      this.checkNotificationPermission(),
      this.checkFirebaseConfiguration(),
      this.checkVAPIDKey(),
      this.checkServiceWorkerRegistration(),
      this.checkNetworkConnectivity(),
      this.checkStorageAvailability(),
      this.checkSecurityContext(),
      this.checkAPIEndpoints()
    ]);

    // Calculate overall health score
    const score = this.calculateHealthScore();
    const overall = this.determineOverallHealth(score);

    return {
      timestamp: new Date(),
      overall,
      score,
      results: [...this.diagnosticResults],
      environment: {
        userAgent: navigator.userAgent,
        platform: navigator.platform,
        url: window.location.href,
        timestamp: new Date().toISOString()
      }
    };
  }

  /**
   * Check browser support for FCM
   */
  private async checkBrowserSupport(): Promise<void> {
    const isSupported = (
      typeof window !== 'undefined' &&
      'serviceWorker' in navigator &&
      'PushManager' in window &&
      'Notification' in window
    );

    if (isSupported) {
      this.addResult({
        component: 'Browser Support',
        status: 'pass',
        message: 'Browser fully supports FCM notifications'
      });
    } else {
      const missing = [];
      if (!('serviceWorker' in navigator)) missing.push('Service Worker');
      if (!('PushManager' in window)) missing.push('Push Manager');
      if (!('Notification' in window)) missing.push('Notifications');

      this.addResult({
        component: 'Browser Support',
        status: 'fail',
        message: `Browser missing FCM support: ${missing.join(', ')}`,
        recommendation: 'Please use a modern browser like Chrome, Firefox, or Safari'
      });
    }
  }

  /**
   * Check service worker support
   */
  private async checkServiceWorkerSupport(): Promise<void> {
    if ('serviceWorker' in navigator) {
      try {
        const registration = await navigator.serviceWorker.getRegistration();
        if (registration) {
          this.addResult({
            component: 'Service Worker',
            status: 'pass',
            message: 'Service worker is registered and active',
            details: {
              scope: registration.scope,
              state: registration.active?.state
            }
          });
        } else {
          this.addResult({
            component: 'Service Worker',
            status: 'warning',
            message: 'No service worker registration found',
            recommendation: 'Service worker will be registered when FCM is initialized'
          });
        }
      } catch (error) {
        this.addResult({
          component: 'Service Worker',
          status: 'fail',
          message: 'Service worker check failed',
          details: error,
          recommendation: 'Check browser console for service worker errors'
        });
      }
    } else {
      this.addResult({
        component: 'Service Worker',
        status: 'fail',
        message: 'Service workers not supported',
        recommendation: 'Use a browser that supports service workers'
      });
    }
  }

  /**
   * Check notification permission status
   */
  private async checkNotificationPermission(): Promise<void> {
    if ('Notification' in window) {
      const permission = Notification.permission;
      
      switch (permission) {
        case 'granted':
          this.addResult({
            component: 'Notification Permission',
            status: 'pass',
            message: 'Notifications are allowed'
          });
          break;
        case 'denied':
          this.addResult({
            component: 'Notification Permission',
            status: 'fail',
            message: 'Notifications are blocked',
            recommendation: 'Click the lock icon in the address bar to enable notifications'
          });
          break;
        case 'default':
          this.addResult({
            component: 'Notification Permission',
            status: 'warning',
            message: 'Notification permission not requested yet',
            recommendation: 'Permission will be requested when subscribing to notifications'
          });
          break;
      }
    } else {
      this.addResult({
        component: 'Notification Permission',
        status: 'fail',
        message: 'Notifications API not available'
      });
    }
  }

  /**
   * Check Firebase configuration
   */
  private async checkFirebaseConfiguration(): Promise<void> {
    const requiredConfig = [
      'NEXT_PUBLIC_FIREBASE_API_KEY',
      'NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN',
      'NEXT_PUBLIC_FIREBASE_PROJECT_ID',
      'NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET',
      'NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID',
      'NEXT_PUBLIC_FIREBASE_APP_ID'
    ];

    const missingConfig = requiredConfig.filter(key => !process.env[key]);

    if (missingConfig.length === 0) {
      this.addResult({
        component: 'Firebase Configuration',
        status: 'pass',
        message: 'All required Firebase config variables are present'
      });
    } else {
      this.addResult({
        component: 'Firebase Configuration',
        status: 'fail',
        message: `Missing Firebase config: ${missingConfig.join(', ')}`,
        recommendation: 'Check your environment variables'
      });
    }
  }

  /**
   * Check VAPID key configuration
   */
  private async checkVAPIDKey(): Promise<void> {
    const vapidKey = process.env.NEXT_PUBLIC_FIREBASE_VAPID_KEY;
    
    if (vapidKey) {
      // Basic VAPID key format validation
      if (vapidKey.length === 88 && /^[A-Za-z0-9_-]+$/.test(vapidKey)) {
        this.addResult({
          component: 'VAPID Key',
          status: 'pass',
          message: 'VAPID key is properly configured'
        });
      } else {
        this.addResult({
          component: 'VAPID Key',
          status: 'warning',
          message: 'VAPID key format may be invalid',
          details: { length: vapidKey.length },
          recommendation: 'Verify VAPID key from Firebase console'
        });
      }
    } else {
      this.addResult({
        component: 'VAPID Key',
        status: 'fail',
        message: 'VAPID key not configured',
        recommendation: 'Set NEXT_PUBLIC_FIREBASE_VAPID_KEY environment variable'
      });
    }
  }

  /**
   * Check service worker registration
   */
  private async checkServiceWorkerRegistration(): Promise<void> {
    try {
      const response = await fetch('/firebase-messaging-sw.js', { method: 'HEAD' });
      
      if (response.ok) {
        this.addResult({
          component: 'Service Worker File',
          status: 'pass',
          message: 'FCM service worker file is accessible',
          details: {
            status: response.status,
            contentType: response.headers.get('content-type')
          }
        });
      } else {
        this.addResult({
          component: 'Service Worker File',
          status: 'fail',
          message: `Service worker file not accessible (${response.status})`,
          recommendation: 'Ensure firebase-messaging-sw.js exists in public folder'
        });
      }
    } catch (error) {
      this.addResult({
        component: 'Service Worker File',
        status: 'fail',
        message: 'Cannot access service worker file',
        details: error,
        recommendation: 'Check network connectivity and file path'
      });
    }
  }

  /**
   * Check network connectivity
   */
  private async checkNetworkConnectivity(): Promise<void> {
    try {
      // Test connection to Firebase
      const firebaseResponse = await fetch('https://fcm.googleapis.com/fcm/send', {
        method: 'HEAD',
        mode: 'no-cors'
      });

      // Test connection to backend API
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
      const backendResponse = await fetch(`${backendUrl}/health`, {
        method: 'HEAD'
      });

      this.addResult({
        component: 'Network Connectivity',
        status: 'pass',
        message: 'Network connectivity is working',
        details: {
          firebaseReachable: true,
          backendReachable: backendResponse.ok
        }
      });
    } catch (error) {
      this.addResult({
        component: 'Network Connectivity',
        status: 'warning',
        message: 'Network connectivity issues detected',
        details: error,
        recommendation: 'Check internet connection and firewall settings'
      });
    }
  }

  /**
   * Check storage availability
   */
  private async checkStorageAvailability(): Promise<void> {
    try {
      // Test localStorage
      const testKey = 'fcm_diagnostic_test';
      localStorage.setItem(testKey, 'test');
      localStorage.removeItem(testKey);

      // Test IndexedDB (used by Firebase)
      if ('indexedDB' in window) {
        this.addResult({
          component: 'Storage',
          status: 'pass',
          message: 'Local storage and IndexedDB are available'
        });
      } else {
        this.addResult({
          component: 'Storage',
          status: 'warning',
          message: 'IndexedDB not available',
          recommendation: 'Some FCM features may not work properly'
        });
      }
    } catch (error) {
      this.addResult({
        component: 'Storage',
        status: 'fail',
        message: 'Storage access denied',
        details: error,
        recommendation: 'Check browser privacy settings'
      });
    }
  }

  /**
   * Check security context (HTTPS)
   */
  private async checkSecurityContext(): Promise<void> {
    const isSecure = location.protocol === 'https:' || location.hostname === 'localhost';
    
    if (isSecure) {
      this.addResult({
        component: 'Security Context',
        status: 'pass',
        message: 'Secure context available (HTTPS or localhost)'
      });
    } else {
      this.addResult({
        component: 'Security Context',
        status: 'fail',
        message: 'Insecure context - FCM requires HTTPS',
        recommendation: 'Use HTTPS or localhost for FCM functionality'
      });
    }
  }

  /**
   * Check API endpoints availability
   */
  private async checkAPIEndpoints(): Promise<void> {
    const endpoints = [
      '/api/v1/notifications/fcm/subscribe',
      '/api/v1/notifications/fcm/unsubscribe',
      '/api/v1/notifications/fcm/test',
      '/api/v1/notifications/fcm/tokens/my'
    ];

    const results = await Promise.allSettled(
      endpoints.map(endpoint => 
        fetch(endpoint, { method: 'OPTIONS' })
      )
    );

    const availableEndpoints = results.filter(result => 
      result.status === 'fulfilled' && result.value.ok
    ).length;

    if (availableEndpoints === endpoints.length) {
      this.addResult({
        component: 'API Endpoints',
        status: 'pass',
        message: 'All FCM API endpoints are available'
      });
    } else {
      this.addResult({
        component: 'API Endpoints',
        status: 'warning',
        message: `${availableEndpoints}/${endpoints.length} API endpoints available`,
        recommendation: 'Check backend server status'
      });
    }
  }

  /**
   * Add diagnostic result
   */
  private addResult(result: FCMDiagnosticResult): void {
    this.diagnosticResults.push(result);
  }

  /**
   * Calculate health score based on diagnostic results
   */
  private calculateHealthScore(): number {
    if (this.diagnosticResults.length === 0) return 0;

    const scores = this.diagnosticResults.map(result => {
      switch (result.status) {
        case 'pass': return 100;
        case 'warning': return 70;
        case 'info': return 90;
        case 'fail': return 0;
        default: return 50;
      }
    });

    return Math.round(scores.reduce((sum, score) => sum + score, 0) / scores.length);
  }

  /**
   * Determine overall health status
   */
  private determineOverallHealth(score: number): 'healthy' | 'degraded' | 'unhealthy' {
    if (score >= 80) return 'healthy';
    if (score >= 50) return 'degraded';
    return 'unhealthy';
  }

  /**
   * Get diagnostic recommendations
   */
  getRecommendations(): string[] {
    return this.diagnosticResults
      .filter(result => result.recommendation)
      .map(result => result.recommendation!)
      .filter((recommendation, index, array) => array.indexOf(recommendation) === index);
  }

  /**
   * Get critical issues that prevent FCM from working
   */
  getCriticalIssues(): FCMDiagnosticResult[] {
    return this.diagnosticResults.filter(result => result.status === 'fail');
  }

  /**
   * Generate diagnostic summary
   */
  generateSummary(report: FCMDiagnosticReport): string {
    const { overall, score, results } = report;
    const critical = results.filter(r => r.status === 'fail').length;
    const warnings = results.filter(r => r.status === 'warning').length;

    return `FCM Health: ${overall.toUpperCase()} (Score: ${score}/100) - ${critical} critical issues, ${warnings} warnings`;
  }
}

// Export singleton instance
export const fcmDiagnostics = new FCMDiagnostics();

// Convenience function for quick health check
export const runFCMHealthCheck = () => fcmDiagnostics.runDiagnostics();

export default fcmDiagnostics;