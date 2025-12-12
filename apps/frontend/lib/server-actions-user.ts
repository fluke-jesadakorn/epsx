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
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('session');
    if (!sessionCookie) {
      return null;
    }

    // TODO: Decode and validate session cookie
    // For now, return null to avoid build errors
    return null;
  } catch (error) {
    console.error('Error getting current user:', error);
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

export async function getStockData(symbol: string) {
  // TODO: Implement when backend is ready
  return null;
}