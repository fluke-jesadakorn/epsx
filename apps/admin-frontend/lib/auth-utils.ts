import { jwtVerify, SignJWT, JWTPayload } from 'jose';
import { env } from '@/config/env';

export interface JWTUser {
  uid: string;
  email: string;
  permissions: string[]; // Structured permissions only
  iat?: number;
  exp?: number;
  // firebaseUid, package_tier removed - derived from permissions
}

export interface CreateJWTClaimsOptions {
  id: string;
  email: string;
  name?: string;
  permissions: string[];
  role?: string;
  package_tier?: string;
}

export interface EPSXJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  permissions: string[];  // Structured permissions: "platform:resource:action"
  iat: number;
  exp: number;
  // package_tier, firebase_uid, platforms, primary_platform removed - derived from permissions
}

/**
 * Derive package tier from permissions (matches backend logic)
 */
export function derivePackageTierFromPermissions(permissions: string[]): string {
  if (hasEnterprisePermissions(permissions)) {
    return 'ENTERPRISE';
  } else if (hasPlatinumPermissions(permissions)) {
    return 'PLATINUM';
  } else if (hasGoldPermissions(permissions)) {
    return 'GOLD';
  } else if (hasSilverPermissions(permissions)) {
    return 'SILVER';
  } else if (hasBronzePermissions(permissions)) {
    return 'BRONZE';
  } else {
    return 'FREE';
  }
}

/**
 * Derive accessible platforms from permissions
 */
export function deriveAccessiblePlatformsFromPermissions(permissions: string[]): string[] {
  const platforms = new Set<string>();
  
  for (const permission of permissions) {
    const platform = permission.split(':')[0];
    if (platform) {
      platforms.add(platform);
    }
  }
  
  return platforms.size > 0 ? Array.from(platforms) : ['epsx'];
}

/**
 * Derive primary platform from permissions (priority: admin > epsx > epsx-pay > epsx-token)
 */
export function derivePrimaryPlatformFromPermissions(permissions: string[]): string {
  if (permissions.some(p => p.startsWith('admin:'))) {
    return 'admin';
  } else if (permissions.some(p => p.startsWith('epsx:'))) {
    return 'epsx';
  } else if (permissions.some(p => p.startsWith('epsx-pay:'))) {
    return 'epsx-pay';
  } else if (permissions.some(p => p.startsWith('epsx-token:'))) {
    return 'epsx-token';
  } else {
    return 'epsx';
  }
}

// Helper functions for tier detection
function hasEnterprisePermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('enterprise:') || 
    p === 'admin:*:*' ||
    p.includes('enterprise') ||
    permissions.some(perm => perm.startsWith('admin:'))
  );
}

function hasPlatinumPermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('platinum:') ||
    p.includes('platinum') ||
    permissions.length >= 10 // Many permissions indicate higher tier
  );
}

function hasGoldPermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('gold:') ||
    p.includes('gold') ||
    permissions.some(perm => perm.includes('export') || perm.includes('advanced'))
  );
}

function hasSilverPermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('silver:') ||
    p.includes('silver') ||
    permissions.length >= 5 // Several permissions indicate silver tier
  );
}

function hasBronzePermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('bronze:') ||
    p.includes('bronze') ||
    permissions.length >= 3 // Few permissions indicate bronze tier
  );
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
  permissions: string[];
} | CreateJWTClaimsOptions): EPSXJWTPayload {
  const now = Math.floor(Date.now() / 1000);
  const exp = now + (24 * 60 * 60);
  
  return {
    sub: user.id,
    email: user.email,
    name: user.name || user.email.split('@')[0],
    permissions: user.permissions || [],
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
export function canJWTAccessPlatform(payload: EPSXJWTPayload | JWTUser, platform: string): boolean {
  const accessiblePlatforms = deriveAccessiblePlatformsFromPermissions(payload.permissions);
  return accessiblePlatforms.includes(platform);
}

/**
 * Create permission-based user claims for backward compatibility
 */
export function createPermissionBasedClaims(legacyUser: {
  id: string;
  email: string;
  name: string;
  role?: string;
}): EPSXJWTPayload {
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
    id: legacyUser.id,
    email: legacyUser.email,
    name: legacyUser.name,
    permissions,
  });
}