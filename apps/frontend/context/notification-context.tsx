'use client';

import React, { createContext, useContext, useCallback, useEffect, useMemo, useRef } from 'react';
import { useAppState } from './app-state';
import { NotificationState, Notification, NotificationPreferences } from '@/lib/state/types';
import { useOptimisticUpdates } from '@/lib/state/core';
import { logger } from '@/lib/logger';

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
  
  // Push notifications
  requestPermission: () => Promise<boolean>;
  subscribeToPush: () => Promise<void>;
  unsubscribeFromPush: () => Promise<void>;
  
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
  const wsRef = useRef<WebSocket | null>(null);
  const pushSubscriptionRef = useRef<PushSubscription | null>(null);
  
  const {
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate
  } = useOptimisticUpdates();

  // WebSocket connection for real-time notifications
  const connectWebSocket = useCallback(() => {
    if (typeof window === 'undefined' || wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const wsUrl = process.env.NODE_ENV === 'development' 
        ? 'ws://localhost:8080/ws/notifications'
        : 'wss://api.epsx.com/ws/notifications';
      
      wsRef.current = new WebSocket(wsUrl);
      
      wsRef.current.onopen = () => {
        logger.info('Notifications WebSocket connected');
        actions.notifications.setRealtimeStatus({ 
          connected: true, 
          lastSync: Date.now() 
        });
      };

      wsRef.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          
          switch (data.type) {
            case 'new_notification':
              actions.notifications.addNotification(data.notification);
              // Show browser notification if permission granted
              showBrowserNotification(data.notification);
              break;
            case 'notification_read':
              actions.notifications.markRead(data.notificationId);
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
          logger.error('Error parsing notification WebSocket message', { error: error instanceof Error ? error.message : String(error) });
        }
      };

      wsRef.current.onclose = () => {
        logger.info('Notifications WebSocket disconnected');
        actions.notifications.setRealtimeStatus({ connected: false });
        
        // Reconnect after 5 seconds
        setTimeout(connectWebSocket, 5000);
      };

      wsRef.current.onerror = (error) => {
        logger.error('Notifications WebSocket error', { error: error instanceof Error ? error.message : String(error) });
        actions.notifications.setRealtimeStatus({ connected: false });
      };
    } catch (error) {
      logger.error('Failed to connect notification WebSocket', { error: error instanceof Error ? error.message : String(error) });
    }
  }, [actions.notifications]);

  // Initialize WebSocket connection
  useEffect(() => {
    connectWebSocket();
    
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connectWebSocket]);

  // Browser notification helper
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

  // API calls
  const markReadAPI = useCallback(async (id: string) => {
    const response = await fetch(`/api/notifications/${id}/read`, {
      method: 'POST',
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Failed to mark notification as read');
    }
  }, []);

  const markAllReadAPI = useCallback(async () => {
    const response = await fetch('/api/notifications/read-all', {
      method: 'POST',
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Failed to mark all notifications as read');
    }
  }, []);

  const updatePreferencesAPI = useCallback(async (preferences: Partial<NotificationPreferences>) => {
    const response = await fetch('/api/notifications/preferences', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(preferences)
    });
    
    if (!response.ok) {
      throw new Error('Failed to update notification preferences');
    }
    
    return response.json();
  }, []);

  const deleteNotificationAPI = useCallback(async (id: string) => {
    const response = await fetch(`/api/notifications/${id}`, {
      method: 'DELETE',
      credentials: 'include'
    });
    
    if (!response.ok) {
      throw new Error('Failed to delete notification');
    }
  }, []);

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
        fetch('/api/notifications', { credentials: 'include' }),
        fetch('/api/notifications/preferences', { credentials: 'include' })
      ]);

      const [notificationsData, preferencesData] = await Promise.all([
        notificationsRes.ok ? notificationsRes.json() : { notifications: [], unreadCount: 0 },
        preferencesRes.ok ? preferencesRes.json() : null
      ]);

      actions.notifications.setNotifications(notificationsData.notifications);
      
      if (preferencesData) {
        actions.notifications.updatePreferences(preferencesData);
      }
    } catch (error) {
      logger.error('Failed to refresh notifications', { error: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }, [actions.notifications]);

  // Push notification support
  const requestPermission = useCallback(async () => {
    if (typeof window === 'undefined' || !('Notification' in window)) {
      return false;
    }

    if (Notification.permission === 'granted') {
      return true;
    }

    const permission = await Notification.requestPermission();
    return permission === 'granted';
  }, []);

  const subscribeToPush = useCallback(async () => {
    if (typeof window === 'undefined' || !('serviceWorker' in navigator) || !('PushManager' in window)) {
      throw new Error('Push notifications not supported');
    }

    try {
      const registration = await navigator.serviceWorker.ready;
      const subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: process.env.NEXT_PUBLIC_VAPID_PUBLIC_KEY
      });

      pushSubscriptionRef.current = subscription;

      // Send subscription to server
      await fetch('/api/notifications/push-subscription', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(subscription)
      });
    } catch (error) {
      logger.error('Failed to subscribe to push notifications', { error: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }, []);

  const unsubscribeFromPush = useCallback(async () => {
    if (pushSubscriptionRef.current) {
      await pushSubscriptionRef.current.unsubscribe();
      pushSubscriptionRef.current = null;

      // Notify server
      await fetch('/api/notifications/push-subscription', {
        method: 'DELETE',
        credentials: 'include'
      });
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
    
    // Push notifications
    requestPermission,
    subscribeToPush,
    unsubscribeFromPush,
    
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
    updatePreferences
  } = useNotifications();
  
  return { 
    requestPermission, 
    subscribeToPush, 
    unsubscribeFromPush,
    pushEnabled: preferences.push,
    setPushEnabled: (enabled: boolean) => updatePreferences({ push: enabled })
  };
}