'use client';

import Script from 'next/script';
import { useEffect, useState } from 'react';
import { clientConfig } from '@/config/env';

interface FirebaseMessagingScriptProps {
  onMessagingReady?: (messaging: any) => void;
  onError?: (error: Error) => void;
}

export default function FirebaseMessagingScript({ 
  onMessagingReady, 
  onError 
}: FirebaseMessagingScriptProps) {
  const [scriptsLoaded, setScriptsLoaded] = useState(false);
  const [firebaseApp, setFirebaseApp] = useState<any>(null);

  // Firebase configuration from centralized config
  const firebaseConfig = clientConfig.firebase;

  const initializeFirebase = () => {
    if (typeof window === 'undefined' || !scriptsLoaded) return;

    try {
      // Check if Firebase scripts are loaded
      if (typeof window.firebase === 'undefined') {
        console.warn('Firebase scripts not loaded yet');
        return;
      }

      // Validate configuration
      if (!firebaseConfig.apiKey) {
        console.warn('Missing Firebase API key');
        onError?.(new Error('Invalid Firebase configuration'));
        return;
      }
      
      // Check if all required fields are present
      const requiredFields = ['apiKey', 'authDomain', 'projectId', 'messagingSenderId', 'appId'];
      const missingFields = requiredFields.filter(field => !firebaseConfig[field as keyof typeof firebaseConfig]);
      if (missingFields.length > 0) {
        console.warn('Missing Firebase config fields:', missingFields);
        onError?.(new Error(`Missing Firebase configuration: ${missingFields.join(', ')}`));
        return;
      }

      // Initialize Firebase app if not already initialized
      let app;
      try {
        app = window.firebase.app();
      } catch (e) {
        app = window.firebase.initializeApp(firebaseConfig);
      }
      
      setFirebaseApp(app);

      // Initialize messaging
      if ('serviceWorker' in navigator && 'PushManager' in window) {
        const messaging = window.firebase.messaging();
        onMessagingReady?.(messaging);
      } else {
        console.warn('Push messaging not supported in this browser');
      }

    } catch (error) {
      console.error('Firebase initialization error:', error);
      onError?.(error as Error);
    }
  };

  useEffect(() => {
    initializeFirebase();
  }, [scriptsLoaded]);

  const handleScriptsLoaded = () => {
    console.log('Firebase scripts loaded successfully');
    setScriptsLoaded(true);
  };

  const handleScriptError = (error: any) => {
    console.error('Firebase script loading error:', error);
    onError?.(new Error('Failed to load Firebase scripts'));
  };

  return (
    <>
      <Script
        src="https://www.gstatic.com/firebasejs/12.2.1/firebase-app-compat.js"
        strategy="afterInteractive"
        onLoad={() => {
          console.log('Firebase app script loaded');
          // Wait for messaging script to also load
        }}
        onError={handleScriptError}
      />
      <Script
        src="https://www.gstatic.com/firebasejs/12.2.1/firebase-messaging-compat.js"
        strategy="afterInteractive"
        onLoad={handleScriptsLoaded}
        onError={handleScriptError}
      />
      <Script
        id="firebase-messaging-sw-registration"
        strategy="afterInteractive"
        dangerouslySetInnerHTML={{
          __html: `
            // Register Firebase messaging service worker with proper scope
            if ('serviceWorker' in navigator) {
              navigator.serviceWorker.register('/firebase-messaging-sw.js', {
                scope: '/firebase-cloud-messaging-push-scope'
              })
                .then((registration) => {
                  console.log('Firebase messaging service worker registered:', registration);
                })
                .catch((error) => {
                  console.error('Firebase messaging service worker registration failed:', error);
                });
            }
          `
        }}
      />
      <Script
        id="firebase-messaging-sw-config"
        strategy="afterInteractive"
        dangerouslySetInnerHTML={{
          __html: `
            // Inject Firebase config into service worker global scope
            if ('serviceWorker' in navigator) {
              navigator.serviceWorker.ready.then((registration) => {
                if (registration.active) {
                  registration.active.postMessage({
                    type: 'FIREBASE_CONFIG',
                    config: {
                      apiKey: '${firebaseConfig.apiKey}',
                      authDomain: '${firebaseConfig.authDomain}',
                      projectId: '${firebaseConfig.projectId}',
                      storageBucket: '${firebaseConfig.storageBucket}',
                      messagingSenderId: '${firebaseConfig.messagingSenderId}',
                      appId: '${firebaseConfig.appId}'
                    }
                  });
                }
              });
            }
          `
        }}
      />
    </>
  );
}