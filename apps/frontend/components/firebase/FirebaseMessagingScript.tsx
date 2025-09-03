'use client';

import Script from 'next/script';
import { useEffect, useState } from 'react';

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

  // Firebase configuration from environment variables
  const firebaseConfig = {
    apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
    authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
    projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
    storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
    messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
    appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID
  };

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
            // Register service worker for Firebase messaging
            if ('serviceWorker' in navigator) {
              navigator.serviceWorker.register('/firebase-messaging-sw.js')
                .then((registration) => {
                  console.log('Service worker registered:', registration);
                })
                .catch((error) => {
                  console.error('Service worker registration failed:', error);
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