/**
 * Server-side Web3 Session Utilities for Frontend
 * Web3-First Migration: Uses Web3 wallet signatures and session management
 */
import { clientConfig } from '@/config/env';
import { type EPSXJWTPayload } from '@/shared/auth/jwt';
import { cookies } from 'next/headers';

/**
 * Web3-First: Get Web3 session token from cookies
 */
export async function getWeb3SessionFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('web3_session')?.value ?? null;
  } catch (_error) {
    return null;
  }
}

/**
 * Web3-First: Get wallet address from session cookies
 */
export async function getWalletAddressFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('wallet_address')?.value ?? null;
  } catch (_error) {
    return null;
  }
}

/**
 * Web3-First: Get Web3 signature from cookies
 */
export async function getWeb3SignatureFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('web3_signature')?.value ?? null;
  } catch (_error) {
    return null;
  }
}

/**
 * Web3-First: Get user info from backend using Web3 session
 */
export async function getUserInfoFromWeb3(): Promise<EPSXJWTPayload | null> {
  try {
    const sessionToken = await getWeb3SessionFromCookies();
    const walletAddress = await getWalletAddressFromCookies();

    if (!sessionToken || !walletAddress) {
      return null;
    }

    // Get user info from backend Web3 authentication endpoint
    const backendUrl = clientConfig.backendUrl || 'http://127.0.0.1:8080';
    const response = await fetch(`${backendUrl}/api/auth/web3/user`, {
      headers: {
        'Authorization': `Bearer ${sessionToken}`,
        'Content-Type': 'application/json',
        'X-Wallet-Address': walletAddress
      }
    });

    if (!response.ok) {
      return null;
    }

    const userInfo = await response.json();

    // Convert to EPSXJWTPayload format for compatibility
    return {
      sub: userInfo.wallet_address ?? userInfo.id,
      id: userInfo.id,
      wallet_address: userInfo.wallet_address,
      email: userInfo.email,
      name: userInfo.name,
      permissions: userInfo.permissions ?? [],
      platform_context: userInfo.platform_context,
      exp: Math.floor(Date.now() / 1000) + (60 * 60), // 1 hour from now
      iat: Math.floor(Date.now() / 1000),
      iss: 'epsx-backend-web3',
      aud: 'epsx-frontend',
    } as EPSXJWTPayload;

  } catch (error: any) {
    if (error.code === 'ECONNREFUSED') {
    } else {
    }
    return null;
  }
}

/**
 * Web3-First: Get user session data from Web3 cookies and backend
 */
export async function getSessionFromWeb3(): Promise<{
  isAuthenticated: boolean;
  user: EPSXJWTPayload | null;
}> {
  try {
    const userInfo = await getUserInfoFromWeb3();

    if (!userInfo) {
      return { isAuthenticated: false, user: null };
    }

    return { isAuthenticated: true, user: userInfo };
  } catch (_error) {
    return { isAuthenticated: false, user: null };
  }
}

// ============================================================================
// MAIN EXPORTS - WEB3 FIRST
// ============================================================================
// Legacy OIDC functions removed - Web3-first migration is 100% complete

// Backward compatibility aliases
export const getOIDCAccessTokenFromCookies = getWeb3SessionFromCookies;
export const getOIDCIdTokenFromCookies = getWalletAddressFromCookies;
export const getUserInfoFromOIDC = getUserInfoFromWeb3;
export const getSessionFromOIDC = getSessionFromWeb3;