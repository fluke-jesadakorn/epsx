'use server';
import { cookies } from 'next/headers';

import { COOKIES, COOKIE_OPTIONS, getServerAuthToken } from '@/shared/auth/cookies';

import { getExplorerTxLink } from '@/shared/config/constants';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { getBackendUrl } from '@/shared/utils/url-resolver';

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

interface SessionData {
  authenticated: boolean;
  role?: string;
  permissions?: string[];
  wallet_address: string;
}

function mapSession(data: SessionData): AuthUser {
  const d = data as Record<string, unknown>;
  return {
    id: data.wallet_address,
    wallet_address: data.wallet_address,
    emailVerified: true,
    permissions: Array.isArray(data.permissions) ? data.permissions : [],
    role: data.role ?? 'user',
    name: (d.name as string) || (d.email as string) || '',
    email: (d.email as string) || '',
    package_tier: (d.package_tier as string) || 'FREE',
  };
}

/**
 * Attempt token refresh using refresh_token cookie.
 * Safe for SSR: tries to persist cookies but won't fail if it can't.
 */
async function tryRefresh(
  cookieStore: Awaited<ReturnType<typeof cookies>>
): Promise<string | null> {
  try {
    const refreshToken = cookieStore.get(COOKIES.refresh_token)?.value;
    if (refreshToken === undefined || refreshToken === '') {return null;}

    const backendUrl = getBackendUrl('server');
    const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID ?? 'epsx-frontend';

    const res = await fetch(`${backendUrl}/api/auth/session/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken, client_id: clientId }),
      cache: 'no-store',
    });

    if (!res.ok) {return null;}

    const data = (await res.json()) as {
      access_token?: string;
      refresh_token?: string;
      expires_in?: number;
      user?: Record<string, unknown>;
    };
    const newToken = data.access_token;
    if (typeof newToken !== 'string' || newToken === '') {return null;}

    // Try to persist refreshed cookies (works in server actions, silently fails in SSR)
    try {
      cookieStore.set(COOKIES.access_token, newToken, {
        ...COOKIE_OPTIONS.httpOnly,
        maxAge: data.expires_in ?? COOKIE_OPTIONS.maxAge.access_token,
      });
      if (typeof data.refresh_token === 'string' && data.refresh_token !== '') {
        cookieStore.set(COOKIES.refresh_token, data.refresh_token, {
          ...COOKIE_OPTIONS.httpOnly,
          maxAge: COOKIE_OPTIONS.maxAge.refresh_token,
        });
      }
      // Update user cookie with fresh access token
      const userCookie = cookieStore.get(COOKIES.user)?.value;
      if (userCookie !== undefined && userCookie !== '') {
        const existing = JSON.parse(decodeURIComponent(userCookie)) as Record<string, unknown>;
        existing.access = newToken;
        if (data.user) {Object.assign(existing, data.user);}
        cookieStore.set(COOKIES.user, JSON.stringify(existing), {
          ...COOKIE_OPTIONS.clientSide,
          maxAge: COOKIE_OPTIONS.maxAge.user,
        });
      }
      const expiresAt = Date.now() + ((data.expires_in ?? COOKIE_OPTIONS.maxAge.access_token) * 1000);
      cookieStore.set(COOKIES.expires_at, expiresAt.toString(), {
        ...COOKIE_OPTIONS.clientSide,
        maxAge: COOKIE_OPTIONS.maxAge.expires_at,
      });
    } catch {
      // Cookie setting not available in SSR context - token still usable for this request
    }

    return newToken;
  } catch {
    return null;
  }
}

/**
 * Get a valid token, trying refresh if the current one is missing or expired.
 */
async function getValidToken(): Promise<string | null> {
  const cookieStore = await cookies();
  const token = getServerAuthToken(cookieStore);
  if (token !== null) {return token;}
  return tryRefresh(cookieStore);
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
    blockExplorerUrl: txHash !== '' ? getExplorerTxLink(txHash) : ''
  };
}

export async function getCurrentUser(): Promise<AuthUser | null> {
  try {
    const cookieStore = await cookies();
    let token = getServerAuthToken(cookieStore);

    if (token === null) {
      // No token in any cookie - try refresh_token
      token = await tryRefresh(cookieStore);
      if (token === null) {return null;}
    }

    // Validate session with current token
    const client = createFrontendApiClient({ token, serverSide: true });
    const response = await client.get<SessionData>(
      '/api/auth/web3/session', undefined, { cache: 'no-store' }
    );

    if (response.success && response.data?.authenticated) {
      return mapSession(response.data);
    }

    // Token expired/invalid - attempt refresh
    const freshToken = await tryRefresh(cookieStore);
    if (freshToken === null) {return null;}

    const freshClient = createFrontendApiClient({ token: freshToken, serverSide: true });
    const retry = await freshClient.get<SessionData>(
      '/api/auth/web3/session', undefined, { cache: 'no-store' }
    );

    if (retry.success && retry.data?.authenticated) {
      return mapSession(retry.data);
    }

    return null;
  } catch (_error) {
    return null;
  }
}

export async function getPaymentHistory() {
  try {
    const token = await getValidToken();
    if (token === null) {return [];}

    const client = createFrontendApiClient({ token, serverSide: true });

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
    const token = await getValidToken();
    if (token === null) {
      return { hasAccess: false, reason: 'Not authenticated', limits: undefined };
    }

    const client = createFrontendApiClient({ token, serverSide: true });

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
    const token = await getValidToken();
    if (token === null) {
      return { status: 'unauthenticated', activeSubscription: null, paymentHistory: [] };
    }

    const client = createFrontendApiClient({ token, serverSide: true });

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