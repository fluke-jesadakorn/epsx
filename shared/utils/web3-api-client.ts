/**
 * Web3 API Client
 * Type-safe client for backend Web3 wallet authentication endpoints
 * Builds on the unified API client with Web3-specific methods
 */

import { createFrontendApiClient, createAdminApiClient, UnifiedApiClient, ApiResponse } from './api-client'
import type { 
  WalletChallengeRequest,
  WalletChallengeResponse,
  WalletVerificationRequest,
  WalletAuthResponse,
  WalletSessionData,
  WalletPermissionsResponse,
  WalletPermissionEntry,
  FrontendSessionResponse,
  AdminSessionResponse
} from '../types/wallet-auth'

// ============================================================================
// WEB3 API CLIENT CLASS
// ============================================================================

export class Web3ApiClient {
  private client: UnifiedApiClient
  private isAdmin: boolean

  constructor(options: {
    client?: UnifiedApiClient
    isAdmin?: boolean
    baseURL?: string
    token?: string
    serverSide?: boolean
  } = {}) {
    this.isAdmin = options.isAdmin || false
    
    if (options.client) {
      this.client = options.client
    } else {
      // Create appropriate client based on context
      this.client = this.isAdmin 
        ? createAdminApiClient({
            baseURL: options.baseURL,
            token: options.token,
            serverSide: options.serverSide
          })
        : createFrontendApiClient({
            baseURL: options.baseURL,
            token: options.token,
            serverSide: options.serverSide
          })
    }
  }

  // ============================================================================
  // WEB3 AUTHENTICATION METHODS
  // ============================================================================

  /**
   * Get Web3 challenge for wallet signature
   * Endpoint: GET /api/v1/auth/web3/challenge?wallet_address=...
   */
  async getChallenge(walletAddress: string): Promise<WalletChallengeResponse> {
    const response = await this.client.get<WalletChallengeResponse>(
      '/api/v1/auth/web3/challenge',
      { wallet_address: walletAddress }
    )

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get wallet challenge')
    }

    return response.data
  }

  /**
   * Verify wallet signature and authenticate
   * Endpoint: POST /api/v1/auth/web3/verify
   */
  async verifySignature(request: WalletVerificationRequest): Promise<WalletAuthResponse> {
    const headers: Record<string, string> = {}
    
    // Add admin context header if this is admin client
    if (this.isAdmin) {
      headers['X-Admin-Context'] = 'true'
    }

    const response = await this.client.post<WalletAuthResponse>(
      '/api/v1/auth/web3/verify',
      request,
      { headers }
    )

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to verify wallet signature')
    }

    return response.data
  }

  /**
   * Get current session data
   * Endpoint: GET /api/v1/auth/web3/session
   */
  async getSession(): Promise<WalletSessionData> {
    const headers: Record<string, string> = {}
    
    if (this.isAdmin) {
      headers['X-Admin-Context'] = 'true'
    }

    const response = await this.client.get<WalletSessionData>(
      '/api/v1/auth/web3/session',
      {},
      { headers }
    )

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get session data')
    }

    return response.data
  }

  /**
   * Get wallet permissions with optional filtering
   * Endpoint: GET /api/v1/auth/web3/permissions
   */
  async getPermissions(filters?: {
    wallet_address?: string
    permission?: string
    source?: string
    limit?: number
    offset?: number
  }): Promise<WalletPermissionsResponse> {
    const headers: Record<string, string> = {}
    
    if (this.isAdmin) {
      headers['X-Admin-Context'] = 'true'
    }

    const response = await this.client.get<WalletPermissionsResponse>(
      '/api/v1/auth/web3/permissions',
      filters,
      { headers }
    )

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get permissions')
    }

    return response.data
  }

  /**
   * Refresh current session and get new tokens
   * Endpoint: POST /api/v1/auth/web3/refresh
   */
  async refreshSession(): Promise<WalletSessionData> {
    const headers: Record<string, string> = {}
    
    if (this.isAdmin) {
      headers['X-Admin-Context'] = 'true'
    }

    const response = await this.client.post<WalletSessionData>(
      '/api/v1/auth/web3/refresh',
      {},
      { headers }
    )

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to refresh session')
    }

    return response.data
  }

  /**
   * Logout and invalidate session
   * Endpoint: DELETE /api/v1/auth/web3/logout
   */
  async logout(): Promise<void> {
    const headers: Record<string, string> = {}
    
    if (this.isAdmin) {
      headers['X-Admin-Context'] = 'true'
    }

    const response = await this.client.delete(
      '/api/v1/auth/web3/logout',
      { headers }
    )

    if (!response.success) {
      throw new Error(response.error || 'Failed to logout')
    }
  }

  // ============================================================================
  // ADMIN-SPECIFIC METHODS
  // ============================================================================

  /**
   * Get users list (admin only)
   * Endpoint: GET /api/v1/admin/users
   */
  async getUsers(): Promise<any[]> {
    if (!this.isAdmin) {
      throw new Error('Admin access required for this endpoint')
    }

    const response = await this.client.get<any[]>(
      '/api/v1/admin/users',
      {},
      { headers: { 'X-Admin-Context': 'true' } }
    )

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get users')
    }

    return response.data
  }

  // ============================================================================
  // PERMISSION UTILITY METHODS
  // ============================================================================

  /**
   * Check if current user has specific permission
   */
  async hasPermission(permission: string): Promise<boolean> {
    try {
      const session = await this.getSession()
      return session.permissions.includes(permission)
    } catch {
      return false
    }
  }

  /**
   * Check if current user has admin permissions
   */
  async hasAdminPermissions(): Promise<boolean> {
    try {
      const session = await this.getSession()
      return session.permissions.some(p => 
        p === 'admin:*:*' || 
        p.startsWith('admin:') ||
        p === 'epsx:admin:*' ||
        p === 'epsx:*:*'
      )
    } catch {
      return false
    }
  }

  /**
   * Check if current user has any of the specified permissions
   */
  async hasAnyPermission(permissions: string[]): Promise<boolean> {
    try {
      const session = await this.getSession()
      return permissions.some(perm => session.permissions.includes(perm))
    } catch {
      return false
    }
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  /**
   * Set authentication token
   */
  setAuthToken(token: string): void {
    this.client.setAuthToken(token)
  }

  /**
   * Remove authentication token
   */
  removeAuthToken(): void {
    this.client.removeAuthToken()
  }

  /**
   * Get the underlying unified client
   */
  getUnifiedClient(): UnifiedApiClient {
    return this.client
  }

  /**
   * Create a new Web3 client with different configuration
   */
  clone(overrides: {
    isAdmin?: boolean
    baseURL?: string
    token?: string
    serverSide?: boolean
  } = {}): Web3ApiClient {
    return new Web3ApiClient({
      isAdmin: overrides.isAdmin ?? this.isAdmin,
      baseURL: overrides.baseURL,
      token: overrides.token,
      serverSide: overrides.serverSide
    })
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Web3 API client for frontend application
 */
export function createWeb3FrontendClient(options: {
  baseURL?: string
  token?: string
  serverSide?: boolean
} = {}): Web3ApiClient {
  return new Web3ApiClient({
    isAdmin: false,
    ...options
  })
}

/**
 * Create Web3 API client for admin application
 */
export function createWeb3AdminClient(options: {
  baseURL?: string
  token?: string
  serverSide?: boolean
} = {}): Web3ApiClient {
  return new Web3ApiClient({
    isAdmin: true,
    ...options
  })
}

/**
 * Create Web3 API client based on context
 */
export function createWeb3Client(isAdmin: boolean, options: {
  baseURL?: string
  token?: string
  serverSide?: boolean
} = {}): Web3ApiClient {
  return new Web3ApiClient({
    isAdmin,
    ...options
  })
}

// ============================================================================
// CONVENIENCE FUNCTIONS FOR COMMON OPERATIONS
// ============================================================================

/**
 * Complete Web3 authentication flow
 */
export async function authenticateWallet(
  client: Web3ApiClient,
  walletAddress: string,
  signMessage: (message: string) => Promise<string>
): Promise<WalletAuthResponse> {
  // Step 1: Get challenge
  const challenge = await client.getChallenge(walletAddress)
  
  // Step 2: Sign message
  const signature = await signMessage(challenge.message)
  
  // Step 3: Verify signature
  return client.verifySignature({
    wallet_address: walletAddress,
    signature,
    message: challenge.message,
    nonce: challenge.nonce
  })
}

/**
 * Check wallet permissions helper
 */
export async function checkWalletPermissions(
  client: Web3ApiClient,
  requiredPermissions: string[]
): Promise<{
  hasAccess: boolean
  missingPermissions: string[]
  userPermissions: string[]
}> {
  try {
    const session = await client.getSession()
    const userPermissions = session.permissions
    const missingPermissions = requiredPermissions.filter(
      perm => !userPermissions.includes(perm)
    )
    
    return {
      hasAccess: missingPermissions.length === 0,
      missingPermissions,
      userPermissions
    }
  } catch {
    return {
      hasAccess: false,
      missingPermissions: requiredPermissions,
      userPermissions: []
    }
  }
}

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type {
  WalletChallengeRequest,
  WalletChallengeResponse,
  WalletVerificationRequest,
  WalletAuthResponse,
  WalletSessionData,
  WalletPermissionsResponse,
  WalletPermissionEntry
}