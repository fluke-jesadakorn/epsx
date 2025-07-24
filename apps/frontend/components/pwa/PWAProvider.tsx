'use client';

import { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { pwaManager } from '@/lib/pwa';
import { InstallPrompt } from './InstallPrompt';
import { UpdatePrompt } from './UpdatePrompt';

interface PWAContextType {
  isInstalled: boolean;
  isStandalone: boolean;
  canInstall: boolean;
  install: () => Promise<boolean>;
  requestNotificationPermission: () => Promise<NotificationPermission>;
  subscribeToPushNotifications: () => Promise<PushSubscription | null>;
  unsubscribeFromPushNotifications: () => Promise<boolean>;
  shareContent: (data: ShareData) => Promise<boolean>;
  notificationPermission: NotificationPermission;
}

const PWAContext = createContext<PWAContextType | undefined>(undefined);

interface PWAProviderProps {
  children: ReactNode;
  showInstallPrompt?: boolean;
  showUpdatePrompt?: boolean;
}

export function PWAProvider({ 
  children, 
  showInstallPrompt = true, 
  showUpdatePrompt = true 
}: PWAProviderProps) {
  const [isInstalled, setIsInstalled] = useState(false);
  const [isStandalone, setIsStandalone] = useState(false);
  const [canInstall, setCanInstall] = useState(false);
  const [notificationPermission, setNotificationPermission] = useState<NotificationPermission>('default');

  useEffect(() => {
    // Initialize PWA state
    const updateState = () => {
      setIsInstalled(pwaManager.isAppInstalled());
      setIsStandalone(pwaManager.isRunningStandalone());
      setCanInstall(pwaManager.canInstall());
      
      if ('Notification' in window) {
        setNotificationPermission(Notification.permission);
      }
    };

    updateState();

    // Listen for PWA events
    const handleInstallAvailable = () => {
      setCanInstall(true);
    };

    const handleInstalled = () => {
      setIsInstalled(true);
      setCanInstall(false);
    };

    window.addEventListener('pwa:installAvailable', handleInstallAvailable);
    window.addEventListener('pwa:installed', handleInstalled);

    return () => {
      window.removeEventListener('pwa:installAvailable', handleInstallAvailable);
      window.removeEventListener('pwa:installed', handleInstalled);
    };
  }, []);

  const install = async (): Promise<boolean> => {
    const success = await pwaManager.install();
    if (success) {
      setIsInstalled(true);
      setCanInstall(false);
    }
    return success;
  };

  const requestNotificationPermission = async (): Promise<NotificationPermission> => {
    const permission = await pwaManager.requestNotificationPermission();
    setNotificationPermission(permission);
    return permission;
  };

  const subscribeToPushNotifications = async (): Promise<PushSubscription | null> => {
    return await pwaManager.subscribeToPushNotifications();
  };

  const unsubscribeFromPushNotifications = async (): Promise<boolean> => {
    return await pwaManager.unsubscribeFromPushNotifications();
  };

  const shareContent = async (data: ShareData): Promise<boolean> => {
    return await pwaManager.shareContent(data);
  };

  const contextValue: PWAContextType = {
    isInstalled,
    isStandalone,
    canInstall,
    install,
    requestNotificationPermission,
    subscribeToPushNotifications,
    unsubscribeFromPushNotifications,
    shareContent,
    notificationPermission
  };

  return (
    <PWAContext.Provider value={contextValue}>
      {children}
      {showInstallPrompt && <InstallPrompt />}
      {showUpdatePrompt && <UpdatePrompt />}
    </PWAContext.Provider>
  );
}

export function usePWA(): PWAContextType {
  const context = useContext(PWAContext);
  if (context === undefined) {
    throw new Error('usePWA must be used within a PWAProvider');
  }
  return context;
}