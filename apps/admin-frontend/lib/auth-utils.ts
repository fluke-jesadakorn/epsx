import { jwtVerify, SignJWT, JWTPayload } from 'jose';

export interface JWTUser {
  uid: string;
  email: string;
  firebaseUid?: string;
  role?: string;
  iat?: number;
  exp?: number;
}

/**
 * Check if JWT token is expired
 */
export function isJWTExpired(token: string): boolean {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    const currentTime = Math.floor(Date.now() / 1000);
    return payload.exp < currentTime;
  } catch {
    return true;
  }
}

/**
 * Get time to expiry for JWT token in seconds
 */
export function getJWTTimeToExpiry(token: string): number {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    const currentTime = Math.floor(Date.now() / 1000);
    return Math.max(0, payload.exp - currentTime);
  } catch {
    return 0;
  }
}

/**
 * Verify JWT token with secret
 */
export async function verifyJWT(token: string, secret: string): Promise<JWTUser | null> {
  try {
    const { payload } = await jwtVerify(token, new TextEncoder().encode(secret));
    return payload as JWTUser;
  } catch {
    return null;
  }
}

/**
 * Sign JWT token with payload
 */
export async function signJWT(payload: JWTPayload, secret: string, expiresIn = '7d'): Promise<string> {
  const jwt = new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(expiresIn);
  
  return await jwt.sign(new TextEncoder().encode(secret));
}

/**
 * Decode JWT token without verification
 */
export function decodeJWT(token: string): JWTUser | null {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return payload as JWTUser;
  } catch {
    return null;
  }
}