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

import { createFrontendApiClient } from '@/shared/utils/api-client';

export async function getCurrentUser() {
  try {
    const cookieStore = await cookies();

    // Check for our synced client session first, then fallback to httpOnly access token
    let token = cookieStore.get('epsx.client_session')?.value;

    if (!token) {
      token = cookieStore.get('epsx.access')?.value;
      if (token) console.log('🔍 [Debug] getCurrentUser: Found epsx.access cookie');
    }

    if (!token) {
      return null;
    }

    // Use UnifiedApiClient with the token we found
    const client = createFrontendApiClient({
      token,
      serverSide: true
    });

    const response = await client.get<{
      authenticated: boolean;
      role?: string;
      permissions?: string[];
      wallet_address: string;
    }>('/api/auth/web3/session', undefined, {
      cache: 'no-store'
    });

    if (!response.success || !response.data) {
      // Token invalid or expired
      return null;
    }

    const data = response.data;

    if (!data.authenticated) {
      return null;
    }

    // Use backend-provided role directly (computed server-side)
    // Backend computes: "user" | "admin" | "super_admin" based on permissions
    const role = data.role || 'user';
    const perms = Array.isArray(data.permissions) ? data.permissions : [];

    // Map backend response to AuthUser
    return {
      id: data.wallet_address,
      wallet_address: data.wallet_address,
      emailVerified: true, // Wallet is always verified
      permissions: perms,
      role: role,
    };

  } catch (error) {
    console.error('❌ [Debug] error getting current user:', error);
    return null;
  }
}

export async function getPaymentHistory() {
  try {
    const cookieStore = await cookies();

    // Get auth token
    let token = cookieStore.get('epsx.client_session')?.value;
    if (!token) {
      token = cookieStore.get('epsx.access')?.value;
    }

    if (!token) {
      console.log('[getPaymentHistory] No auth token found');
      return [];
    }

    // Use UnifiedApiClient with the token
    const client = createFrontendApiClient({
      token,
      serverSide: true
    });

    const response = await client.get<{ payments: any[] }>('/api/payments/history', undefined, {
      cache: 'no-store'
    });

    if (!response.success || !response.data) {
      console.error('[getPaymentHistory] Failed to fetch or invalid response:', response.error);
      return [];
    }

    // Map backend response to Transaction format expected by PaymentStatusSection
    const payments = response.data.payments || [];
    return payments.map((payment: any) => ({
      orderNo: payment.payment_reference || payment.id || '',
      actualAmount: payment.amount || 0,
      currency: payment.currency || 'USD',
      status: payment.status || 'pending',
      finishTime: payment.completed_at || payment.created_at || new Date().toISOString(),
      blockchainData: {
        txHash: payment.tx_hash || '',  // Backend returns 'tx_hash' not 'transaction_hash'
        network: 'BSC'
      },
      blockExplorerUrl: payment.tx_hash
        ? `https://bscscan.com/tx/${payment.tx_hash}`
        : ''
    }));
  } catch (error) {
    console.error('[getPaymentHistory] Error:', error);
    return [];
  }
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