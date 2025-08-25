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
        
        const registration = await serviceWorkerManager.register();
        
        if (registration) {
          console.log('Service workers registered successfully');
          
          // Check for updates periodically
          setInterval(async () => {
            try {
              await serviceWorkerManager.update();
            } catch (error) {
              console.error('Service worker update check failed:', error);
            }
          }, 60000); // Check every minute
        } else {
          console.warn('Service worker registration failed');
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