'use client';

import { useEffect } from 'react';
import { serviceWorkerManager } from '@/lib/service-worker';

export function ServiceWorkerInitializer() {
  useEffect(() => {
    // Only register service workers in production or when explicitly enabled
    const shouldRegisterSW = 
      process.env.NODE_ENV === 'production' || 
      process.env.NEXT_PUBLIC_ENABLE_SW === 'true';

    if (!shouldRegisterSW) {
      console.log('Service worker registration skipped in development');
      return;
    }

    const registerSW = async () => {
      try {
        console.log('Initializing service workers...');
        
        // Register only Firebase messaging service worker to avoid /sw.js 404 error
        let registration = null;
        if ('serviceWorker' in navigator) {
          registration = await navigator.serviceWorker.register('/firebase-messaging-sw.js', {
            scope: '/firebase-cloud-messaging-push-scope'
          });
          console.log('Firebase messaging SW registered successfully:', registration);
          
          // Check for updates periodically
          setInterval(async () => {
            try {
              await registration?.update();
            } catch (error) {
              console.error('Service worker update check failed:', error);
            }
          }, 60000); // Check every minute
        } else {
          console.warn('Service workers not supported');
        }
      } catch (error) {
        console.error('Service worker initialization error:', error);
      }
    };

    // Register service workers after a short delay to avoid blocking initial page load
    const timeoutId = setTimeout(registerSW, 1000);

    return () => {
      clearTimeout(timeoutId);
    };
  }, []);

  // This component doesn't render anything, it just handles SW registration
  return null;
}

export default ServiceWorkerInitializer;