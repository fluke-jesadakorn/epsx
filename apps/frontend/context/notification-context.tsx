'use client';

import React, { createContext, useContext, useCallback, useEffect, useMemo, useRef } from 'react';
import { useAppState } from './app-state';
import { NotificationState as _NotificationState,  NotificationPreferences } from '@/lib/state/types';
import type {Notification} from '@/lib/state/types';
import { useOptimisticUpdates } from '@/lib/state/core';
// Direct API implementation using fetch to avoid bundling complexity
import { 
  requestNotificationPermission, 
  getFCMToken, 
  onForegroundMessage, 
  isFCMSupported,
  getNotificationPermissionStatus 
} from '@/lib/firebase-config';

interface NotificationContextType {
  // Data
  notifications: Notification[];
  unreadCount: number;
  preferences: NotificationPreferences;
  
  // Loading states
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
  
  // Realtime
  realtimeConnected: boolean;
  lastSync: number | null;
  
  // Actions
  setNotifications: (notifications: Notification[]) => void;
  addNotification: (notification: Omit<Notification, 'id' | 'createdAt'>) => void;
  markRead: (id: string, optimistic?: boolean) => Promise<void>;
  markAllRead: (optimistic?: boolean) => Promise<void>;
  updatePreferences: (preferences: Partial<NotificationPreferences>, optimistic?: boolean) => Promise<void>;
  deleteNotification: (id: string, optimistic?: boolean) => Promise<void>;
  refreshNotifications: () => Promise<void>;
  
  // Push notifications (FCM)
  requestPermission: () => Promise<boolean>;
  subscribeToPush: () => Promise<void>;
  unsubscribeFromPush: () => Promise<void>;
  fcmSupported: boolean;
  fcmToken: string | null;
  
  // Helpers
  getUnreadNotifications: () => Notification[];
  getNotificationsByType: (type: Notification['type']) => Notification[];
  hasUnreadNotifications: () => boolean;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

interface NotificationProviderProps {
  children: React.ReactNode;
}

export function NotificationProvider({ children }: NotificationProviderProps) {
  const { state, actions } = useAppState();
  const { notifications } = state;
  const sseRef = useRef<EventSource | null>(null);
  const fcmTokenRef = useRef<string | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const unsubscribeForegroundRef = useRef<(() => void) | null>(null);
  
  const {
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate
  } = useOptimisticUpdates();

  // Initialize API client to use Next.js API routes
  const notificationApiClient = useMemo(() => {
    // Direct API client using fetch to avoid webpack bundling issues
    return {
      async markNotificationRead(id: string) {
        const response = await fetch(`/api/v1/notifications/read/${id}`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' }
        });
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        return await response.json();
      },
      
      async markAllNotificationsRead() {
        const response = await fetch('/api/v1/notifications/read-all', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mark_all: true, notification_ids: [] })
        });
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        return await response.json();
      },
      
      async getNotifications(page = 1, per_page = 20) {
        const response = await fetch(`/api/v1/notifications?page=${page}&per_page=${per_page}`, {
          method: 'GET',
          headers: { 'Content-Type': 'application/json' }
        });
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const data = await response.json();
        return {
          data: {
            notifications: data.notifications?.map((n: any) => ({
              id: n.id,
              category: n.notification_type,
              title: n.title,
              message: n.message,
              status: n.status,
              created_at: n.created_at,
              metadata: n.metadata
            })) || []
          }
        };
      },
      
      async updateNotificationPreferences(preferences: any) {
        // For now, return success - preferences can be implemented later
        return { data: preferences };
      },
      
      async getNotificationPreferences() {
        // For now, return default preferences
        return {
          data: {
            email_enabled: true,
            push_enabled: true,
            in_app_enabled: true,
            categories: [
              { category: 'trading', enabled: true },
              { category: 'system', enabled: true }
            ]
          }
        };
      },
      
      async deleteNotification(id: string) {
        const response = await fetch(`/api/v1/notifications/${id}`, {
          method: 'DELETE',
          headers: { 'Content-Type': 'application/json' }
        });
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        return await response.json();
      }
    };
  }, []);

  // Browser notification helper - moved to top
  const showBrowserNotification = useCallback((notification: Notification) => {
    if (typeof window === 'undefined' || !('Notification' in window)) {
      return;
    }

    if (Notification.permission === 'granted') {
      const browserNotification = new Notification(notification.title, {
        body: notification.message,
        icon: '/favicon.ico',
        badge: '/favicon.ico',
        tag: notification.id,
        requireInteraction: notification.type === 'trading',
        data: {
          notificationId: notification.id,
          actionUrl: notification.actionUrl
        }
      });

      browserNotification.onclick = () => {
        window.focus();
        if (notification.actionUrl) {
          window.location.href = notification.actionUrl;
        }
        browserNotification.close();
      };

      // Auto-close after 10 seconds
      setTimeout(() => {
        browserNotification.close();
      }, 10000);
    }
  }, []);

  // SSE connection for real-time notifications (Cloud Run compatible)
  const connectSSE = useCallback(() => {
    if (typeof window === 'undefined' || sseRef.current?.readyState === EventSource.OPEN) {
      return;
    }

    try {
      const sseUrl = process.env.NODE_ENV === 'development' 
        ? 'http://localhost:8080/api/v1/realtime/events'
        : `${process.env.NEXT_PUBLIC_API_URL}/api/v1/realtime/events`;
      
      sseRef.current = new EventSource(sseUrl, {
        withCredentials: true
      });
      
      sseRef.current.onopen = () => {
        // Notifications SSE connected
        actions.notifications.setRealtimeStatus({ 
          connected: true, 
          lastSync: Date.now() 
        });
        
        // Clear any reconnect timeout
        if (reconnectTimeoutRef.current) {
          clearTimeout(reconnectTimeoutRef.current);
          reconnectTimeoutRef.current = null;
        }
      };

      sseRef.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          
          switch (data.type) {
            case 'new_notification':
              // Convert backend format to frontend format
              const frontendNotification: Notification = {
                id: data.notification.id,
                type: mapNotificationCategory(data.notification.category),
                title: data.notification.title,
                message: data.notification.message,
                read: data.notification.status === 'read',
                actionUrl: data.notification.metadata?.action_url,
                createdAt: data.notification.created_at,
                metadata: data.notification.metadata
              };
              actions.notifications.addNotification(frontendNotification);
              // Show browser notification if permission granted
              showBrowserNotification(frontendNotification);
              break;
            case 'notification_read':
              actions.notifications.markRead(data.notification_id);
              break;
            case 'bulk_read':
              actions.notifications.markAllRead();
              break;
            case 'heartbeat':
              actions.notifications.setRealtimeStatus({ 
                connected: true, 
                lastSync: Date.now() 
              });
              break;
          }
        } catch (error) {
          console.error('Error parsing notification SSE message', { error: error instanceof Error ? error.message : String(error) });
        }
      };

      sseRef.current.onerror = (error) => {
        console.error('Notifications SSE error', { error: error instanceof Error ? error.message : String(error) });
        actions.notifications.setRealtimeStatus({ connected: false });
        
        // Reconnect with exponential backoff
        if (!reconnectTimeoutRef.current) {
          const reconnectDelay = Math.min(5000 * Math.pow(2, Math.random()), 30000); // 5s to 30s
          reconnectTimeoutRef.current = setTimeout(() => {
            if (sseRef.current?.readyState !== EventSource.OPEN) {
              connectSSE();
            }
          }, reconnectDelay);
        }
      };
    } catch (error) {
      console.error('Failed to connect notification SSE', { error: error instanceof Error ? error.message : String(error) });
    }
  }, [actions.notifications]);

  // Helper function to map backend categories to frontend types
  const mapNotificationCategory = (category: string): Notification['type'] => {
    switch (category) {
      case 'trading': return 'trading';
      case 'system': return 'system';
      case 'payment': return 'account';
      case 'security': return 'system';
      default: return 'system';
    }
  };

  // Initialize FCM foreground message listener
  useEffect(() => {
    if (isFCMSupported()) {
      const unsubscribe = onForegroundMessage((payload) => {
        console.log('FCM foreground message received:', payload);
        
        // Convert FCM payload to frontend notification format
        const notification: Notification = {
          id: payload.data?.notificationId || Math.random().toString(36),
          type: (payload.data?.type as Notification['type']) || 'system',
          title: payload.notification?.title || payload.data?.title || 'New Notification',
          message: payload.notification?.body || payload.data?.body || '',
          read: false,
          actionUrl: payload.data?.url || payload.fcmOptions?.link,
          createdAt: new Date().toISOString(),
          metadata: payload.data ? { ...payload.data } : undefined
        };
        
        actions.notifications.addNotification(notification);
        showBrowserNotification(notification);
      });
      
      unsubscribeForegroundRef.current = unsubscribe;
    }
    
    return () => {
      if (unsubscribeForegroundRef.current) {
        unsubscribeForegroundRef.current();
        unsubscribeForegroundRef.current = null;
      }
    };
  }, [actions.notifications, showBrowserNotification]);

  // Initialize SSE connection
  useEffect(() => {
    connectSSE();
    
    return () => {
      if (sseRef.current) {
        sseRef.current.close();
        sseRef.current = null;
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
    };
  }, [connectSSE]);


  // API calls using direct fetch client
  const markReadAPI = useCallback(async (id: string) => {
    try {
      await notificationApiClient.markNotificationRead(id);
    } catch (error) {
      console.error('Failed to mark notification as read:', error);
      throw error;
    }
  }, [notificationApiClient]);

  const markAllReadAPI = useCallback(async () => {
    try {
      await notificationApiClient.markAllNotificationsRead();
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error);
      throw error;
    }
  }, [notificationApiClient]);

  const updatePreferencesAPI = useCallback(async (preferences: Partial<NotificationPreferences>) => {
    try {
      const response = await notificationApiClient.updateNotificationPreferences(preferences);
      return response.data;
    } catch (error) {
      console.error('Failed to update notification preferences:', error);
      throw error;
    }
  }, [notificationApiClient]);

  const deleteNotificationAPI = useCallback(async (id: string) => {
    try {
      await notificationApiClient.deleteNotification(id);
    } catch (error) {
      console.error('Failed to delete notification:', error);
      throw error;
    }
  }, [notificationApiClient]);

  // Actions with optimistic updates
  const markRead = useCallback(async (id: string, optimistic = true) => {
    if (!optimistic) {
      await markReadAPI(id);
      actions.notifications.markRead(id);
      return;
    }

    const updateId = Math.random().toString(36);
    const currentNotification = notifications.data?.notifications.find(n => n.id === id);
    
    if (!currentNotification || currentNotification.read) {
      return; // Already read or not found
    }

    startOptimisticUpdate(
      updateId,
      () => actions.notifications.markRead(id),
      () => {
        // Rollback: mark as unread (would need a new action)
        // For now, just refresh notifications
        refreshNotifications();
      }
    );

    try {
      await markReadAPI(id);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [notifications.data?.notifications, actions.notifications, markReadAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  const markAllRead = useCallback(async (optimistic = true) => {
    if (!optimistic) {
      await markAllReadAPI();
      actions.notifications.markAllRead();
      return;
    }

    const updateId = Math.random().toString(36);
    const unreadNotifications = notifications.data?.notifications.filter(n => !n.read) || [];
    
    if (unreadNotifications.length === 0) {
      return; // Nothing to mark as read
    }

    startOptimisticUpdate(
      updateId,
      () => actions.notifications.markAllRead(),
      () => {
        // Rollback: would need to restore unread state
        refreshNotifications();
      }
    );

    try {
      await markAllReadAPI();
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [notifications.data?.notifications, actions.notifications, markAllReadAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  const updatePreferences = useCallback(async (prefs: Partial<NotificationPreferences>, optimistic = true) => {
    if (!optimistic) {
      const result = await updatePreferencesAPI(prefs);
      actions.notifications.updatePreferences(result);
      return;
    }

    const updateId = Math.random().toString(36);
    const currentPreferences = notifications.data?.preferences;
    
    startOptimisticUpdate(
      updateId,
      () => actions.notifications.updatePreferences(prefs),
      () => currentPreferences && actions.notifications.updatePreferences(currentPreferences)
    );

    try {
      const result = await updatePreferencesAPI(prefs);
      actions.notifications.updatePreferences(result);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [notifications.data?.preferences, actions.notifications, updatePreferencesAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  const deleteNotification = useCallback(async (id: string, optimistic = true) => {
    const currentNotification = notifications.data?.notifications.find(n => n.id === id);
    
    if (!optimistic) {
      await deleteNotificationAPI(id);
      // Would need a delete action in the reducer
      return;
    }

    const updateId = Math.random().toString(36);
    
    startOptimisticUpdate(
      updateId,
      () => {
        // Would need a delete action
        // For now, just filter out locally
      },
      () => currentNotification && actions.notifications.addNotification(currentNotification)
    );

    try {
      await deleteNotificationAPI(id);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [notifications.data?.notifications, actions.notifications, deleteNotificationAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  // Refresh notifications
  const refreshNotifications = useCallback(async () => {
    try {
      const [notificationsRes, preferencesRes] = await Promise.all([
        notificationApiClient.getNotifications(),
        notificationApiClient.getNotificationPreferences()
      ]);

      // Handle notifications response
      if (notificationsRes?.data?.notifications) {
        // Convert backend format to frontend format
        const backendNotifications = notificationsRes.data.notifications;
        const frontendNotifications: Notification[] = backendNotifications.map((n: any) => ({
          id: n.id,
          type: mapNotificationCategory(n.category),
          title: n.title,
          message: n.message,
          read: n.status === 'read',
          actionUrl: n.metadata?.action_url,
          createdAt: n.created_at,
          metadata: n.metadata
        }));
        
        actions.notifications.setNotifications(frontendNotifications);
      } else {
        console.warn('No notifications data received');
        actions.notifications.setNotifications([]);
      }

      // Handle preferences response
      if (preferencesRes?.data) {
        // Convert backend format to frontend format
        const backendPrefs = preferencesRes.data;
        const frontendPrefs: NotificationPreferences = {
          email: backendPrefs.email_enabled,
          push: backendPrefs.push_enabled,
          inApp: backendPrefs.in_app_enabled,
          tradingAlerts: backendPrefs.categories.some((c: any) => c.category === 'trading' && c.enabled),
          systemUpdates: backendPrefs.categories.some((c: any) => c.category === 'system' && c.enabled)
        };
        actions.notifications.updatePreferences(frontendPrefs);
      } else {
        console.warn('No preferences data received');
      }
    } catch (error) {
      console.error('Failed to refresh notifications', { error: error instanceof Error ? error.message : String(error) });
      // Set empty state on error to avoid undefined state
      actions.notifications.setNotifications([]);
      throw error;
    }
  }, [actions.notifications, notificationApiClient, mapNotificationCategory]);

  // FCM Push notification support
  const requestPermission = useCallback(async () => {
    if (!isFCMSupported()) {
      console.warn('FCM not supported on this browser');
      return false;
    }

    try {
      const token = await requestNotificationPermission();
      if (token) {
        fcmTokenRef.current = token;
        return true;
      }
      return false;
    } catch (error) {
      console.error('Failed to request FCM permission:', error);
      return false;
    }
  }, []);

  const subscribeToPush = useCallback(async () => {
    if (!isFCMSupported()) {
      throw new Error('FCM not supported on this browser');
    }

    try {
      // Request permission and get FCM token
      let token = fcmTokenRef.current;
      if (!token) {
        token = await requestNotificationPermission();
        if (!token) {
          throw new Error('Failed to get FCM token');
        }
        fcmTokenRef.current = token;
      }

      // Register FCM token with backend via Next.js API route
      const response = await fetch('/api/v1/notifications/fcm/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          fcmToken: token,
          userAgent: navigator.userAgent,
          platform: navigator.platform
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to register FCM token');
      }

      console.log('Successfully subscribed to FCM notifications');
    } catch (error) {
      console.error('Failed to subscribe to FCM notifications:', error);
      throw error;
    }
  }, []);

  const unsubscribeFromPush = useCallback(async () => {
    if (!fcmTokenRef.current) {
      return;
    }

    try {
      // Unregister FCM token with backend
      const response = await fetch('/api/v1/notifications/fcm/unsubscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          fcmToken: fcmTokenRef.current 
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        console.error('Failed to unregister FCM token:', errorData.error);
      } else {
        console.log('Successfully unsubscribed from FCM notifications');
      }
      
      fcmTokenRef.current = null;
    } catch (error) {
      console.error('Failed to unsubscribe from FCM notifications:', error);
    }
  }, []);

  // Helper functions
  const getUnreadNotifications = useCallback(() => {
    return notifications.data?.notifications.filter(n => !n.read) || [];
  }, [notifications.data?.notifications]);

  const getNotificationsByType = useCallback((type: Notification['type']) => {
    return notifications.data?.notifications.filter(n => n.type === type) || [];
  }, [notifications.data?.notifications]);

  const hasUnreadNotifications = useCallback(() => {
    return (notifications.data?.unreadCount || 0) > 0;
  }, [notifications.data?.unreadCount]);

  // Add new notification with auto-generated ID
  const addNotification = useCallback((notification: Omit<Notification, 'id' | 'createdAt'>) => {
    const newNotification: Notification = {
      ...notification,
      id: Math.random().toString(36),
      createdAt: new Date().toISOString()
    };
    
    actions.notifications.addNotification(newNotification);
    showBrowserNotification(newNotification);
  }, [actions.notifications, showBrowserNotification]);

  const contextValue = useMemo(() => ({
    // Data
    notifications: notifications.data?.notifications || [],
    unreadCount: notifications.data?.unreadCount || 0,
    preferences: notifications.data?.preferences || {
      email: true,
      push: true,
      inApp: true,
      tradingAlerts: true,
      systemUpdates: true
    },
    
    // Loading states
    loading: notifications.loading,
    error: notifications.error,
    lastUpdated: notifications.lastUpdated,
    
    // Realtime
    realtimeConnected: notifications.realtime.connected,
    lastSync: notifications.realtime.lastSync,
    
    // Actions
    setNotifications: actions.notifications.setNotifications,
    addNotification,
    markRead,
    markAllRead,
    updatePreferences,
    deleteNotification,
    refreshNotifications,
    
    // Push notifications (FCM)
    requestPermission,
    subscribeToPush,
    unsubscribeFromPush,
    fcmSupported: isFCMSupported(),
    fcmToken: fcmTokenRef.current,
    
    // Helpers
    getUnreadNotifications,
    getNotificationsByType,
    hasUnreadNotifications
  }), [
    notifications,
    actions.notifications,
    addNotification,
    markRead,
    markAllRead,
    updatePreferences,
    deleteNotification,
    refreshNotifications,
    requestPermission,
    subscribeToPush,
    unsubscribeFromPush,
    getUnreadNotifications,
    getNotificationsByType,
    hasUnreadNotifications
  ]);

  return (
    <NotificationContext.Provider value={contextValue}>
      {children}
    </NotificationContext.Provider>
  );
}

export function useNotifications() {
  const context = useContext(NotificationContext);
  if (context === undefined) {
    throw new Error('useNotifications must be used within a NotificationProvider');
  }
  return context;
}

// Specialized hooks
export function useNotificationPreferences() {
  const { preferences, updatePreferences } = useNotifications();
  return { preferences, updatePreferences };
}

export function useUnreadNotifications() {
  const { 
    unreadCount, 
    getUnreadNotifications, 
    hasUnreadNotifications, 
    markRead, 
    markAllRead 
  } = useNotifications();
  
  return { 
    count: unreadCount, 
    notifications: getUnreadNotifications(), 
    hasUnread: hasUnreadNotifications(), 
    markRead, 
    markAllRead 
  };
}

export function usePushNotifications() {
  const { 
    requestPermission, 
    subscribeToPush, 
    unsubscribeFromPush,
    preferences,
    updatePreferences,
    fcmSupported,
    fcmToken
  } = useNotifications();
  
  return { 
    requestPermission, 
    subscribeToPush, 
    unsubscribeFromPush,
    pushEnabled: preferences.push,
    setPushEnabled: (enabled: boolean) => updatePreferences({ push: enabled }),
    fcmSupported,
    fcmToken,
    isSubscribed: !!fcmToken
  };
}