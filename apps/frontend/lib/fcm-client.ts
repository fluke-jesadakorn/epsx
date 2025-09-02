'use client';

import {
  requestNotificationPermission,
  getFCMToken,
  onForegroundMessage,
  isPushNotificationSupported,
  getNotificationPermissionStatus,
  isFCMSupported,
  messaging
} from './firebase-config';
import { deleteToken } from 'firebase/messaging';

export interface FCMTokenInfo {
  token: string;
  createdAt: Date;
  userAgent: string;
  platform: string;
  isActive: boolean;
}

export interface FCMSubscriptionRequest {
  fcmToken: string;
  userAgent: string;
  platform: string;
  subscriptionId?: string;
}

export interface FCMSubscriptionResponse {
  success: boolean;
  data: {
    subscriptionId: string;
    fcmToken: string;
    subscribedAt: string;
  };
}

export interface FCMTestNotificationRequest {
  title: string;
  body: string;
  icon?: string;
  url?: string;
  data?: Record<string, any>;
}

export interface FCMNotificationPayload {
  notification?: {
    title?: string;
    body?: string;
    icon?: string;
  };
  data?: Record<string, any>;
  fcmOptions?: {
    link?: string;
  };
}

export type FCMPermissionStatus = 'granted' | 'denied' | 'default' | 'unsupported';

class FCMClient {
  private currentToken: string | null = null;
  private tokenRefreshCallbacks: Array<(token: string) => void> = [];
  private messageCallbacks: Array<(payload: FCMNotificationPayload) => void> = [];
  private unsubscribeTokenRefresh: (() => void) | null = null;
  private unsubscribeForegroundMessage: (() => void) | null = null;

  constructor() {
    this.initializeListeners();
  }

  /**
   * Initialize FCM listeners
   */
  private initializeListeners(): void {
    if (!isFCMSupported()) {
      console.warn('FCM not supported in this environment');
      return;
    }

    // Token refresh listener not available in this Firebase version
    // Tokens will be refreshed automatically by Firebase SDK
    console.log('Token refresh monitoring: Using automatic Firebase token refresh');

    // Set up foreground message listener
    try {
      this.unsubscribeForegroundMessage = onForegroundMessage((payload) => {
        console.log('FCM foreground message received:', payload);
        this.messageCallbacks.forEach(callback => callback(payload));
      });
    } catch (error) {
      console.error('Error setting up foreground message listener:', error);
    }
  }

  /**
   * Check if FCM is supported
   */
  isSupported(): boolean {
    return isFCMSupported();
  }

  /**
   * Get current notification permission status
   */
  getPermissionStatus(): FCMPermissionStatus {
    if (!isPushNotificationSupported()) {
      return 'unsupported';
    }
    
    const status = getNotificationPermissionStatus();
    return status || 'unsupported';
  }

  /**
   * Request notification permission and get FCM token
   */
  async requestPermission(): Promise<{ granted: boolean; token?: string }> {
    if (!this.isSupported()) {
      throw new Error('FCM not supported in this browser');
    }

    try {
      const token = await requestNotificationPermission();
      if (token) {
        this.currentToken = token;
        return { granted: true, token };
      } else {
        return { granted: false };
      }
    } catch (error) {
      console.error('Error requesting FCM permission:', error);
      throw new Error(`Failed to request notification permission: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get current FCM token
   */
  async getToken(): Promise<string | null> {
    if (!this.isSupported()) {
      return null;
    }

    try {
      const token = await getFCMToken();
      if (token) {
        this.currentToken = token;
      }
      return token;
    } catch (error) {
      console.error('Error getting FCM token:', error);
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
   * Delete current FCM token
   */
  async deleteToken(): Promise<boolean> {
    if (!this.isSupported() || !messaging) {
      return false;
    }

    try {
      const result = await deleteToken(messaging);
      if (result) {
        this.currentToken = null;
      }
      return result;
    } catch (error) {
      console.error('Error deleting FCM token:', error);
      return false;
    }
  }

  /**
   * Register FCM token with backend
   */
  private async registerTokenWithBackend(token: string): Promise<FCMSubscriptionResponse> {
    const subscriptionData: FCMSubscriptionRequest = {
      fcmToken: token,
      userAgent: navigator.userAgent,
      platform: navigator.platform
    };

    const response = await fetch('/api/v1/notifications/fcm/subscribe', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(subscriptionData)
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || 'Failed to register FCM token with backend');
    }

    return response.json();
  }

  /**
   * Subscribe to push notifications
   */
  async subscribe(): Promise<FCMSubscriptionResponse> {
    // Request permission and get token
    const { granted, token } = await this.requestPermission();
    
    if (!granted || !token) {
      throw new Error('Notification permission denied or token not available');
    }

    // Register with backend
    return this.registerTokenWithBackend(token);
  }

  /**
   * Unsubscribe from push notifications
   */
  async unsubscribe(): Promise<{ success: boolean; backendSuccess: boolean }> {
    let backendSuccess = false;
    
    // Unregister from backend if we have a token
    if (this.currentToken) {
      try {
        const response = await fetch('/api/v1/notifications/fcm/unsubscribe', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ fcmToken: this.currentToken })
        });
        
        backendSuccess = response.ok;
        if (!backendSuccess) {
          console.warn('Backend unsubscription failed, but proceeding with client cleanup');
        }
      } catch (error) {
        console.error('Error unsubscribing from backend:', error);
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
   * Send test notification
   */
  async sendTestNotification(notification: FCMTestNotificationRequest): Promise<{ success: boolean; message: string }> {
    if (!this.currentToken) {
      throw new Error('No FCM token available. Please subscribe first.');
    }

    const response = await fetch('/api/v1/notifications/fcm/test', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        ...notification,
        fcmToken: this.currentToken
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || 'Failed to send test notification');
    }

    return response.json();
  }

  /**
   * Get user's FCM tokens from backend
   */
  async getUserTokens(): Promise<FCMTokenInfo[]> {
    const response = await fetch('/api/v1/notifications/fcm/tokens/my', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      }
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || 'Failed to get user tokens');
    }

    const data = await response.json();
    return data.tokens || [];
  }

  /**
   * Add callback for token refresh events
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
   * Add callback for foreground messages
   */
  onMessage(callback: (payload: FCMNotificationPayload) => void): () => void {
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
   * Get subscription status
   */
  async getSubscriptionStatus(): Promise<{
    isSupported: boolean;
    permission: FCMPermissionStatus;
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
        console.error('Error checking subscription status:', error);
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
   * Cleanup listeners
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

// Export singleton instance
export const fcmClient = new FCMClient();

// Export convenience functions
export const requestFCMPermission = () => fcmClient.requestPermission();
export const subscribeToFCM = () => fcmClient.subscribe();
export const unsubscribeFromFCM = () => fcmClient.unsubscribe();
export const getFCMSubscriptionStatus = () => fcmClient.getSubscriptionStatus();
export const sendFCMTestNotification = (notification: FCMTestNotificationRequest) => 
  fcmClient.sendTestNotification(notification);

export default fcmClient;