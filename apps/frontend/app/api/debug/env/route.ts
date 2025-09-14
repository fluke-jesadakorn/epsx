import { NextResponse } from 'next/server';

export async function GET() {
  // Always allow debug endpoint for this investigation
  const isDebugEnabled = true;
  
  if (!isDebugEnabled) {
    return NextResponse.json({ error: 'Debug endpoint disabled' }, { status: 403 });
  }

  const debugInfo = {
    nodeEnv: process.env.NODE_ENV,
    hasProcessEnv: typeof process !== 'undefined',
    
    // Check Next.js public environment variables
    nextPublicVars: Object.keys(process.env || {})
      .filter(key => key.startsWith('NEXT_PUBLIC_'))
      .reduce((acc, key) => {
        acc[key] = process.env[key] || 'undefined';
        return acc;
      }, {} as Record<string, string>),
    
    // Check specific Firebase variables
    firebaseConfig: {
      NEXT_PUBLIC_FIREBASE_API_KEY: process.env.NEXT_PUBLIC_FIREBASE_API_KEY ? 
        `${process.env.NEXT_PUBLIC_FIREBASE_API_KEY.substring(0, 10)}...` : 'undefined',
      NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN || 'undefined',
      NEXT_PUBLIC_FIREBASE_PROJECT_ID: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID || 'undefined',
      NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET || 'undefined',
      NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID || 'undefined',
      NEXT_PUBLIC_FIREBASE_APP_ID: process.env.NEXT_PUBLIC_FIREBASE_APP_ID ? 
        `${process.env.NEXT_PUBLIC_FIREBASE_APP_ID.substring(0, 15)}...` : 'undefined',
      NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID || 'undefined',
    },
    
    // Check other important variables
    otherVars: {
      NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL || 'undefined',
      NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL || 'undefined',
      NEXT_PUBLIC_ADMIN_URL: process.env.NEXT_PUBLIC_ADMIN_URL || 'undefined',
      NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'undefined',
    },
    
    // Runtime information
    runtime: {
      isServer: typeof window === 'undefined',
      isBrowser: typeof window !== 'undefined',
      userAgent: process.env.USER_AGENT || 'undefined'
    }
  };

  return NextResponse.json(debugInfo, {
    headers: {
      'Cache-Control': 'no-cache, no-store, must-revalidate',
      'Pragma': 'no-cache',
      'Expires': '0'
    }
  });
}