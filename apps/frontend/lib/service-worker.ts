// Firebase Cloud Messaging Service Worker registration

interface FirebaseMessagingOptions {
  onUpdate?: () => void;
  onReady?: () => void;
  onOffline?: () => void;
}

class FirebaseMessagingManager {
  private registration: ServiceWorkerRegistration | null = null;
  private options: FirebaseMessagingOptions;

  constructor(options: FirebaseMessagingOptions = {}) {
    this.options = options;
  }

  async register(): Promise<ServiceWorkerRegistration | null> {
    if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
      console.warn('Service workers are not supported in this browser');
      return null;
    }

    try {
      // Register Firebase messaging service worker only
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

      // Set up event listeners
      this.setupEventListeners();

      this.options.onReady?.();
      return this.registration;

    } catch (error) {
      console.error('Firebase messaging service worker registration failed:', error);
      return null;
    }
  }

  private setupEventListeners(): void {
    if (!('serviceWorker' in navigator)) return;

    // Listen for service worker updates
    navigator.serviceWorker.addEventListener('message', (event) => {
      console.log('Received message from Firebase messaging service worker:', event.data);
      
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
  }

  async unregister(): Promise<boolean> {
    if (!this.registration) return false;

    try {
      const result = await this.registration.unregister();
      this.registration = null;
      console.log('Firebase messaging service worker unregistered successfully');
      return result;
    } catch (error) {
      console.error('Firebase messaging service worker unregistration failed:', error);
      return false;
    }
  }

  async update(): Promise<void> {
    if (!this.registration) return;

    try {
      await this.registration.update();
      console.log('Firebase messaging service worker update triggered');
    } catch (error) {
      console.error('Firebase messaging service worker update failed:', error);
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
}

// Export singleton instance
export const firebaseMessagingManager = new FirebaseMessagingManager({
  onUpdate: () => {
    console.log('Firebase messaging service worker update available');
  },
  onReady: () => {
    console.log('Firebase messaging service worker ready');
  },
  onOffline: () => {
    console.log('App is offline');
  }
});

// Convenience functions
export const registerFirebaseMessaging = () => firebaseMessagingManager.register();
export const unregisterFirebaseMessaging = () => firebaseMessagingManager.unregister();
export const updateFirebaseMessaging = () => firebaseMessagingManager.update();
export const isFirebaseMessagingSupported = () => firebaseMessagingManager.isSupported();
export const getNotificationPermission = () => firebaseMessagingManager.getNotificationPermission();
export const requestNotificationPermission = () => firebaseMessagingManager.requestNotificationPermission();

// Legacy compatibility (deprecated - use Firebase-specific functions)
export const serviceWorkerManager = firebaseMessagingManager;
export const registerServiceWorker = registerFirebaseMessaging;
export const unregisterServiceWorker = unregisterFirebaseMessaging;
export const updateServiceWorker = updateFirebaseMessaging;
export const isServiceWorkerSupported = isFirebaseMessagingSupported;

export default firebaseMessagingManager;