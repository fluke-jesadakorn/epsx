// OpenID + Web3 Authentication System (Shared)
export { 
  useSharedAuth as useAuth, 
  SharedOpenIDWeb3Provider as AuthProvider,
  useSharedAuth as useOpenIDWeb3Auth
} from '@/shared/components/auth/SharedOpenIDWeb3Provider';

export { openidApiClient as authService } from './openid-api-client';
export { web3OpenidService } from './web3-openid-service';

// Re-export types from shared system
export type { 
  SharedAuthContextValue as AuthContextValue
} from '@/shared/components/auth/SharedOpenIDWeb3Provider';

export type {
  OpenIDTokenResponse,
  Web3AuthRequest,
  UserInfoResponse,
  // UserInfoResponse as User - Removed alias to avoid conflict with shared User type
} from './openid-api-client';

// ApiResponse export temporarily removed to fix build conflict
// Components needing ApiResponse should import directly from shared types

// Legacy compatibility exports (deprecated)
export * from './store';
export * from './utils';