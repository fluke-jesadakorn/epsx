'use server';
import { cookies } from 'next/headers';

import { COOKIES, getServerAuthToken } from '@/shared/auth/cookies';

import { createFrontendApiClient } from '@/shared/utils/api-client';

export interface AuthUser {
  id: string;
  email?: string;
  user_id?: string;
  wallet_address?: string;
  walletAddress?: string;
  emailVerified?: boolean;
  permissions?: string[] | Record<string, unknown>;
  role?: string;
  package_tier?: string;
  name?: string;
}

function getStringValue(value: unknown, defaultValue = ''): string {
  return typeof value === 'string' ? value : defaultValue;
}

function getNumberValue(value: unknown, defaultValue = 0): number {
  return typeof value === 'number' ? value : defaultValue;
}

function mapPaymentToTransaction(payment: Record<string, unknown>) {
  const paymentRef = getStringValue(payment.payment_reference);
  const paymentId = getStringValue(payment.id);
  const txHash = getStringValue(payment.tx_hash);

  return {
    orderNo: paymentRef || paymentId,
    actualAmount: getNumberValue(payment.amount),
    currency: getStringValue(payment.currency, 'USD'),
    status: getStringValue(payment.status, 'pending'),
    finishTime: getStringValue(payment.completed_at) || getStringValue(payment.created_at) || new Date().toISOString(),
    blockchainData: {
      txHash,  // Backend returns 'tx_hash' not 'transaction_hash'
      network: 'BSC'
    },
    blockExplorerUrl: txHash !== '' ? `https://bscscan.com/tx/${txHash}` : ''
  };
}

export async function getCurrentUser(): Promise<AuthUser | null> {
  try {
    const cookieStore = await cookies();
    const token = getServerAuthToken(cookieStore);

    if (token === null) {
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
    const role = data.role ?? 'user';
    const perms = Array.isArray(data.permissions) ? data.permissions : [];

    // Map backend response to AuthUser
    return {
      id: data.wallet_address,
      wallet_address: data.wallet_address,
      emailVerified: true, // Wallet is always verified
      permissions: perms,
      role,
      name: (data as Record<string, unknown>).name as string || (data as Record<string, unknown>).email as string || '',
      email: (data as Record<string, unknown>).email as string || '',
      package_tier: (data as Record<string, unknown>).package_tier as string || 'FREE',
    };

  } catch (_error) {
    return null;
  }
}

export async function getPaymentHistory() {
  try {
    const cookieStore = await cookies();
    const token = getServerAuthToken(cookieStore);

    if (token === null) {
      return [];
    }

    // Use UnifiedApiClient with the token
    const client = createFrontendApiClient({
      token,
      serverSide: true
    });

    const response = await client.get<{ payments: Record<string, unknown>[] }>('/api/payments/history', undefined, {
      cache: 'no-store'
    });

    if (!response.success || !response.data) {
      return [];
    }

    // Map backend response to Transaction format expected by PaymentStatusSection
    const payments = response.data.payments;
    return payments.map(mapPaymentToTransaction);
  } catch (_error) {
    return [];
  }
}

export async function checkFeatureAccess(feature: string) {
  try {
    const cookieStore = await cookies();
    const token = getServerAuthToken(cookieStore);

    if (token === null) {
      return {
        hasAccess: false,
        reason: 'Not authenticated',
        limits: undefined
      };
    }

    const client = createFrontendApiClient({
      token,
      serverSide: true
    });

    const response = await client.get<{
      has_permission: boolean;
      reason?: string;
      limits?: { daily?: number; monthly?: number };
    }>(`/api/permissions/check?permission=${encodeURIComponent(feature)}`, undefined, {
      cache: 'no-store'
    });

    if (!response.success || !response.data) {
      return {
        hasAccess: false,
        reason: response.error ?? 'Permission check failed',
        limits: undefined
      };
    }

    return {
      hasAccess: response.data.has_permission,
      reason: response.data.reason ?? (response.data.has_permission ? 'Access granted' : 'Access denied'),
      limits: response.data.limits
    };
  } catch (_error) {
    return {
      hasAccess: false,
      reason: 'Error checking permissions',
      limits: undefined
    };
  }
}

export async function getPaymentStatus(paymentId?: string) {
  try {
    const cookieStore = await cookies();
    const token = getServerAuthToken(cookieStore);

    if (token === null) {
      return {
        status: 'unauthenticated',
        activeSubscription: null,
        paymentHistory: []
      };
    }

    const client = createFrontendApiClient({
      token,
      serverSide: true
    });

    // Get subscription status
    const subResponse = await client.get<{
      subscriptions: Array<{
        id: string;
        plan_name: string;
        status: string;
        expires_at: string;
      }>;
    }>('/api/subscriptions/my', undefined, { cache: 'no-store' });

    const activeSubscription = subResponse.success && subResponse.data?.subscriptions
      ? subResponse.data.subscriptions.find(s => s.status === 'active') ?? null
      : null;

    // Get specific payment if ID provided
    if (paymentId !== undefined) {
      const paymentResponse = await client.get<{ payment: { status: string } }>(
        `/api/payments/${paymentId}`,
        undefined,
        { cache: 'no-store' }
      );
      const paymentStatus = paymentResponse.data?.payment.status;
      return {
        status: typeof paymentStatus === 'string' ? paymentStatus : 'unknown',
        activeSubscription,
        paymentHistory: []
      };
    }

    return {
      status: activeSubscription ? 'subscribed' : 'none',
      activeSubscription,
      paymentHistory: []
    };
  } catch (_error) {
    return {
      status: 'error',
      activeSubscription: null,
      paymentHistory: []
    };
  }
}

export async function getBatchStocks(_symbols: string[]) {
  // TODO: Implement when backend is ready
  return {
    success: true,
    data: {},
    errors: [],
    cached: _symbols,
    fetched: []
  };
}

export async function preloadStocks(_symbols: string[]) {
  // TODO: Implement when backend is ready
}

export async function checkStockCacheStatus(symbols: string[]) {
  // TODO: Implement when backend is ready
  return {
    cached: {},
    notCached: symbols,
    symbols
  };
}

export async function getDebugSessionInfo() {
  try {
    const { headers } = await import('next/headers');
    const headerStore = await headers();
    const cookieStore = await cookies();

    const clientSession = cookieStore.get(COOKIES.sid)?.value;
    const accessCookie = cookieStore.get(COOKIES.access_token)?.value;
    const allCookies = cookieStore.getAll().map(c => `${c.name} (${c.value.length} chars)`);
    const rawHeader = headerStore.get('cookie');

    const hasClientSession = clientSession !== undefined;
    const hasAccessCookie = accessCookie !== undefined;
    const hasRawHeader = rawHeader !== null;

    return {
      foundClientSession: hasClientSession,
      foundAccessCookie: hasAccessCookie,
      clientSessionLength: hasClientSession ? clientSession.length : 0,
      clientSessionPreview: hasClientSession ? `${clientSession.slice(0, 10)  }...` : 'none',
      accessCookieLength: hasAccessCookie ? accessCookie.length : 0,
      backendUrl: process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080',
      allCookieNames: allCookies,
      rawCookieHeader: hasRawHeader ? (`${rawHeader.slice(0, 50)  }...`) : 'missing'
    };
  } catch (e) {
    return { error: 'Failed to get debug info', details: String(e) };
  }
}