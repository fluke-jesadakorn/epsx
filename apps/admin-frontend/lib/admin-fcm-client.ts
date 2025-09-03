'use client';

import {
  requestAdminNotificationPermission,
  getAdminFCMToken,
  onAdminForegroundMessage,
  isAdminPushNotificationSupported,
  getAdminNotificationPermissionStatus,
  isAdminFCMSupported,
  deleteAdminFCMToken,
  onAdminTokenRefresh,
  registerAdminServiceWorker,
  playAdminNotificationSound,
  messaging
} from './admin-fcm-config';

export interface AdminFCMTokenInfo {
  token: string;
  createdAt: Date;
  userAgent: string;
  platform: string;
  isActive: boolean;
  adminPermissions: string[];
}

export interface AdminFCMSubscriptionRequest {
  fcmToken: string;
  userAgent: string;
  platform: string;
  adminPermissions: string[];
  adminRole: string;
}

export interface AdminFCMTestNotificationRequest {
  title: string;
  body: string;
  icon?: string;
  url?: string;
  adminType?: 'user_management' | 'system_alert' | 'security_warning' | 'analytics_report' | 'permission_change';
  priority?: 'low' | 'normal' | 'high' | 'critical';
  data?: Record<string, any>;
}

export type AdminFCMPermissionStatus = 'granted' | 'denied' | 'default' | 'unsupported';

class AdminFCMClient {
  private currentToken: string | null = null;
  private tokenRefreshCallbacks: Array<(token: string) => void> = [];
  private messageCallbacks: Array<(payload: any) => void> = [];
  private unsubscribeTokenRefresh: (() => void) | null = null;
  private unsubscribeForegroundMessage: (() => void) | null = null;
  private serviceWorkerRegistration: ServiceWorkerRegistration | null = null;

  constructor() {
    this.initializeAdminListeners();
  }

  /**
   * Initialize admin-specific FCM listeners
   */
  private async initializeAdminListeners(): Promise<void> {
    if (!isAdminFCMSupported()) {
      console.warn('Admin FCM not supported in this environment');
      return;
    }

    // Register admin service worker
    try {
      this.serviceWorkerRegistration = await registerAdminServiceWorker();
    } catch (error) {
      console.error('Error registering admin service worker:', error);
    }

    // Set up token refresh listener
    try {
      this.unsubscribeTokenRefresh = onAdminTokenRefresh((token) => {
        console.log('Admin FCM token refreshed:', token);
        this.currentToken = token;
        this.tokenRefreshCallbacks.forEach(callback => callback(token));
        // Automatically re-register with backend
        this.registerAdminTokenWithBackend(token);
      });
    } catch (error) {
      console.error('Error setting up admin token refresh listener:', error);
    }

    // Set up foreground message listener with admin-specific handling
    try {
      this.unsubscribeForegroundMessage = onAdminForegroundMessage((payload) => {
        console.log('Admin FCM foreground message received:', payload);
        
        // Play admin notification sound
        if (payload.data?.adminType) {
          playAdminNotificationSound(payload.data.adminType);
        }
        
        this.messageCallbacks.forEach(callback => callback(payload));
      });
    } catch (error) {
      console.error('Error setting up admin foreground message listener:', error);
    }
  }

  /**
   * Check if admin FCM is supported
   */
  isSupported(): boolean {
    return isAdminFCMSupported();
  }

  /**
   * Get current notification permission status
   */
  getPermissionStatus(): AdminFCMPermissionStatus {
    if (!isAdminPushNotificationSupported()) {
      return 'unsupported';
    }
    
    const status = getAdminNotificationPermissionStatus();
    return status || 'unsupported';
  }

  /**
   * Request notification permission and get admin FCM token
   */
  async requestPermission(): Promise<{ granted: boolean; token?: string }> {
    if (!this.isSupported()) {
      throw new Error('Admin FCM not supported in this browser');
    }

    try {
      const token = await requestAdminNotificationPermission();
      if (token) {
        this.currentToken = token;
        return { granted: true, token };
      } else {
        return { granted: false };
      }
    } catch (error) {
      console.error('Error requesting admin FCM permission:', error);
      throw new Error(`Failed to request admin notification permission: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get current admin FCM token
   */
  async getToken(): Promise<string | null> {
    if (!this.isSupported()) {
      return null;
    }

    try {
      const token = await getAdminFCMToken();
      if (token) {
        this.currentToken = token;
      }
      return token;
    } catch (error) {
      console.error('Error getting admin FCM token:', error);
      return null;
    }
  }

  /**
   * Get current token from memory (faster than async call)
   */
  getCurrentToken(): string | null {
    return this.currentToken;
  }

  /**
   * Delete current admin FCM token
   */
  async deleteToken(): Promise<boolean> {
    if (!this.isSupported() || !messaging) {
      return false;
    }

    try {
      const result = await deleteAdminFCMToken();
      if (result) {
        this.currentToken = null;
      }
      return result;
    } catch (error) {
      console.error('Error deleting admin FCM token:', error);
      return false;
    }
  }

  /**
   * Register admin FCM token with backend
   */
  private async registerAdminTokenWithBackend(token: string): Promise<any> {
    // Get admin user permissions from session/auth
    const adminPermissions = ['admin:notifications:receive']; // This should come from auth context
    const adminRole = 'admin'; // This should come from auth context

    const subscriptionData: AdminFCMSubscriptionRequest = {
      fcmToken: token,
      userAgent: navigator.userAgent,
      platform: navigator.platform,
      adminPermissions,
      adminRole
    };

    const response = await fetch('/api/v1/admin/notifications/fcm/subscribe', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(subscriptionData)
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || 'Failed to register admin FCM token with backend');
    }

    return response.json();
  }

  /**
   * Subscribe to admin push notifications
   */
  async subscribe(): Promise<any> {
    // Request permission and get token
    const { granted, token } = await this.requestPermission();
    
    if (!granted || !token) {
      throw new Error('Admin notification permission denied or token not available');
    }

    // Register with backend
    return this.registerAdminTokenWithBackend(token);
  }

  /**
   * Unsubscribe from admin push notifications
   */
  async unsubscribe(): Promise<{ success: boolean; backendSuccess: boolean }> {
    let backendSuccess = false;
    
    // Unregister from backend if we have a token
    if (this.currentToken) {
      try {
        const response = await fetch('/api/v1/admin/notifications/fcm/unsubscribe', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          credentials: 'include',
          body: JSON.stringify({ fcmToken: this.currentToken })
        });
        
        backendSuccess = response.ok;
        if (!backendSuccess) {
          console.warn('Admin backend unsubscription failed, but proceeding with client cleanup');
        }
      } catch (error) {
        console.error('Error unsubscribing from admin backend:', error);
      }
    }

    // Delete local token
    const tokenDeleted = await this.deleteToken();
    
    return {
      success: tokenDeleted || this.currentToken === null,
      backendSuccess
    };
  }

  /**
   * Send admin test notification
   */
  async sendTestNotification(notification: AdminFCMTestNotificationRequest): Promise<{ success: boolean; message: string }> {
    if (!this.currentToken) {
      throw new Error('No admin FCM token available. Please subscribe first.');
    }

    const response = await fetch('/api/v1/admin/notifications/fcm/test', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({
        ...notification,
        fcmToken: this.currentToken,
        isAdminTest: true
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || 'Failed to send admin test notification');
    }

    return response.json();
  }

  /**
   * Get admin user's FCM tokens from backend
   */
  async getUserTokens(): Promise<AdminFCMTokenInfo[]> {
    const response = await fetch('/api/v1/admin/notifications/fcm/tokens/my', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include'
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || 'Failed to get admin user tokens');
    }

    const data = await response.json();
    return data.tokens || [];
  }

  /**
   * Add callback for admin token refresh events
   */
  onTokenRefresh(callback: (token: string) => void): () => void {
    this.tokenRefreshCallbacks.push(callback);
    
    // Return unsubscribe function
    return () => {
      const index = this.tokenRefreshCallbacks.indexOf(callback);
      if (index > -1) {
        this.tokenRefreshCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Add callback for admin foreground messages
   */
  onMessage(callback: (payload: any) => void): () => void {
    this.messageCallbacks.push(callback);
    
    // Return unsubscribe function
    return () => {
      const index = this.messageCallbacks.indexOf(callback);
      if (index > -1) {
        this.messageCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Get admin subscription status
   */
  async getSubscriptionStatus(): Promise<{
    isSupported: boolean;
    permission: AdminFCMPermissionStatus;
    hasToken: boolean;
    isSubscribed: boolean;
  }> {
    const isSupported = this.isSupported();
    const permission = this.getPermissionStatus();
    const hasToken = !!this.currentToken || !!(await this.getToken());
    
    let isSubscribed = false;
    if (hasToken) {
      try {
        const tokens = await this.getUserTokens();
        isSubscribed = tokens.some(token => token.isActive);
      } catch (error) {
        console.error('Error checking admin subscription status:', error);
      }
    }

    return {
      isSupported,
      permission,
      hasToken,
      isSubscribed
    };
  }

  /**
   * Cleanup admin listeners
   */
  cleanup(): void {
    if (this.unsubscribeTokenRefresh) {
      this.unsubscribeTokenRefresh();
      this.unsubscribeTokenRefresh = null;
    }
    
    if (this.unsubscribeForegroundMessage) {
      this.unsubscribeForegroundMessage();
      this.unsubscribeForegroundMessage = null;
    }
    
    this.tokenRefreshCallbacks = [];
    this.messageCallbacks = [];
  }
}

// Export singleton instance for admin
export const adminFCMClient = new AdminFCMClient();

// Export convenience functions
export const requestAdminFCMPermission = () => adminFCMClient.requestPermission();
export const subscribeToAdminFCM = () => adminFCMClient.subscribe();
export const unsubscribeFromAdminFCM = () => adminFCMClient.unsubscribe();
export const getAdminFCMSubscriptionStatus = () => adminFCMClient.getSubscriptionStatus();
export const sendAdminFCMTestNotification = (notification: AdminFCMTestNotificationRequest) => 
  adminFCMClient.sendTestNotification(notification);

export default adminFCMClient;