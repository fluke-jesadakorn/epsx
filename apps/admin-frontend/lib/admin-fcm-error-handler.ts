'use client';

export interface AdminFCMError {
  code: string;
  message: string;
  originalError?: Error;
  context?: string;
  timestamp: Date;
  userAction?: string;
  adminContext: boolean;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

export interface AdminFCMErrorHandlerOptions {
  enableLogging?: boolean;
  enableRetry?: boolean;
  maxRetries?: number;
  retryDelay?: number;
  fallbackEnabled?: boolean;
  adminAlertsEnabled?: boolean;
  onError?: (error: AdminFCMError) => void;
  onRecovery?: (context: string) => void;
  onCriticalError?: (error: AdminFCMError) => void;
}

export class AdminFCMErrorHandler {
  private options: Required<AdminFCMErrorHandlerOptions>;
  private retryCount = new Map<string, number>();
  private errorLog: AdminFCMError[] = [];
  private criticalErrors: AdminFCMError[] = [];

  constructor(options: AdminFCMErrorHandlerOptions = {}) {
    this.options = {
      enableLogging: true,
      enableRetry: true,
      maxRetries: 3,
      retryDelay: 1000,
      fallbackEnabled: true,
      adminAlertsEnabled: true,
      onError: () => {},
      onRecovery: () => {},
      onCriticalError: () => {},
      ...options
    };
  }

  /**
   * Handle admin FCM errors with enhanced categorization and security considerations
   */
  handleError(error: Error | any, context: string, userAction?: string): AdminFCMError {
    const adminError = this.categorizeAdminError(error, context, userAction);
    
    // Log error if enabled
    if (this.options.enableLogging) {
      this.logAdminError(adminError);
      console.error(`Admin FCM Error [${context}]:`, adminError);
    }

    // Handle critical errors specially
    if (adminError.severity === 'critical') {
      this.handleCriticalError(adminError);
    }

    // Call custom error handlers
    this.options.onError(adminError);
    if (adminError.severity === 'critical') {
      this.options.onCriticalError(adminError);
    }

    // Determine if retry is appropriate
    if (this.shouldRetryAdminOperation(adminError)) {
      this.scheduleAdminRetry(adminError, context);
    }

    // Apply admin-specific fallback mechanisms
    this.applyAdminFallback(adminError);

    return adminError;
  }

  /**
   * Categorize admin FCM errors with security and permission considerations
   */
  private categorizeAdminError(error: Error | any, context: string, userAction?: string): AdminFCMError {
    let code = 'ADMIN_UNKNOWN_ERROR';
    let message = 'An unknown admin FCM error occurred';
    let severity: AdminFCMError['severity'] = 'medium';

    if (error instanceof Error) {
      // Admin permission errors (critical)
      if (error.message.includes('admin') && error.message.includes('permission')) {
        code = 'ADMIN_PERMISSION_DENIED';
        message = 'Admin permissions denied for FCM notifications. Contact system administrator.';
        severity = 'critical';
      } else if (error.message.includes('Insufficient permissions')) {
        code = 'ADMIN_INSUFFICIENT_PERMISSIONS';
        message = 'Insufficient admin permissions for notification management.';
        severity = 'high';
      }
      // Admin authentication errors (critical)
      else if (error.message.includes('Admin access required')) {
        code = 'ADMIN_AUTH_REQUIRED';
        message = 'Admin authentication required for FCM operations.';
        severity = 'critical';
      } else if (error.message.includes('Admin session expired')) {
        code = 'ADMIN_SESSION_EXPIRED';
        message = 'Admin session expired. Please log in again.';
        severity = 'high';
      }
      // Admin-specific FCM errors
      else if (error.message.includes('admin test notification')) {
        code = 'ADMIN_TEST_NOTIFICATION_FAILED';
        message = 'Failed to send admin test notification.';
        severity = 'low';
      } else if (error.message.includes('admin subscription')) {
        code = 'ADMIN_SUBSCRIPTION_FAILED';
        message = 'Failed to manage admin FCM subscription.';
        severity = 'medium';
      }
      // Security-related errors (critical)
      else if (error.message.includes('security') || error.message.includes('Security')) {
        code = 'ADMIN_SECURITY_ERROR';
        message = 'Security error in admin FCM operations.';
        severity = 'critical';
      }
      // Firebase/FCM specific errors
      else if (error.message.includes('messaging/permission-blocked')) {
        code = 'ADMIN_FCM_PERMISSION_BLOCKED';
        message = 'Admin FCM permissions blocked. Check browser settings.';
        severity = 'high';
      } else if (error.message.includes('messaging/vapid-key-required')) {
        code = 'ADMIN_VAPID_KEY_MISSING';
        message = 'VAPID key required for admin FCM notifications.';
        severity = 'critical';
      }
      // Network and service errors
      else if (error.message.includes('Failed to fetch')) {
        code = 'ADMIN_NETWORK_ERROR';
        message = 'Admin FCM network error. Check connection.';
        severity = 'medium';
      } else if (error.message.includes('500') || error.message.includes('Server error')) {
        code = 'ADMIN_SERVER_ERROR';
        message = 'Admin FCM server error. Contact system administrator.';
        severity = 'high';
      }
      // Generic admin errors
      else {
        code = 'ADMIN_GENERIC_ERROR';
        message = `Admin FCM error: ${error.message}`;
        severity = 'medium';
      }
    }

    return {
      code,
      message,
      originalError: error instanceof Error ? error : undefined,
      context,
      timestamp: new Date(),
      userAction,
      adminContext: true,
      severity
    };
  }

  /**
   * Handle critical errors that require immediate admin attention
   */
  private handleCriticalError(error: AdminFCMError): void {
    this.criticalErrors.push(error);
    
    // Keep only last 50 critical errors
    if (this.criticalErrors.length > 50) {
      this.criticalErrors = this.criticalErrors.slice(-50);
    }

    // Log to console with high visibility
    console.error('🚨 CRITICAL ADMIN FCM ERROR 🚨', error);

    // Send alert to admin monitoring system (if available)
    if (this.options.adminAlertsEnabled) {
      this.sendCriticalAlert(error);
    }
  }

  /**
   * Send critical alert to admin monitoring system
   */
  private async sendCriticalAlert(error: AdminFCMError): Promise<void> {
    try {
      await fetch('/api/v1/admin/alerts/critical', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          type: 'fcm_critical_error',
          error: {
            code: error.code,
            message: error.message,
            context: error.context,
            timestamp: error.timestamp.toISOString()
          },
          severity: 'critical',
          requiresImmediateAttention: true
        })
      });
    } catch (alertError) {
      console.error('Failed to send critical admin FCM alert:', alertError);
    }
  }

  /**
   * Determine if admin error should trigger a retry
   */
  private shouldRetryAdminOperation(error: AdminFCMError): boolean {
    if (!this.options.enableRetry) return false;
    if (error.severity === 'critical') return false; // Don't retry critical errors

    const retryableErrors = [
      'ADMIN_NETWORK_ERROR',
      'ADMIN_SERVER_ERROR',
      'ADMIN_SUBSCRIPTION_FAILED',
      'ADMIN_TEST_NOTIFICATION_FAILED'
    ];

    const currentRetries = this.retryCount.get(error.code) || 0;
    return retryableErrors.includes(error.code) && currentRetries < this.options.maxRetries;
  }

  /**
   * Schedule retry for admin operations with enhanced logging
   */
  private scheduleAdminRetry(error: AdminFCMError, context: string): void {
    const currentRetries = this.retryCount.get(error.code) || 0;
    const delay = this.options.retryDelay * Math.pow(2, currentRetries); // Exponential backoff

    this.retryCount.set(error.code, currentRetries + 1);

    setTimeout(() => {
      console.log(`Retrying admin FCM operation: ${error.code} (attempt ${currentRetries + 1})`);
      this.options.onRecovery(`Admin retry attempt ${currentRetries + 1} for ${context}`);
    }, delay);
  }

  /**
   * Apply admin-specific fallback mechanisms
   */
  private applyAdminFallback(error: AdminFCMError): void {
    if (!this.options.fallbackEnabled) return;

    switch (error.code) {
      case 'ADMIN_PERMISSION_DENIED':
      case 'ADMIN_INSUFFICIENT_PERMISSIONS':
        this.fallbackToPermissionRequest();
        break;
      case 'ADMIN_AUTH_REQUIRED':
      case 'ADMIN_SESSION_EXPIRED':
        this.fallbackToReauthentication();
        break;
      case 'ADMIN_FCM_PERMISSION_BLOCKED':
        this.fallbackToAdminInAppNotifications();
        break;
      case 'ADMIN_VAPID_KEY_MISSING':
        this.fallbackToAdminSSE();
        break;
      case 'ADMIN_NETWORK_ERROR':
        this.fallbackToAdminOfflineMode();
        break;
      case 'ADMIN_SECURITY_ERROR':
        this.fallbackToSecureMode();
        break;
      default:
        this.fallbackToAdminGenericRecovery();
    }
  }

  /**
   * Fallback to permission request flow
   */
  private fallbackToPermissionRequest(): void {
    console.log('Falling back to admin permission request');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'permission_request');
    }
  }

  /**
   * Fallback to reauthentication
   */
  private fallbackToReauthentication(): void {
    console.log('Falling back to admin reauthentication');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'reauthentication');
      // Could trigger admin login modal here
    }
  }

  /**
   * Fallback to admin in-app notifications only
   */
  private fallbackToAdminInAppNotifications(): void {
    console.log('Falling back to admin in-app notifications only');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'admin_in_app');
    }
  }

  /**
   * Fallback to admin SSE notifications
   */
  private fallbackToAdminSSE(): void {
    console.log('Falling back to admin Server-Sent Events');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'admin_sse');
    }
  }

  /**
   * Fallback to admin offline mode
   */
  private fallbackToAdminOfflineMode(): void {
    console.log('Falling back to admin offline mode');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'admin_offline');
    }
  }

  /**
   * Fallback to secure mode (enhanced security)
   */
  private fallbackToSecureMode(): void {
    console.log('Falling back to admin secure mode');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'secure_mode');
    }
  }

  /**
   * Generic admin recovery mechanism
   */
  private fallbackToAdminGenericRecovery(): void {
    console.log('Applying generic admin FCM error recovery');
    if (typeof window !== 'undefined') {
      localStorage.setItem('admin_fcm_fallback_mode', 'generic_recovery');
    }
  }

  /**
   * Log admin error to internal error log
   */
  private logAdminError(error: AdminFCMError): void {
    this.errorLog.push(error);
    
    // Keep only last 200 errors for admin interface
    if (this.errorLog.length > 200) {
      this.errorLog = this.errorLog.slice(-200);
    }
  }

  /**
   * Get admin error statistics with severity breakdown
   */
  getAdminErrorStatistics(): {
    totalErrors: number;
    criticalErrors: number;
    errorsByCode: Record<string, number>;
    errorsBySeverity: Record<string, number>;
    recentErrors: AdminFCMError[];
    recentCriticalErrors: AdminFCMError[];
  } {
    const errorsByCode: Record<string, number> = {};
    const errorsBySeverity: Record<string, number> = {};
    
    this.errorLog.forEach(error => {
      errorsByCode[error.code] = (errorsByCode[error.code] || 0) + 1;
      errorsBySeverity[error.severity] = (errorsBySeverity[error.severity] || 0) + 1;
    });

    return {
      totalErrors: this.errorLog.length,
      criticalErrors: this.criticalErrors.length,
      errorsByCode,
      errorsBySeverity,
      recentErrors: this.errorLog.slice(-10),
      recentCriticalErrors: this.criticalErrors.slice(-5)
    };
  }

  /**
   * Clear admin error logs
   */
  clearAdminErrorLog(): void {
    this.errorLog = [];
    this.criticalErrors = [];
    this.retryCount.clear();
  }

  /**
   * Check if admin FCM is in fallback mode
   */
  isInAdminFallbackMode(): string | null {
    if (typeof window === 'undefined') return null;
    return localStorage.getItem('admin_fcm_fallback_mode');
  }

  /**
   * Clear admin fallback mode
   */
  clearAdminFallbackMode(): void {
    if (typeof window !== 'undefined') {
      localStorage.removeItem('admin_fcm_fallback_mode');
    }
  }

  /**
   * Get admin-specific user-friendly error message
   */
  getAdminFriendlyMessage(error: AdminFCMError): string {
    const adminFriendlyMessages: Record<string, string> = {
      'ADMIN_PERMISSION_DENIED': 'You do not have admin permissions for FCM notifications. Contact your system administrator.',
      'ADMIN_INSUFFICIENT_PERMISSIONS': 'Insufficient admin privileges. Please contact your system administrator.',
      'ADMIN_AUTH_REQUIRED': 'Admin authentication required. Please log in to the admin interface.',
      'ADMIN_SESSION_EXPIRED': 'Your admin session has expired. Please log in again.',
      'ADMIN_FCM_PERMISSION_BLOCKED': 'Admin FCM notifications are blocked. Enable them in browser settings.',
      'ADMIN_VAPID_KEY_MISSING': 'Admin FCM configuration error. Please contact technical support.',
      'ADMIN_NETWORK_ERROR': 'Network connectivity issue. Check your connection and try again.',
      'ADMIN_SERVER_ERROR': 'Admin FCM server is temporarily unavailable. Please try again later.',
      'ADMIN_SECURITY_ERROR': 'Security error in admin FCM operations. Contact system administrator immediately.',
    };

    return adminFriendlyMessages[error.code] || 'An admin FCM error occurred. Please contact technical support.';
  }

  /**
   * Get admin-specific recovery actions
   */
  getAdminRecoveryActions(error: AdminFCMError): Array<{ label: string; action: string; severity?: string }> {
    const actions: Record<string, Array<{ label: string; action: string; severity?: string }>> = {
      'ADMIN_PERMISSION_DENIED': [
        { label: 'Contact Admin', action: 'contact_admin', severity: 'high' },
        { label: 'Check Permissions', action: 'check_permissions' }
      ],
      'ADMIN_AUTH_REQUIRED': [
        { label: 'Admin Login', action: 'admin_login', severity: 'high' },
        { label: 'Refresh Session', action: 'refresh_session' }
      ],
      'ADMIN_FCM_PERMISSION_BLOCKED': [
        { label: 'Browser Settings', action: 'open_browser_settings' },
        { label: 'Admin Guide', action: 'show_admin_guide' }
      ],
      'ADMIN_SECURITY_ERROR': [
        { label: 'Security Alert', action: 'security_alert', severity: 'critical' },
        { label: 'System Admin', action: 'contact_system_admin', severity: 'critical' }
      ],
      'ADMIN_NETWORK_ERROR': [
        { label: 'Retry', action: 'retry' },
        { label: 'Offline Mode', action: 'admin_offline_mode' }
      ]
    };

    return actions[error.code] || [
      { label: 'Retry', action: 'retry' },
      { label: 'Contact Support', action: 'contact_support', severity: 'medium' }
    ];
  }

  /**
   * Export error report for admin analysis
   */
  exportAdminErrorReport(): {
    timestamp: string;
    statistics: ReturnType<AdminFCMErrorHandler['getAdminErrorStatistics']>;
    detailedErrors: AdminFCMError[];
  } {
    return {
      timestamp: new Date().toISOString(),
      statistics: this.getAdminErrorStatistics(),
      detailedErrors: this.errorLog
    };
  }
}

// Export singleton instance
export const adminFCMErrorHandler = new AdminFCMErrorHandler({
  enableLogging: process.env.NODE_ENV === 'development',
  enableRetry: true,
  maxRetries: 3,
  retryDelay: 1000,
  fallbackEnabled: true,
  adminAlertsEnabled: process.env.NODE_ENV === 'production'
});

export default adminFCMErrorHandler;