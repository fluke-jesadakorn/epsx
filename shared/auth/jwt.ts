/**
 * Shared JWT Utilities
 * Provides JWT handling, verification, and payload processing
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
  firebase_uid?: string;
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
 * Check if user has permission based on JWT payload
 */
export function hasJWTPermission(
  payload: EPSXJWTPayload | JWTUser,
  permission: string
): boolean {
  if (!payload.permissions) return false;

  const parsePermission = (
    permissionString: string
  ): { platform: string; resource: string; action: string } | null => {
    const parts = permissionString.split(':');
    if (parts.length !== 3) return null;
    return { platform: parts[0], resource: parts[1], action: parts[2] };
  };

  const checkPermissionAccess = (
    userPermissions: string[],
    requiredPermission: string
  ): boolean => {
    const required = parsePermission(requiredPermission);
    if (!required) return false;

    for (const permStr of userPermissions) {
      const userPerm = parsePermission(permStr);
      if (!userPerm) continue;

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
    const payload = JSON.parse(atob(token.split('.')[1]));

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
    name: options.name || options.email.split('@')[0],
    permissions: options.permissions || [],
    iat: now,
    exp: expiry,
  };
}

/**
 * Sign JWT token with secret
 */
export async function signJWT(payload: EPSXJWTPayload): Promise<string> {
  const secret =
    process.env.NEXTAUTH_SECRET || 'your-secret-key-change-in-production';
  const encoder = new TextEncoder();
  const key = encoder.encode(secret);

  return new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(payload.exp)
    .sign(key);
}

/**
 * Verify and decode JWT token
 */
export async function verifyJWT(token: string): Promise<EPSXJWTPayload | null> {
  try {
    const secret =
      process.env.NEXTAUTH_SECRET || 'your-secret-key-change-in-production';
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