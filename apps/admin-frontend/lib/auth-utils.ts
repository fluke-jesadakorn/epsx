import { jwtVerify, SignJWT, JWTPayload } from 'jose';
import { env } from '@/config/env';

export interface JWTUser {
  uid: string;
  email: string;
  firebaseUid?: string;
  permissions: string[]; // Structured permissions only
  package_tier: string;
  iat?: number;
  exp?: number;
}

export interface EPSXJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  permissions: string[];  // Structured permissions: "platform:resource:action"
  package_tier: string;
  firebase_uid: string;
  platforms?: string[];         // Accessible platforms
  primary_platform?: string;    // Default platform
  platform_context?: string;    // Current platform context
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
    const jwtSecret = secret || env.NEXTAUTH_SECRET;
    const { payload } = await jwtVerify(token, new TextEncoder().encode(jwtSecret));
    return payload as EPSXJWTPayload;
  } catch {
    return null;
  }
}

/**
 * Sign JWT token with payload
 */
export async function signJWT(payload: EPSXJWTPayload, secret?: string, expiresIn = '24h'): Promise<string> {
  const jwtSecret = secret || env.NEXTAUTH_SECRET;
  const jwt = new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(expiresIn);
  
  return await jwt.sign(new TextEncoder().encode(jwtSecret));
}

/**
 * Create JWT claims for EPSX user - permission-only system
 */
export function createJWTClaims(user: {
  id: string;
  email: string;
  name: string;
  permissions: string[];        // Required: structured permissions
  package_tier?: string;
  firebase_uid?: string;
  platforms?: string[];
  primary_platform?: string;
  platform_context?: string;
}): EPSXJWTPayload {
  const now = Math.floor(Date.now() / 1000);
  const exp = now + (24 * 60 * 60); // 24 hours from now
  
  return {
    sub: user.id,
    email: user.email,
    name: user.name,
    permissions: user.permissions,
    package_tier: user.package_tier || 'FREE',
    firebase_uid: user.firebase_uid || user.id,
    platforms: user.platforms || ['epsx', 'admin'],
    primary_platform: user.primary_platform || 'admin',
    platform_context: user.platform_context,
    iat: now,
    exp: exp,
  };
}

/**
 * Decode JWT token without verification
 */
export function decodeJWT(token: string): JWTUser | null {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    
    // Convert to permission-only format
    return {
      uid: payload.sub || payload.uid,
      email: payload.email,
      firebaseUid: payload.firebase_uid || payload.firebaseUid,
      permissions: payload.permissions || [],
      package_tier: payload.package_tier || 'FREE',
      iat: payload.iat,
      exp: payload.exp,
    };
  } catch {
    return null;
  }
}

/**
 * Check if user has permission based on JWT payload
 */
export function hasJWTPermission(payload: EPSXJWTPayload | JWTUser, permission: string): boolean {
  if (!payload.permissions) return false;
  
  const parsePermission = (permissionString: string): { platform: string, resource: string, action: string } | null => {
    const parts = permissionString.split(':');
    if (parts.length !== 3) return null;
    return { platform: parts[0], resource: parts[1], action: parts[2] };
  };
  
  const checkPermissionAccess = (userPermissions: string[], requiredPermission: string): boolean => {
    const required = parsePermission(requiredPermission);
    if (!required) return false;
    
    for (const permStr of userPermissions) {
      const userPerm = parsePermission(permStr);
      if (!userPerm) continue;
      
      // Check for exact match
      if (userPerm.platform === required.platform && 
          userPerm.resource === required.resource && 
          userPerm.action === required.action) {
        return true;
      }
      
      // Check for wildcard matches
      if (userPerm.platform === required.platform) {
        if (userPerm.resource === '*' && userPerm.action === '*') {
          return true;
        }
        if (userPerm.resource === required.resource && userPerm.action === '*') {
          return true;
        }
      }
      
      // Global admin permission: "admin:*:*"
      if (userPerm.platform === 'admin' && userPerm.resource === '*' && userPerm.action === '*') {
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
 * Check if user can access platform based on JWT payload
 */
export function canJWTAccessPlatform(payload: EPSXJWTPayload, platform: string): boolean {
  // Check if platform is in accessible platforms
  if (payload.platforms?.includes(platform)) {
    return true;
  }
  
  // Check if user has any permissions for this platform
  return payload.permissions.some(perm => perm.startsWith(`${platform}:`));
}

/**
 * Create permission-based user claims for backward compatibility
 */
export function createPermissionBasedClaims(legacyUser: {
  id: string;
  email: string;
  name: string;
  role?: string; // Legacy role field
  package_tier?: string;
  firebase_uid?: string;
}): EPSXJWTPayload {
  // Convert legacy role to permissions
  let permissions: string[] = [];
  
  switch (legacyUser.role?.toLowerCase()) {
    case 'admin':
      permissions = ['admin:*:*'];
      break;
    case 'user':
    case 'premium':
      permissions = [
        'epsx:analytics:view',
        'epsx:analytics:export',
        'epsx:analytics:advanced',
        'epsx:realtime:access',
        'epsx:profile:manage',
        'epsx:notifications:receive',
        'epsx:billing:manage'
      ];
      break;
    case 'guest':
    case 'basic':
    default:
      permissions = [
        'epsx:analytics:view',
        'epsx:profile:manage',
        'epsx:notifications:receive'
      ];
      break;
  }
  
  return createJWTClaims({
    ...legacyUser,
    permissions
  });
}