import { cookies } from 'next/headers';

const SESSION_KEY = '__session';

export interface EdgeSessionClaims {
  uid: string;
  email: string;
  email_verified: boolean;
  role?: string;
  iat?: number;
  exp?: number;
}

// Simple JWT decoder for Edge Runtime (without verification)
// Note: This is for basic parsing only. Full verification should be done server-side
function parseJWT(token: string): EdgeSessionClaims | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;
    
    const payload = JSON.parse(atob(parts[1]));
    
    // Basic expiration check
    if (payload.exp && Date.now() >= payload.exp * 1000) {
      return null;
    }
    
    return {
      uid: payload.sub || payload.uid,
      email: payload.email,
      email_verified: payload.email_verified || false,
      role: payload.role,
      iat: payload.iat,
      exp: payload.exp,
    };
  } catch (error) {
    console.error('Failed to parse JWT:', error);
    return null;
  }
}

export async function verifySessionEdge(): Promise<EdgeSessionClaims | null> {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get(SESSION_KEY)?.value || null;
    
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
