import { UserRole, TokenFeature } from './roles.enum';
import { Permission } from '../permissions';

export interface TokenClaims {
  // Firebase auth claims
  uid: string;
  email?: string;
  email_verified?: boolean;

  // Custom EPSx claims
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
  
  // Standard JWT claims
  iss?: string;  // Issuer
  sub?: string;  // Subject
  aud?: string;  // Audience
  exp?: number;  // Expiration time
  nbf?: number;  // Not before
  iat?: number;  // Issued at
  jti?: string;  // JWT ID
}

export interface UserTokenData {
  uid: string;
  email: string | null;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
}

export interface TokenMetadata {
  createdAt: number;
  expiresAt: number;
  lastRefreshedAt?: number;
}

export interface TokenResponse {
  token: string;
  user: UserTokenData;
  metadata: TokenMetadata;
}

// Type guard to check if an object is a valid TokenClaims
export function isTokenClaims(obj: any): obj is TokenClaims {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.uid === 'string' &&
    typeof obj.role === 'string' &&
    typeof obj.tokenBalance === 'number' &&
    Array.isArray(obj.features) &&
    Array.isArray(obj.permissions)
  );
}

// Helper to validate that all required claims are present
export function validateTokenClaims(claims: any): TokenClaims {
  if (!isTokenClaims(claims)) {
    throw new Error('Invalid token claims structure');
  }

  // Validate role
  if (!Object.values(UserRole).includes(claims.role)) {
    throw new Error('Invalid role in token claims');
  }

  // Validate features
  if (!claims.features.every((feature: any) => 
    Object.values(TokenFeature).includes(feature)
  )) {
    throw new Error('Invalid features in token claims');
  }

  // Validate token balance
  if (claims.tokenBalance < 0) {
    throw new Error('Invalid token balance in claims');
  }

  return claims;
}

// Helper to extract user data from claims
export function extractUserDataFromClaims(claims: TokenClaims): UserTokenData {
  return {
    uid: claims.uid,
    email: claims.email || null,
    role: claims.role,
    tokenBalance: claims.tokenBalance,
    features: claims.features,
    permissions: claims.permissions
  };
}

// Helper to create token metadata
export function createTokenMetadata(expiresIn: number = 604800): TokenMetadata { // Default 1 week
  const now = Date.now();
  return {
    createdAt: now,
    expiresAt: now + (expiresIn * 1000),
  };
}
