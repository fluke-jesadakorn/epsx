/**
 * Server-side JWT Cookie Utilities for Admin Frontend - WEB3-FIRST
 * Phase 4.3: Refactored to delegate verification to Backend API (Sidecar Pattern)
 * This avoids local key management and algorithm mismatch issues (HS256 vs RS256)
 */
import { decodeJwt } from 'jose';
import { cookies } from 'next/headers';

import { config } from '@/config/env';
import { COOKIES } from '@/shared/auth/cookies';
import type { EPSXJWTPayload } from '@/shared/auth/jwt';
import { logger } from '@/shared/utils/logger';

export type { EPSXJWTPayload };

/**
 * Extract wallet address from JWT without verification
 * Used to provide context for the backend verification call
 */
function getWalletAddressFromToken(token: string): string | null {
  try {
    const claims = decodeJwt(token);
    // In our backend, 'sub' is the wallet address, also present as 'wallet_address'
    return (claims.wallet_address as string | undefined) ?? (claims.sub) ?? null;
  } catch (error) {
    logger.error('❌ Failed to decode JWT for wallet address:', error);
    return null;
  }
}

/**
 * Get JWT token from httpOnly cookies
 */
export async function getJWTFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();

    // Check multiple cookie possibilities for robustness
    const jwtCookie = cookieStore.get(COOKIES.access_token) ??
      cookieStore.get('epsx.access_token') ??
      cookieStore.get('access_token');

    return jwtCookie?.value ?? null;
  } catch (error) {
    logger.error('❌ Failed to get JWT from cookies:', error);
    return null;
  }
}

/**
 * Verify JWT using Backend API (Delegated Verification)
 * Instead of verifying locally, we ask the backend "Who is this?"
 */
export async function verifyJWTWithBackend(token: string): Promise<EPSXJWTPayload | null> {
  try {
    const walletAddress = getWalletAddressFromToken(token);

    if (!walletAddress) {
      logger.error('❌ Could not extract wallet address from token');
      return null;
    }

    const backendUrl = config.backendUrl;

    // Call Backend Web3 Session Endpoint
    // We use the session endpoint because it validates the token and returns permissions
    const response = await fetch(`${backendUrl}/api/auth/web3/session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        'X-Wallet-Address': walletAddress
      },
      cache: 'no-store' // Critical: Always validate with backend
    });

    if (!response.ok) {
      // If backend says 401/403, the token is invalid
      if (response.status === 401 || response.status === 403) {
        logger.warn(`⚠️ Backend rejected session: ${response.status}`);
      } else if (response.status === 404) {
        // Handle 404 specifically - might mean user not found despite valid token signature
        logger.warn(`⚠️ Backend could not find user session: ${response.status}`);
      } else {
        logger.error(`❌ Backend verification error: ${response.status}`);
      }
      return null;
    }

    const sessionData = await response.json() as {
      authenticated: boolean;
      wallet_address: string;
      permissions: string[]
    };

    if (!sessionData.authenticated) {
      logger.warn('⚠️ Backend returned unauthenticated session');
      return null;
    }

    // Extract iat/exp from original token since session endpoint doesn't return them
    // This is safe because we just validated the token with the backend
    const claims = decodeJwt(token);

    // Map backend response to EPSXJWTPayload standard
    return {
      sub: sessionData.wallet_address,
      id: sessionData.wallet_address,
      wallet_address: sessionData.wallet_address,
      email: `${sessionData.wallet_address}@web3.epsx.io`, // synthetic email
      name: `Admin (${sessionData.wallet_address.slice(0, 6)}...${sessionData.wallet_address.slice(-4)})`,
      permissions: sessionData.permissions,
      platform_context: '', // Missing from session endpoint, default to empty string
      iss: 'epsx-backend',
      aud: 'epsx-admin',
      exp: claims.exp ?? Math.floor(Date.now() / 1000) + 3600,
      iat: claims.iat ?? Math.floor(Date.now() / 1000),
    } as EPSXJWTPayload;

  } catch (error: unknown) {
    const err = error as { code?: string; message?: string };
    if (err.code === 'ECONNREFUSED') {
      logger.error(`❌ Backend connection refused at ${config.backendUrl}. Is the backend running?`);
    } else {
      logger.error('❌ Backend verification request failed:', error);
    }
    return null;
  }
}

/**
 * Verify and decode JWT token from cookies
 * Now delegates to backend
 */
export async function verifyJWTFromCookies(): Promise<EPSXJWTPayload | null> {
  try {
    const token = await getJWTFromCookies();
    if (!token) { return null; }

    return await verifyJWTWithBackend(token);
  } catch (error) {
    logger.error('❌ Failed to verify JWT from cookies:', error);
    return null;
  }
}

/**
 * Get user session data from JWT cookies
 */
export async function getSessionFromJWT(): Promise<{
  isAuthenticated: boolean;
  user: EPSXJWTPayload | null;
}> {
  try {
    const payload = await verifyJWTFromCookies();

    if (!payload) {
      return { isAuthenticated: false, user: null };
    }

    return { isAuthenticated: true, user: payload };
  } catch (error) {
    logger.error('❌ Failed to get session from JWT:', error);
    return { isAuthenticated: false, user: null };
  }
}