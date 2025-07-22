import { cookies } from 'next/headers';
import type { SessionClaims, SessionConfig, SessionResult } from './types';

// Default session configuration
export const DEFAULT_SESSION_CONFIG: SessionConfig = {
  sessionKey: '__session',
  maxAge: 60 * 60 * 24 * 5, // 5 days in seconds
  refreshThreshold: 60 * 60, // 1 hour before expiry
  secure: process.env.NODE_ENV === 'production',
  sameSite: 'lax',
  httpOnly: true,
};

/**
 * Get Firebase Admin Auth instance
 * Initialize Firebase Admin if not already done
 */
async function getAuthAdmin() {
  try {
    const admin = await import('firebase-admin');
    
    // Check if Firebase Admin is already initialized
    if (admin.default.apps.length === 0) {
      // Initialize Firebase Admin with service account
      const serviceAccount = {
        type: process.env.FIREBASE_TYPE,
        project_id: process.env.FIREBASE_PROJECT_ID,
        private_key_id: process.env.FIREBASE_PRIVATE_KEY_ID,
        private_key: process.env.FIREBASE_PRIVATE_KEY ? process.env.FIREBASE_PRIVATE_KEY.replace(/\\n/g, '\n') : '',
        client_email: process.env.FIREBASE_CLIENT_EMAIL,
        client_id: process.env.FIREBASE_CLIENT_ID,
        auth_uri: process.env.FIREBASE_AUTH_URI,
        token_uri: process.env.FIREBASE_TOKEN_URI,
        auth_provider_x509_cert_url: process.env.FIREBASE_AUTH_PROVIDER_CERT_URL,
        client_x509_cert_url: process.env.FIREBASE_CLIENT_CERT_URL,
        universe_domain: process.env.FIREBASE_UNIVERSE_DOMAIN,
      };

      admin.default.initializeApp({
        credential: admin.default.credential.cert(serviceAccount as admin.ServiceAccount),
        databaseURL: `https://${process.env.FIREBASE_PROJECT_ID}.firebaseio.com`,
      });
    }

    return admin.default.auth();
  } catch (error) {
    console.error('Failed to initialize Firebase Admin Auth:', error);
    throw new Error(`Firebase Admin initialization failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Create a new session with the provided Firebase ID token
 */
export async function createSession(
  token: string,
  config: Partial<SessionConfig> = {},
): Promise<{ success: boolean; error?: string }> {
  try {
    const sessionConfig = { ...DEFAULT_SESSION_CONFIG, ...config };

    // Verify the token first
    const auth = await getAuthAdmin();
    await auth.verifyIdToken(token, true); // checkRevoked = true

    const cookieStore = await cookies();

    // Set the session cookie with explicit configuration
    cookieStore.set(sessionConfig.sessionKey, token, {
      maxAge: sessionConfig.maxAge,
      httpOnly: sessionConfig.httpOnly,
      secure: sessionConfig.secure,
      sameSite: sessionConfig.sameSite,
      path: '/',
    });

    console.log('Session cookie set with token length:', token.length);
    return { success: true };
  } catch (error) {
    console.error('Failed to create session:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

/**
 * Verify the current session and return claims
 */
export async function verifySession(
  config: Partial<SessionConfig> = {},
): Promise<SessionResult> {
  try {
    const sessionConfig = { ...DEFAULT_SESSION_CONFIG, ...config };
    const cookieStore = await cookies();
    const token = cookieStore.get(sessionConfig.sessionKey)?.value;

    if (!token) {
      return { success: false };
    }

    // Verify the Firebase ID token
    const auth = await getAuthAdmin();
    const decodedToken = await auth.verifyIdToken(token, true); // checkRevoked = true

    const claims: SessionClaims = {
      uid: decodedToken.uid,
      ...(decodedToken.email && { email: decodedToken.email }),
      ...(decodedToken.email_verified !== undefined && {
        email_verified: decodedToken.email_verified,
      }),
      ...(decodedToken.name && { name: decodedToken.name }),
      ...(decodedToken.picture && { picture: decodedToken.picture }),
      ...(decodedToken.exp && { exp: decodedToken.exp }),
      ...(decodedToken.iat && { iat: decodedToken.iat }),
      ...(decodedToken.role && { role: decodedToken.role }),
      ...(decodedToken.permissions && {
        permissions: decodedToken.permissions,
      }),
      custom_claims: decodedToken,
    };

    // Check if token needs refresh (within threshold of expiry)
    const needsRefresh = decodedToken.exp
      ? decodedToken.exp - Math.floor(Date.now() / 1000) <
        sessionConfig.refreshThreshold
      : false;

    return {
      success: true,
      claims,
      needsRefresh,
    };
  } catch (error) {
    console.error('Session verification failed:', error);

    // Clear invalid session cookie
    try {
      await destroySession(config);
    } catch (destroyError) {
      console.error('Failed to clear invalid session:', destroyError);
    }

    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

/**
 * Refresh the current session with a new token
 */
export async function refreshSession(
  newToken: string,
  config: Partial<SessionConfig> = {},
): Promise<{ success: boolean; error?: string }> {
  try {
    // Verify the new token before updating
    const auth = await getAuthAdmin();
    await auth.verifyIdToken(newToken, true);

    // Update the session cookie
    return await createSession(newToken, config);
  } catch (error) {
    console.error('Session refresh failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

/**
 * Destroy the current session
 */
export async function destroySession(
  config: Partial<SessionConfig> = {},
): Promise<void> {
  const sessionConfig = { ...DEFAULT_SESSION_CONFIG, ...config };
  const cookieStore = await cookies();
  console.log('Destroying session cookie');
  cookieStore.delete(sessionConfig.sessionKey);
}

/**
 * Check if a session exists (without verification)
 */
export async function hasSession(
  config: Partial<SessionConfig> = {},
): Promise<boolean> {
  const sessionConfig = { ...DEFAULT_SESSION_CONFIG, ...config };
  const cookieStore = await cookies();
  return !!cookieStore.get(sessionConfig.sessionKey)?.value;
}

/**
 * Get session info for client-side use (minimal data)
 */
export async function getSessionInfo(
  config: Partial<SessionConfig> = {},
): Promise<{
  isAuthenticated: boolean;
  email?: string;
  emailVerified?: boolean;
  displayName?: string;
}> {
  const result = await verifySession(config);

  if (!result.success || !result.claims) {
    return { isAuthenticated: false };
  }

  return {
    isAuthenticated: true,
    ...(result.claims.email && { email: result.claims.email }),
    ...(result.claims.email_verified !== undefined && {
      emailVerified: result.claims.email_verified,
    }),
    ...(result.claims.name && { displayName: result.claims.name }),
  };
}

/**
 * Edge runtime compatible session verification (basic parsing without verification)
 */
export async function verifySessionEdge(
  config: Partial<SessionConfig> = {},
): Promise<SessionClaims | null> {
  try {
    const sessionConfig = { ...DEFAULT_SESSION_CONFIG, ...config };
    const cookieStore = await cookies();
    const token = cookieStore.get(sessionConfig.sessionKey)?.value || null;

    if (!token) {
      return null;
    }

    // Parse the token (basic parsing without cryptographic verification)
    // Full verification should be done in API routes using Firebase Admin SDK
    const claims = parseJWT(token);

    return claims;
  } catch (error) {
    console.error('Edge session verification failed:', error);
    return null;
  }
}

/**
 * Simple JWT decoder for Edge Runtime (without verification)
 * Note: This is for basic parsing only. Full verification should be done server-side
 */
function parseJWT(token: string): SessionClaims | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;

    const payload = JSON.parse(
      Buffer.from(
        parts[1]?.replace(/-/g, '+').replace(/_/g, '/') || '',
        'base64',
      ).toString(),
    );

    return {
      uid: payload.user_id || payload.sub,
      ...(payload.email && { email: payload.email }),
      ...(payload.email_verified !== undefined && {
        email_verified: payload.email_verified,
      }),
      ...(payload.name && { name: payload.name }),
      ...(payload.picture && { picture: payload.picture }),
      ...(payload.exp && { exp: payload.exp }),
      ...(payload.iat && { iat: payload.iat }),
      ...(payload.role && { role: payload.role }),
      ...(payload.permissions && { permissions: payload.permissions }),
      custom_claims: payload,
    };
  } catch (error) {
    console.error('Failed to parse JWT:', error);
    return null;
  }
}
