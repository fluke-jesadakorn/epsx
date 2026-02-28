'use client';

import { useAccount } from 'wagmi';
import { useSharedAuth } from '../components/auth/provider';

/**
 * Progressive authentication hook.
 * Call requireAuth() before any protected action - it opens the SIWE modal
 * if not authenticated and resolves when auth is complete or cancelled.
 *
 * Usage:
 *   const { requireAuth } = useRequireAuth();
 *   const ok = await requireAuth();
 *   if (!ok) return; // user cancelled
 */
export function useRequireAuth() {
    const { isAuthenticated, requireAuth } = useSharedAuth();
    const { address: connectedAddress, isConnected } = useAccount();

    return { isAuthenticated, isConnected, connectedAddress, requireAuth };
}
