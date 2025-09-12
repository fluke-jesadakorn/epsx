'use client';

import { useEffect } from 'react';
import { firebaseMessagingManager } from '@/lib/service-worker';

export function FirebaseMessagingInitializer() {
  useEffect(() => {
    // Only register Firebase messaging in production or when explicitly enabled
    const shouldRegisterFCM = 
      process.env.NODE_ENV === 'production' || 
      process.env.NEXT_PUBLIC_ENABLE_SW === 'true';

    if (!shouldRegisterFCM) {
      console.log('Firebase messaging registration skipped in development');
      return;
    }

    const registerFCM = async () => {
      try {
        console.log('Initializing Firebase messaging service worker...');
        
        const registration = await firebaseMessagingManager.register();
        
        if (registration) {
          console.log('Firebase messaging service worker registered successfully');
          
          // Check for updates periodically
          setInterval(async () => {
            try {
              await firebaseMessagingManager.update();
            } catch (error) {
              console.error('Firebase messaging service worker update check failed:', error);
            }
          }, 60000); // Check every minute
        } else {
          console.warn('Firebase messaging service worker registration failed');
        }
      } catch (error) {
        console.error('Firebase messaging service worker initialization error:', error);
      }
    };

    // Register Firebase messaging after a short delay to avoid blocking initial page load
    const timeoutId = setTimeout(registerFCM, 1000);

    return () => {
      clearTimeout(timeoutId);
    };
  }, []);

  // This component doesn't render anything, it just handles Firebase messaging registration
  return null;
}

// Legacy export for backward compatibility
export const ServiceWorkerInitializer = FirebaseMessagingInitializer;
export default FirebaseMessagingInitializer;