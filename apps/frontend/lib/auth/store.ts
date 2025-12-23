/**
 * FRONTEND AUTH STORE
 * Unified Web3/SIWE state management
 */

import { createFrontendAuthStore } from '@/shared/auth/store';

// Keep same name for backward compatibility
export const useWeb3AuthStore = createFrontendAuthStore();

// Export as default for convenience
export default useWeb3AuthStore;