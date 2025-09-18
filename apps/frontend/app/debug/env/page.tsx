'use client';

import { useEffect, useState } from 'react';
import { env } from '../../../../../shared/env/schema';

export default function EnvDebugPage() {
  const [debugInfo, setDebugInfo] = useState<any>(null);
  const [serverDebugInfo, setServerDebugInfo] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Client-side environment debug
    const clientDebugInfo = {
      hasWindow: typeof window !== 'undefined',
      hasProcess: typeof process !== 'undefined',
      
      // Raw process.env access
      processEnv: {
        hasProcessEnv: typeof process !== 'undefined' && !!process.env,
        nodeEnv: typeof process !== 'undefined' ? process.env?.NODE_ENV : 'undefined',
        nextPhase: typeof process !== 'undefined' ? process.env?.NEXT_PHASE : 'undefined',
      },
      
      // Next.js public variables (should be embedded in client bundle)
      nextPublicVars: typeof process !== 'undefined' ? 
        Object.keys(process.env || {})
          .filter(key => key.startsWith('NEXT_PUBLIC_'))
          .reduce((acc, key) => {
            acc[key] = process.env![key] || 'undefined';
            return acc;
          }, {} as Record<string, string>) : {},
      
      // Test unified schema access
      unifiedSchemaAccess: {
        canAccessFirebaseApiKey: (() => {
          try {
            const value = env.FIREBASE_API_KEY;
            return value ? `${value.substring(0, 10)}...` : 'undefined';
          } catch (error) {
            return `Error: ${error}`;
          }
        })(),
        canAccessFirebaseProjectId: (() => {
          try {
            return env.FIREBASE_PROJECT_ID || 'undefined';
          } catch (error) {
            return `Error: ${error}`;
          }
        })(),
        canAccessBackendUrl: (() => {
          try {
            return env.BACKEND_URL || 'undefined';
          } catch (error) {
            return `Error: ${error}`;
          }
        })(),
      },
      
      // Firebase config object test
      firebaseConfigTest: (() => {
        try {
          const firebaseConfig = {
            apiKey: env.FIREBASE_API_KEY,
            authDomain: env.FIREBASE_AUTH_DOMAIN,
            projectId: env.FIREBASE_PROJECT_ID,
            storageBucket: env.FIREBASE_STORAGE_BUCKET,
            messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID,
            appId: env.FIREBASE_APP_ID,
            measurementId: env.FIREBASE_MEASUREMENT_ID
          };
          return {
            apiKey: firebaseConfig.apiKey ? `${firebaseConfig.apiKey.substring(0, 10)}...` : 'undefined',
            authDomain: firebaseConfig.authDomain || 'undefined',
            projectId: firebaseConfig.projectId || 'undefined',
            storageBucket: firebaseConfig.storageBucket || 'undefined',
            messagingSenderId: firebaseConfig.messagingSenderId || 'undefined',
            appId: firebaseConfig.appId ? `${firebaseConfig.appId.substring(0, 15)}...` : 'undefined',
            measurementId: firebaseConfig.measurementId || 'undefined',
          };
        } catch (error) {
          return `Error: ${error}`;
        }
      })(),
    };

    setDebugInfo(clientDebugInfo);

    // Fetch server-side debug info
    fetch('/api/debug/env')
      .then(res => res.json())
      .then(data => {
        setServerDebugInfo(data);
      })
      .catch(error => {
        setServerDebugInfo({ error: error.message });
      })
      .finally(() => {
        setLoading(false);
      });

    // Console logging for easy browser console access
    console.log('🔍 Client Environment Debug Info:', clientDebugInfo);
    
    // Make debug functions available globally
    (window as any).debugEnv = {
      checkFirebaseConfig: () => {
        console.log('Firebase Config:', {
          apiKey: env.FIREBASE_API_KEY,
          authDomain: env.FIREBASE_AUTH_DOMAIN,
          projectId: env.FIREBASE_PROJECT_ID,
          storageBucket: env.FIREBASE_STORAGE_BUCKET,
          messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID,
          appId: env.FIREBASE_APP_ID,
          measurementId: env.FIREBASE_MEASUREMENT_ID
        });
      },
      checkProcessEnv: () => {
        if (typeof process !== 'undefined' && process.env) {
          const nextPublicVars = Object.keys(process.env)
            .filter(key => key.startsWith('NEXT_PUBLIC_'))
            .reduce((acc, key) => {
              acc[key] = process.env[key] || 'undefined';
              return acc;
            }, {} as Record<string, string>);
          console.log('NEXT_PUBLIC_ variables:', nextPublicVars);
        } else {
          console.log('process.env is not available');
        }
      }
    };

  }, []);

  if (loading) {
    return <div className="p-8">Loading debug information...</div>;
  }

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Environment Variables Debug</h1>
      
      <div className="space-y-8">
        <section>
          <h2 className="text-xl font-semibold mb-4">Client-Side Environment</h2>
          <pre className="bg-gray-100 p-4 rounded overflow-auto text-sm">
            {JSON.stringify(debugInfo, null, 2)}
          </pre>
        </section>

        <section>
          <h2 className="text-xl font-semibold mb-4">Server-Side Environment</h2>
          <pre className="bg-gray-100 p-4 rounded overflow-auto text-sm">
            {JSON.stringify(serverDebugInfo, null, 2)}
          </pre>
        </section>

        <section>
          <h2 className="text-xl font-semibold mb-4">Browser Console Commands</h2>
          <div className="bg-blue-50 p-4 rounded">
            <p className="mb-2">Open browser console and run:</p>
            <ul className="list-disc list-inside space-y-1 text-sm font-mono">
              <li>window.debugEnv.checkFirebaseConfig()</li>
              <li>window.debugEnv.checkProcessEnv()</li>
            </ul>
          </div>
        </section>
      </div>
    </div>
  );
}