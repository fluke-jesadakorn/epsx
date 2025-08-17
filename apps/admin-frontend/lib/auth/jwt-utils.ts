import { SignJWT, jwtVerify, type JWTPayload } from 'jose'

/**
 * Extended JWT payload with custom claims for EPSX
 */
export interface EPSXJWTPayload extends JWTPayload {
  sub: string // User ID
  email: string
  name?: string
  admin_modules: string[]
  permissions: string[]
  package_tier: string
  role: string
  firebase_uid?: string
}

/**
 * JWT secret key for signing and verification
 */
const getSecretKey = () => {
  const secret = process.env.JWT_SECRET || process.env.NEXTAUTH_SECRET
  if (!secret) {
    throw new Error('JWT_SECRET or NEXTAUTH_SECRET environment variable is required')
  }
  return new TextEncoder().encode(secret)
}

/**
 * Sign a JWT token with custom claims
 */
export async function signJWT(payload: Omit<EPSXJWTPayload, 'iat' | 'exp'>): Promise<string> {
  const secret = getSecretKey()
  
  return await new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime('2h')
    .sign(secret)
}

/**
 * Verify and decode a JWT token
 */
export async function verifyJWT(token: string): Promise<EPSXJWTPayload> {
  const secret = getSecretKey()
  
  try {
    const { payload } = await jwtVerify(token, secret)
    return payload as EPSXJWTPayload
  } catch (error) {
    throw new Error(`Invalid JWT token: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Decode JWT without verification (for client-side inspection)
 * DO NOT use this for security-critical operations
 */
export function decodeJWT(token: string): EPSXJWTPayload | null {
  try {
    const parts = token.split('.')
    if (parts.length !== 3) return null
    
    const payload = JSON.parse(atob(parts[1]))
    return payload as EPSXJWTPayload
  } catch {
    return null
  }
}

/**
 * Extract permissions from JWT token (client-side safe)
 */
export function getPermissionsFromJWT(token: string): string[] {
  const payload = decodeJWT(token)
  return payload?.permissions || []
}

/**
 * Check if JWT token has specific permission
 */
export function hasPermissionInJWT(token: string, permission: string): boolean {
  const permissions = getPermissionsFromJWT(token)
  
  // Check exact match
  if (permissions.includes(permission)) {
    return true
  }
  
  // Check wildcard permissions
  return permissions.some(userPermission => {
    if (userPermission.endsWith('.*') || userPermission.endsWith(':*')) {
      const prefix = userPermission.slice(0, -2)
      return permission.startsWith(prefix + '.') || permission.startsWith(prefix + ':')
    }
    if (userPermission === '*') {
      return true
    }
    return false
  })
}

/**
 * Extract admin modules from JWT token
 */
export function getAdminModulesFromJWT(token: string): string[] {
  const payload = decodeJWT(token)
  return payload?.admin_modules || []
}

/**
 * Check if JWT token has specific admin module
 */
export function hasAdminModuleInJWT(token: string, module: string): boolean {
  const adminModules = getAdminModulesFromJWT(token)
  return adminModules.includes(module)
}

/**
 * Extract package tier from JWT token
 */
export function getPackageTierFromJWT(token: string): string {
  const payload = decodeJWT(token)
  return payload?.package_tier || 'FREE'
}

/**
 * Check if JWT token has package tier or higher
 */
export function hasPackageTierInJWT(token: string, requiredTier: string): boolean {
  const userTier = getPackageTierFromJWT(token)
  
  const tierHierarchy: Record<string, number> = {
    'FREE': 1,
    'BRONZE': 2,
    'SILVER': 3,
    'GOLD': 4,
    'PLATINUM': 5,
    'ENTERPRISE': 6
  }
  
  const userLevel = tierHierarchy[userTier] || 0
  const requiredLevel = tierHierarchy[requiredTier] || 1
  
  return userLevel >= requiredLevel
}

/**
 * Check if JWT token is expired
 */
export function isJWTExpired(token: string): boolean {
  const payload = decodeJWT(token)
  if (!payload?.exp) return true
  
  const now = Math.floor(Date.now() / 1000)
  return payload.exp < now
}

/**
 * Get time until JWT expires (in seconds)
 */
export function getJWTTimeToExpiry(token: string): number {
  const payload = decodeJWT(token)
  if (!payload?.exp) return 0
  
  const now = Math.floor(Date.now() / 1000)
  return Math.max(0, payload.exp - now)
}

/**
 * Create JWT claims from user data
 */
export function createJWTClaims(user: {
  id: string
  email: string
  name?: string
  admin_modules?: string[]
  permissions?: string[]
  package_tier?: string
  role?: string
  firebase_uid?: string
}): Omit<EPSXJWTPayload, 'iat' | 'exp'> {
  return {
    sub: user.id,
    email: user.email,
    name: user.name,
    admin_modules: user.admin_modules || [],
    permissions: user.permissions || ['user:read'],
    package_tier: user.package_tier || 'FREE',
    role: user.role || 'user',
    firebase_uid: user.firebase_uid || user.id,
  }
}