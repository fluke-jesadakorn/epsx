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
        console.warn('[Admin] Firebase scripts not loaded yet');
        return;
      }

      // Validate configuration
      if (!firebaseConfig.apiKey) {
        console.warn('[Admin] Missing Firebase API key');
        onError?.(new Error('Invalid Firebase configuration'));
        return;
      }
      
      // Check if all required fields are present
      const requiredFields = ['apiKey', 'authDomain', 'projectId', 'messagingSenderId', 'appId'];
      const missingFields = requiredFields.filter(field => !firebaseConfig[field]);
      if (missingFields.length > 0) {
        console.warn('[Admin] Missing Firebase config fields:', missingFields);
        onError?.(new Error(`Missing Firebase configuration: ${missingFields.join(', ')}`));
        return;
      }

      // Initialize Firebase app if not already initialized
      let app;
      try {
        // Check if app with this name already exists
        app = window.firebase.app('admin-app');
        console.log('[Admin] Using existing Firebase app');
      } catch (e) {
        try {
          // Initialize with a unique name to avoid conflicts
          app = window.firebase.initializeApp(firebaseConfig, 'admin-app');
          console.log('[Admin] Firebase app initialized successfully');
        } catch (initError) {
          console.error('[Admin] Firebase app initialization failed:', initError);
          onError?.(initError as Error);
          return;
        }
      }
      
      setFirebaseApp(app);

      // Initialize messaging
      if ('serviceWorker' in navigator && 'PushManager' in window) {
        try {
          const messaging = window.firebase.messaging(app);
          console.log('[Admin] Firebase messaging initialized');
          onMessagingReady?.(messaging);
        } catch (msgError) {
          console.error('[Admin] Firebase messaging initialization failed:', msgError);
          onError?.(msgError as Error);
        }
      } else {
        console.warn('[Admin] Push messaging not supported in this browser');
      }

    } catch (error) {
      console.error('[Admin] Firebase initialization error:', error);
      onError?.(error as Error);
    }
  };

  useEffect(() => {
    initializeFirebase();
  }, [scriptsLoaded]);

  const [appScriptLoaded, setAppScriptLoaded] = useState(false);
  const [messagingScriptLoaded, setMessagingScriptLoaded] = useState(false);

  const handleAppScriptLoaded = () => {
    console.log('[Admin] Firebase app script loaded');
    setAppScriptLoaded(true);
  };

  const handleMessagingScriptLoaded = () => {
    console.log('[Admin] Firebase messaging script loaded');
    setMessagingScriptLoaded(true);
  };

  const handleScriptError = (error: any) => {
    console.error('[Admin] Firebase script loading error:', error);
    onError?.(new Error('Failed to load Firebase scripts'));
  };

  // Update scriptsLoaded when both scripts are loaded
  useEffect(() => {
    if (appScriptLoaded && messagingScriptLoaded) {
      console.log('[Admin] All Firebase scripts loaded successfully');
      setScriptsLoaded(true);
    }
  }, [appScriptLoaded, messagingScriptLoaded]);

  return (
    <>
      <Script
        src="https://www.gstatic.com/firebasejs/12.2.1/firebase-app-compat.js"
        strategy="afterInteractive"
        onLoad={handleAppScriptLoaded}
        onError={handleScriptError}
      />
      <Script
        src="https://www.gstatic.com/firebasejs/12.2.1/firebase-messaging-compat.js"
        strategy="afterInteractive"
        onLoad={handleMessagingScriptLoaded}
        onError={handleScriptError}
      />
      <Script
        id="firebase-messaging-sw-registration-admin"
        strategy="afterInteractive"
        dangerouslySetInnerHTML={{
          __html: `
            // Register service worker for Firebase messaging (Admin)
            if ('serviceWorker' in navigator) {
              navigator.serviceWorker.register('/firebase-messaging-sw.js')
                .then((registration) => {
                  console.log('[Admin] Service worker registered:', registration);
                })
                .catch((error) => {
                  console.error('[Admin] Service worker registration failed:', error);
                });
            }
          `
        }}
      />
      <Script
        id="firebase-messaging-sw-config-admin"
        strategy="afterInteractive"
        dangerouslySetInnerHTML={{
          __html: `
            // Inject Firebase config into service worker global scope (Admin)
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