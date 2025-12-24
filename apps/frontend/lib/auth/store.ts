'use client';

/**
 * FRONTEND AUTH STORE
 * Unified Web3/SIWE state management
 */

import { createFrontendAuthStore } from '@/shared/auth/store';

// Create store instance safely
// Note: We use a lazy initializer to avoid calling client-only code during server-side evaluation/bundling
const store = typeof window !== 'undefined' ? createFrontendAuthStore() : null;

// Export hook that only works on client
export const useWeb3AuthStore = ((...args: any[]) => {
    if (typeof window === 'undefined') {
        // Return a dummy state for server-side evaluation if called (should not happen in RSC)
        return {};
    }
    if (!store) return {};
    return (store as any)(...args);
}) as any;

// Export as default for convenience
export default useWeb3AuthStore;