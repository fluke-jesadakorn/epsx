'use server';
import { cookies } from 'next/headers';

export interface AuthUser {
  id: string;
  email?: string;
  user_id?: string;
  wallet_address?: string;
  walletAddress?: string;
  emailVerified?: boolean;
  permissions?: string[] | Record<string, any>;
  role?: string;
}

export async function getCurrentUser() {
  console.log('🔍 [Debug] getCurrentUser: Starting check');
  try {
    const cookieStore = await cookies();

    // Check for our synced client session first, then fallback to httpOnly access token
    let token = cookieStore.get('epsx.client_session')?.value;

    if (!token) {
      token = cookieStore.get('epsx.access')?.value;
      if (token) console.log('🔍 [Debug] getCurrentUser: Found epsx.access cookie');
    } else {
      console.log('🔍 [Debug] getCurrentUser: Found epsx.client_session cookie');
    }

    if (!token) {
      console.log('❌ [Debug] getCurrentUser: No session token found in cookies');
      return null;
    }
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    console.log(`🔍 [Debug] getCurrentUser: Verifying token with ${backendUrl}`);

    const response = await fetch(`${backendUrl}/api/auth/web3/session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store' // Ensure we always get fresh status
    });

    console.log(`🔍 [Debug] getCurrentUser: Backend response status: ${response.status}`);

    if (!response.ok) {
      console.log('❌ [Debug] getCurrentUser: Backend rejected token');
      // Token invalid or expired
      return null;
    }

    const data = await response.json();
    console.log(`🔍 [Debug] getCurrentUser: Backend data:`, JSON.stringify(data));

    if (!data.authenticated) {
      console.log('❌ [Debug] getCurrentUser: Data says not authenticated');
      return null;
    }

    console.log('✅ [Debug] getCurrentUser: Contenticated successfully');
    // Map backend response to AuthUser
    return {
      id: data.wallet_address,
      wallet_address: data.wallet_address,
      emailVerified: true, // Wallet is always verified
      permissions: data.permissions || [],
      role: data.role || 'user', // Default role if not provided
    };

  } catch (error) {
    console.error('❌ [Debug] error getting current user:', error);
    return null;
  }
}

export async function getPaymentHistory() {
  // TODO: Implement when backend is ready
  return [];
}

export async function checkFeatureAccess(feature: string) {
  // TODO: Implement when backend is ready
  return {
    hasAccess: true,
    reason: 'Access granted',
    limits: undefined
  };
}

export async function getPaymentStatus(paymentId?: string) {
  // TODO: Implement when backend is ready
  return {
    status: 'none',
    activeSubscription: null,
    paymentHistory: []
  };
}

export async function getBatchStocks(symbols: string[]) {
  // TODO: Implement when backend is ready
  return {
    success: true,
    data: {},
    errors: [],
    cached: symbols,
    fetched: []
  };
}

export async function preloadStocks(symbols: string[]) {
  // TODO: Implement when backend is ready
  console.log('Preloading stocks:', symbols);
}

export async function checkStockCacheStatus(symbols: string[]) {
  // TODO: Implement when backend is ready
  return {
    cached: {},
    notCached: symbols,
    symbols: symbols
  };
}

export async function getDebugSessionInfo() {
  try {
    const { headers } = await import('next/headers');
    const headerStore = await headers();
    const cookieStore = await cookies();

    const clientSession = cookieStore.get('epsx.client_session')?.value;
    const accessCookie = cookieStore.get('epsx.access')?.value;
    const allCookies = cookieStore.getAll().map(c => `${c.name} (${c.value.length} chars)`);
    const rawHeader = headerStore.get('cookie');

    return {
      foundClientSession: !!clientSession,
      foundAccessCookie: !!accessCookie,
      clientSessionLength: clientSession ? clientSession.length : 0,
      clientSessionPreview: clientSession ? clientSession.substring(0, 10) + '...' : 'none',
      accessCookieLength: accessCookie ? accessCookie.length : 0,
      backendUrl: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
      allCookieNames: allCookies,
      rawCookieHeader: rawHeader ? (rawHeader.substring(0, 50) + '...') : 'missing'
    };
  } catch (e) {
    return { error: 'Failed to get debug info', details: String(e) };
  }
}