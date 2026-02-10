// OpenID + Web3 Authentication System (Shared)
export {
  SharedOpenIDWeb3Provider as AuthProvider, useSharedAuth as useAuth, useSharedAuth as useOpenIDWeb3Auth
} from '@/shared/components/auth/provider';

export { openidApiClient as authService } from './api-client';
// export { web3OpenidService } from './web3-openid-service'; // TODO: Implement web3OpenidService

// Re-export types from shared system
export type {
  SharedAuthContextValue as AuthContextValue
} from '@/shared/components/auth/provider';

export type {
  OpenIDTokenResponse, UserInfoResponse, Web3AuthRequest
} from './api-client';

// ApiResponse export temporarily removed to fix build conflict
// Components needing ApiResponse should import directly from shared types

// Legacy compatibility exports (deprecated)
export * from './utils';
