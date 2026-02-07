/**
 * Shared JWT Utilities - WEB3-FIRST ARCHITECTURE
 * Provides JWT handling, verification, and payload processing for Web3 sessions
 * Phase 4.2: Updated to use Web3 app secrets instead of NEXTAUTH_SECRET
 */

import { JWTPayload, jwtVerify, SignJWT } from 'jose';

export interface JWTUser {
  uid: string;
  email: string;
  permissions: string[];
  role?: string;
  iat?: number;
  exp?: number;
}

export interface CreateJWTClaimsOptions {
  id: string;
  email: string;
  name?: string;
  permissions: string[];
  role?: string;
}

export interface EPSXJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  permissions: string[];
  iat: number;
  exp: number;
  // Additional fields used by applications
  id?: string;
  role?: string;
  package_tier?: string;
  platforms?: string[];
  primary_platform?: string;
  platform_context?: string;
  iss?: string;
  aud?: string;
}

/**
 * Check if JWT token is expired
 */
export function isJWTExpired(token: string): boolean {
  try {
    const payloadPart = token.split('.')[1];
    if (!payloadPart) {return true;}
    const payload = JSON.parse(atob(payloadPart));
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
    const payloadPart = token.split('.')[1];
    if (!payloadPart) {return 0;}
    const payload = JSON.parse(atob(payloadPart));
    const currentTime = Math.floor(Date.now() / 1000);
    return Math.max(0, payload.exp - currentTime);
  } catch {
    return 0;
  }
}

/**
 * Check if user has permission based on JWT payload
 */
export function hasJWTPermission(
  payload: EPSXJWTPayload | JWTUser,
  permission: string
): boolean {
  if (!payload.permissions) {return false;}

  const parsePermission = (
    permissionString: string
  ): { platform: string; resource: string; action: string } | null => {
    const parts = permissionString.split(':');
    if (parts.length !== 3) {return null;}
    const platform = parts[0];
    const resource = parts[1];
    const action = parts[2];
    if (!platform || !resource || !action) {return null;}
    return { platform, resource, action };
  };

  const checkPermissionAccess = (
    userPermissions: string[],
    requiredPermission: string
  ): boolean => {
    const required = parsePermission(requiredPermission);
    if (!required) {return false;}

    for (const permStr of userPermissions) {
      const userPerm = parsePermission(permStr);
      if (!userPerm) {continue;}

      if (
        userPerm.platform === required.platform &&
        userPerm.resource === required.resource &&
        userPerm.action === required.action
      ) {
        return true;
      }

      if (userPerm.platform === required.platform) {
        if (userPerm.resource === '*' && userPerm.action === '*') {
          return true;
        }
        if (
          userPerm.resource === required.resource &&
          userPerm.action === '*'
        ) {
          return true;
        }
      }

      if (
        userPerm.platform === 'admin' &&
        userPerm.resource === '*' &&
        userPerm.action === '*'
      ) {
        return true;
      }
    }

    return false;
  };

  return checkPermissionAccess(payload.permissions, permission);
}

/**
 * Check if user is admin based on JWT payload
 */
export function isJWTAdmin(payload: EPSXJWTPayload | JWTUser): boolean {
  return hasJWTPermission(payload, 'admin:*:*');
}

/**
 * Get user permissions from JWT payload
 */
export function getJWTPermissions(payload: EPSXJWTPayload | JWTUser): string[] {
  return payload.permissions || [];
}

/**
 * Decode JWT token without verification
 */
export function decodeJWT(token: string): JWTUser | null {
  try {
    const payloadPart = token.split('.')[1];
    if (!payloadPart) {return null;}
    const payload = JSON.parse(atob(payloadPart));

    return {
      uid: payload.sub || payload.uid,
      email: payload.email,
      permissions: payload.permissions || [],
      iat: payload.iat,
      exp: payload.exp,
    };
  } catch {
    return null;
  }
}

/**
 * Create JWT claims object for signing
 */
export function createJWTClaims(
  options: CreateJWTClaimsOptions
): EPSXJWTPayload {
  const now = Math.floor(Date.now() / 1000);
  const expiry = now + 24 * 60 * 60; // 24 hours

  return {
    sub: options.id,
    email: options.email,
    name: options.name || options.email.split('@')[0] || 'user',
    permissions: options.permissions || [],
    iat: now,
    exp: expiry,
  };
}

/**
 * Sign JWT token with Web3 app secret
 * Web3-first authentication system
 */
export async function signJWT(payload: EPSXJWTPayload): Promise<string> {
  const secret =
    process.env.WEB3_APP_SECRET ||
    'web3-default-secret-for-development-only-change-in-production';

  const encoder = new TextEncoder();
  const key = encoder.encode(secret);

  return new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(payload.exp)
    .sign(key);
}

/**
 * Verify and decode JWT token with Web3 app secret
 * Web3-first authentication system
 */
export async function verifyJWT(token: string): Promise<EPSXJWTPayload | null> {
  try {
    const secret =
      process.env.WEB3_APP_SECRET ||
      'web3-default-secret-for-development-only-change-in-production';

    const encoder = new TextEncoder();
    const key = encoder.encode(secret);

    const { payload } = await jwtVerify(token, key, {
      algorithms: ['HS256'],
    });

    return payload as EPSXJWTPayload;
  } catch {
    return null;
  }
}

/**
 * Decode EPSX JWT payload without verification
 * Used when verification is handled by upstream/backend or we trust the source (HttpOnly cookie)
 * Useful when we have RS256 tokens but no public key on the frontend
 */
export function decodeEPSXJWT(token: string): EPSXJWTPayload | null {
  try {
    const payloadPart = token.split('.')[1];
    if (!payloadPart) {return null;}
    const payload = JSON.parse(atob(payloadPart));
    return payload as EPSXJWTPayload;
  } catch {
    return null;
  }
}