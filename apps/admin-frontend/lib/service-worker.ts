// Service Worker registration for PWA and FCM notifications

declare global {
  interface Window {
    workbox: any;
  }
}

interface ServiceWorkerOptions {
  enableFCM?: boolean;
  enablePWA?: boolean;
  onUpdate?: () => void;
  onReady?: () => void;
  onOffline?: () => void;
}

class ServiceWorkerManager {
  private registration: ServiceWorkerRegistration | null = null;
  private options: ServiceWorkerOptions;

  constructor(options: ServiceWorkerOptions = {}) {
    this.options = {
      enableFCM: true,
      enablePWA: true,
      ...options
    };
  }

  async register(): Promise<ServiceWorkerRegistration | null> {
    if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
      console.warn('Service workers are not supported in this browser');
      return null;
    }

    try {
      // Register Firebase messaging service worker
      if (this.options.enableFCM) {
        console.log('Registering Firebase messaging service worker...');
        const fcmRegistration = await navigator.serviceWorker.register('/firebase-messaging-sw.js', {
          scope: '/firebase-cloud-messaging-push-scope'
        });

        console.log('Firebase messaging SW registered successfully:', fcmRegistration);
        
        fcmRegistration.addEventListener('updatefound', () => {
          console.log('Firebase messaging SW update found');
          this.options.onUpdate?.();
        });

        this.registration = fcmRegistration;
      }

      // Register main app service worker for PWA features
      if (this.options.enablePWA && 'serviceWorker' in navigator) {
        console.log('Registering PWA service worker...');
        const pwaRegistration = await navigator.serviceWorker.register('/sw.js');
        
        console.log('PWA SW registered successfully:', pwaRegistration);
        
        pwaRegistration.addEventListener('updatefound', () => {
          console.log('PWA SW update found');
          this.options.onUpdate?.();
        });

        // If we don't have FCM registration, use PWA registration as primary
        if (!this.registration) {
          this.registration = pwaRegistration;
        }
      }

      // Set up event listeners
      this.setupEventListeners();

      this.options.onReady?.();
      return this.registration;

    } catch (error) {
      console.error('Service worker registration failed:', error);
      return null;
    }
  }

  private setupEventListeners(): void {
    if (!('serviceWorker' in navigator)) return;

    // Listen for service worker updates
    navigator.serviceWorker.addEventListener('message', (event) => {
      console.log('Received message from service worker:', event.data);
      
      if (event.data?.type === 'SW_UPDATE_AVAILABLE') {
        this.options.onUpdate?.();
      }
    });

    // Listen for connectivity changes
    window.addEventListener('online', () => {
      console.log('App came back online');
    });

    window.addEventListener('offline', () => {
      console.log('App went offline');
      this.options.onOffline?.();
    });

    // Listen for app installation prompt
    window.addEventListener('beforeinstallprompt', (e) => {
      console.log('PWA install prompt available');
      // Store the event for later use
      (window as any).deferredInstallPrompt = e;
    });

    // Listen for successful app installation
    window.addEventListener('appinstalled', () => {
      console.log('PWA installed successfully');
      (window as any).deferredInstallPrompt = null;
    });
  }

  async unregister(): Promise<boolean> {
    if (!this.registration) return false;

    try {
      const result = await this.registration.unregister();
      this.registration = null;
      console.log('Service worker unregistered successfully');
      return result;
    } catch (error) {
      console.error('Service worker unregistration failed:', error);
      return false;
    }
  }

  async update(): Promise<void> {
    if (!this.registration) return;

    try {
      await this.registration.update();
      console.log('Service worker update triggered');
    } catch (error) {
      console.error('Service worker update failed:', error);
    }
  }

  getRegistration(): ServiceWorkerRegistration | null {
    return this.registration;
  }

  isSupported(): boolean {
    return typeof window !== 'undefined' && 'serviceWorker' in navigator;
  }

  async getNotificationPermission(): Promise<NotificationPermission> {
    if (!('Notification' in window)) {
      return 'denied';
    }
    return Notification.permission;
  }

  async requestNotificationPermission(): Promise<boolean> {
    if (!('Notification' in window)) {
      return false;
    }

    if (Notification.permission === 'granted') {
      return true;
    }

    const permission = await Notification.requestPermission();
    return permission === 'granted';
  }

  // PWA installation helpers
  async installPWA(): Promise<boolean> {
    const deferredPrompt = (window as any).deferredInstallPrompt;
    
    if (!deferredPrompt) {
      console.log('PWA install prompt not available');
      return false;
    }

    try {
      await deferredPrompt.prompt();
      const { outcome } = await deferredPrompt.userChoice;
      
      if (outcome === 'accepted') {
        console.log('User accepted PWA install prompt');
        (window as any).deferredInstallPrompt = null;
        return true;
      } else {
        console.log('User dismissed PWA install prompt');
        return false;
      }
    } catch (error) {
      console.error('PWA installation failed:', error);
      return false;
    }
  }

  isPWAInstallable(): boolean {
    return !!(window as any).deferredInstallPrompt;
  }

  isPWAInstalled(): boolean {
    return window.matchMedia('(display-mode: standalone)').matches ||
           (navigator as any).standalone === true ||
           document.referrer.includes('android-app://');
  }
}

// Export singleton instance
export const serviceWorkerManager = new ServiceWorkerManager({
  enableFCM: true,
  enablePWA: true,
  onUpdate: () => {
    console.log('Service worker update available');
    // You can show a toast notification here
  },
  onReady: () => {
    console.log('Service worker ready');
  },
  onOffline: () => {
    console.log('App is offline');
    // You can show offline indicator here
  }
});

// Convenience functions
export const registerServiceWorker = () => serviceWorkerManager.register();
export const unregisterServiceWorker = () => serviceWorkerManager.unregister();
export const updateServiceWorker = () => serviceWorkerManager.update();
export const isServiceWorkerSupported = () => serviceWorkerManager.isSupported();
export const installPWA = () => serviceWorkerManager.installPWA();
export const isPWAInstallable = () => serviceWorkerManager.isPWAInstallable();
export const isPWAInstalled = () => serviceWorkerManager.isPWAInstalled();

export default serviceWorkerManager;