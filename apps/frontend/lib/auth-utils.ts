import { jwtVerify, SignJWT, JWTPayload } from 'jose';

export interface JWTUser {
  uid: string;
  email: string;
  firebaseUid?: string;
  role?: string;
  iat?: number;
  exp?: number;
}

export interface EPSXJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  role: string;
  permissions: string[];
  package_tier: string;
  firebase_uid: string;
  admin_modules?: string[];
  iat: number;
  exp: number;
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
export async function verifyJWT(token: string, secret?: string): Promise<EPSXJWTPayload | null> {
  try {
    const jwtSecret = secret || process.env.NEXTAUTH_SECRET || 'your-default-secret-key';
    const { payload } = await jwtVerify(token, new TextEncoder().encode(jwtSecret));
    return payload as EPSXJWTPayload;
  } catch {
    return null;
  }
}

/**
 * Create JWT claims for EPSX user
 */
export function createJWTClaims(user: {
  id: string;
  email: string;
  name: string;
  role?: string;
  permissions?: string[];
  package_tier?: string;
  firebase_uid?: string;
  admin_modules?: string[];
}): EPSXJWTPayload {
  const now = Math.floor(Date.now() / 1000);
  const exp = now + (24 * 60 * 60); // 24 hours from now
  
  return {
    sub: user.id,
    email: user.email,
    name: user.name,
    role: user.role || 'user',
    permissions: user.permissions || ['user:read'],
    package_tier: user.package_tier || 'FREE',
    firebase_uid: user.firebase_uid || user.id,
    admin_modules: user.admin_modules || [],
    iat: now,
    exp: exp,
  };
}

/**
 * Sign JWT token with payload
 */
export async function signJWT(payload: EPSXJWTPayload, secret?: string, expiresIn = '24h'): Promise<string> {
  const jwtSecret = secret || process.env.NEXTAUTH_SECRET || 'your-default-secret-key';
  const jwt = new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(expiresIn);
  
  return await jwt.sign(new TextEncoder().encode(jwtSecret));
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