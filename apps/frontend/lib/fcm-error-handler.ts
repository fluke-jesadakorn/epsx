'use client';

export interface FCMError {
  code: string;
  message: string;
  originalError?: Error;
  context?: string;
  timestamp: Date;
  userAction?: string;
}

export interface FCMErrorHandlerOptions {
  enableLogging?: boolean;
  enableRetry?: boolean;
  maxRetries?: number;
  retryDelay?: number;
  fallbackEnabled?: boolean;
  onError?: (error: FCMError) => void;
  onRecovery?: (context: string) => void;
}

export class FCMErrorHandler {
  private options: Required<FCMErrorHandlerOptions>;
  private retryCount = new Map<string, number>();
  private errorLog: FCMError[] = [];

  constructor(options: FCMErrorHandlerOptions = {}) {
    this.options = {
      enableLogging: true,
      enableRetry: true,
      maxRetries: 3,
      retryDelay: 1000,
      fallbackEnabled: true,
      onError: () => {},
      onRecovery: () => {},
      ...options
    };
  }

  /**
   * Handle FCM errors with comprehensive error categorization
   */
  handleError(error: Error | any, context: string, userAction?: string): FCMError {
    const fcmError = this.categorizeError(error, context, userAction);
    
    // Log error if enabled
    if (this.options.enableLogging) {
      this.logError(fcmError);
      console.error(`FCM Error [${context}]:`, fcmError);
    }

    // Call custom error handler
    this.options.onError(fcmError);

    // Determine if retry is appropriate
    if (this.shouldRetry(fcmError)) {
      this.scheduleRetry(fcmError, context);
    }

    // Apply fallback mechanisms
    this.applyFallback(fcmError);

    return fcmError;
  }

  /**
   * Categorize FCM errors into specific types
   */
  private categorizeError(error: Error | any, context: string, userAction?: string): FCMError {
    let code = 'UNKNOWN_ERROR';
    let message = 'An unknown error occurred';

    if (error instanceof Error) {
      // Firebase/FCM specific errors
      if (error.message.includes('messaging/permission-blocked')) {
        code = 'PERMISSION_DENIED';
        message = 'Notification permissions have been denied. Please enable notifications in your browser settings.';
      } else if (error.message.includes('messaging/token-unsubscribe-failed')) {
        code = 'TOKEN_UNSUBSCRIBE_FAILED';
        message = 'Failed to unsubscribe FCM token. Please try again.';
      } else if (error.message.includes('messaging/vapid-key-required')) {
        code = 'VAPID_KEY_MISSING';
        message = 'VAPID key is required for FCM notifications.';
      } else if (error.message.includes('messaging/invalid-vapid-key')) {
        code = 'VAPID_KEY_INVALID';
        message = 'Invalid VAPID key configuration.';
      } else if (error.message.includes('messaging/token-subscribe-failed')) {
        code = 'TOKEN_SUBSCRIBE_FAILED';
        message = 'Failed to subscribe to FCM notifications. Please check your connection.';
      } else if (error.message.includes('messaging/failed-service-worker-registration')) {
        code = 'SERVICE_WORKER_REGISTRATION_FAILED';
        message = 'Failed to register service worker for notifications.';
      } else if (error.message.includes('messaging/notifications-blocked')) {
        code = 'NOTIFICATIONS_BLOCKED';
        message = 'Notifications are blocked in your browser. Please enable them in settings.';
      }
      // Network errors
      else if (error.message.includes('fetch')) {
        code = 'NETWORK_ERROR';
        message = 'Network error occurred while processing FCM request.';
      } else if (error.message.includes('Failed to fetch')) {
        code = 'FETCH_FAILED';
        message = 'Failed to connect to FCM service. Please check your internet connection.';
      }
      // Service worker errors
      else if (error.message.includes('service worker')) {
        code = 'SERVICE_WORKER_ERROR';
        message = 'Service worker error occurred. Please refresh the page.';
      }
      // Permission errors
      else if (error.message.includes('permission')) {
        code = 'PERMISSION_ERROR';
        message = 'Permission error. Please check notification permissions.';
      }
      // Token errors
      else if (error.message.includes('token')) {
        code = 'TOKEN_ERROR';
        message = 'FCM token error. Please try re-subscribing to notifications.';
      }
      // API errors
      else if (error.message.includes('401') || error.message.includes('Unauthorized')) {
        code = 'AUTHENTICATION_ERROR';
        message = 'Authentication error. Please log in again.';
      } else if (error.message.includes('403') || error.message.includes('Forbidden')) {
        code = 'AUTHORIZATION_ERROR';
        message = 'Not authorized to perform this action.';
      } else if (error.message.includes('500')) {
        code = 'SERVER_ERROR';
        message = 'Server error occurred. Please try again later.';
      }
      // Generic error handling
      else {
        code = 'GENERIC_ERROR';
        message = error.message || 'An error occurred with FCM notifications.';
      }
    } else if (typeof error === 'string') {
      code = 'STRING_ERROR';
      message = error;
    }

    return {
      code,
      message,
      originalError: error instanceof Error ? error : undefined,
      context,
      timestamp: new Date(),
      userAction
    };
  }

  /**
   * Determine if error should trigger a retry
   */
  private shouldRetry(error: FCMError): boolean {
    if (!this.options.enableRetry) return false;

    const retryableErrors = [
      'NETWORK_ERROR',
      'FETCH_FAILED',
      'TOKEN_SUBSCRIBE_FAILED',
      'SERVER_ERROR'
    ];

    const currentRetries = this.retryCount.get(error.code) || 0;
    return retryableErrors.includes(error.code) && currentRetries < this.options.maxRetries;
  }

  /**
   * Schedule retry for retryable errors
   */
  private scheduleRetry(error: FCMError, context: string): void {
    const currentRetries = this.retryCount.get(error.code) || 0;
    const delay = this.options.retryDelay * Math.pow(2, currentRetries); // Exponential backoff

    this.retryCount.set(error.code, currentRetries + 1);

    setTimeout(() => {
      console.log(`Retrying FCM operation after error: ${error.code} (attempt ${currentRetries + 1})`);
      this.options.onRecovery(`Retry attempt ${currentRetries + 1} for ${context}`);
    }, delay);
  }

  /**
   * Apply fallback mechanisms based on error type
   */
  private applyFallback(error: FCMError): void {
    if (!this.options.fallbackEnabled) return;

    switch (error.code) {
      case 'PERMISSION_DENIED':
      case 'NOTIFICATIONS_BLOCKED':
        this.fallbackToInAppNotifications();
        break;
      case 'SERVICE_WORKER_REGISTRATION_FAILED':
        this.fallbackToPolling();
        break;
      case 'VAPID_KEY_MISSING':
      case 'VAPID_KEY_INVALID':
        this.fallbackToSSE();
        break;
      case 'NETWORK_ERROR':
      case 'FETCH_FAILED':
        this.fallbackToOfflineMode();
        break;
      default:
        this.fallbackToGenericRecovery();
    }
  }

  /**
   * Fallback to in-app notifications only
   */
  private fallbackToInAppNotifications(): void {
    console.log('Falling back to in-app notifications only');
    // Set flag to show in-app notifications banner
    if (typeof window !== 'undefined') {
      localStorage.setItem('fcm_fallback_mode', 'in_app');
    }
  }

  /**
   * Fallback to polling for notifications
   */
  private fallbackToPolling(): void {
    console.log('Falling back to polling for notifications');
    // Enable polling mechanism
    if (typeof window !== 'undefined') {
      localStorage.setItem('fcm_fallback_mode', 'polling');
    }
  }

  /**
   * Fallback to Server-Sent Events
   */
  private fallbackToSSE(): void {
    console.log('Falling back to Server-Sent Events');
    // Enable SSE notifications
    if (typeof window !== 'undefined') {
      localStorage.setItem('fcm_fallback_mode', 'sse');
    }
  }

  /**
   * Fallback to offline mode
   */
  private fallbackToOfflineMode(): void {
    console.log('Falling back to offline mode');
    // Store notifications for when connection is restored
    if (typeof window !== 'undefined') {
      localStorage.setItem('fcm_fallback_mode', 'offline');
    }
  }

  /**
   * Generic recovery mechanism
   */
  private fallbackToGenericRecovery(): void {
    console.log('Applying generic FCM error recovery');
    // Show user-friendly error message and guidance
  }

  /**
   * Log error to internal error log
   */
  private logError(error: FCMError): void {
    this.errorLog.push(error);
    
    // Keep only last 100 errors
    if (this.errorLog.length > 100) {
      this.errorLog = this.errorLog.slice(-100);
    }
  }

  /**
   * Get error statistics
   */
  getErrorStatistics(): {
    totalErrors: number;
    errorsByCode: Record<string, number>;
    recentErrors: FCMError[];
  } {
    const errorsByCode: Record<string, number> = {};
    
    this.errorLog.forEach(error => {
      errorsByCode[error.code] = (errorsByCode[error.code] || 0) + 1;
    });

    return {
      totalErrors: this.errorLog.length,
      errorsByCode,
      recentErrors: this.errorLog.slice(-10)
    };
  }

  /**
   * Clear error log
   */
  clearErrorLog(): void {
    this.errorLog = [];
    this.retryCount.clear();
  }

  /**
   * Check if FCM is in fallback mode
   */
  isInFallbackMode(): string | null {
    if (typeof window === 'undefined') return null;
    return localStorage.getItem('fcm_fallback_mode');
  }

  /**
   * Clear fallback mode
   */
  clearFallbackMode(): void {
    if (typeof window !== 'undefined') {
      localStorage.removeItem('fcm_fallback_mode');
    }
  }

  /**
   * Get user-friendly error message
   */
  getUserFriendlyMessage(error: FCMError): string {
    const friendlyMessages: Record<string, string> = {
      'PERMISSION_DENIED': 'Push notifications are disabled. Click on the lock icon in your browser\'s address bar to enable them.',
      'NOTIFICATIONS_BLOCKED': 'Notifications are blocked. Please enable them in your browser settings.',
      'SERVICE_WORKER_REGISTRATION_FAILED': 'Unable to set up push notifications. Please refresh the page and try again.',
      'VAPID_KEY_MISSING': 'Push notification setup is incomplete. Please contact support.',
      'NETWORK_ERROR': 'Connection issue. Please check your internet connection and try again.',
      'AUTHENTICATION_ERROR': 'Please log in again to receive push notifications.',
      'SERVER_ERROR': 'Server is temporarily unavailable. Push notifications will be restored shortly.',
    };

    return friendlyMessages[error.code] || 'Something went wrong with push notifications. Please try again.';
  }

  /**
   * Get recovery actions for user
   */
  getRecoveryActions(error: FCMError): Array<{ label: string; action: string }> {
    const actions: Record<string, Array<{ label: string; action: string }>> = {
      'PERMISSION_DENIED': [
        { label: 'Enable Notifications', action: 'enable_permissions' },
        { label: 'Learn More', action: 'show_help' }
      ],
      'NOTIFICATIONS_BLOCKED': [
        { label: 'Browser Settings', action: 'open_browser_settings' },
        { label: 'Skip Notifications', action: 'skip_notifications' }
      ],
      'SERVICE_WORKER_REGISTRATION_FAILED': [
        { label: 'Refresh Page', action: 'refresh_page' },
        { label: 'Contact Support', action: 'contact_support' }
      ],
      'NETWORK_ERROR': [
        { label: 'Retry', action: 'retry' },
        { label: 'Use Offline Mode', action: 'offline_mode' }
      ]
    };

    return actions[error.code] || [
      { label: 'Retry', action: 'retry' },
      { label: 'Contact Support', action: 'contact_support' }
    ];
  }
}

// Export singleton instance
export const fcmErrorHandler = new FCMErrorHandler({
  enableLogging: process.env.NODE_ENV === 'development',
  enableRetry: true,
  maxRetries: 3,
  retryDelay: 1000,
  fallbackEnabled: true
});

export default fcmErrorHandler;