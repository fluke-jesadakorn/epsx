/**
 * Basic tests for Web3 Auth Store
 * Tests store functionality without requiring browser environment
 */

import { describe, it, expect, beforeEach } from '@jest/globals';
import { useWeb3AuthStore } from '@/lib/auth/web3-store';

// Mock global console methods to reduce test noise
const consoleMethods = ['log', 'warn', 'error'] as const;
const originalConsole = {} as Record<typeof consoleMethods[number], any>;

beforeEach(() => {
  // Reset store state before each test
  useWeb3AuthStore.setState({
    isConnected: false,
    isAuthenticated: false,
    isAuthenticating: false,
    isLoading: false,
    hasInitialized: false,
    permissions: [],
    userTier: 'free',
    hasApiAccess: false,
    walletAddress: undefined,
    error: undefined,
  });

  // Mock console methods
  consoleMethods.forEach(method => {
    originalConsole[method] = console[method];
    console[method] = jest.fn();
  });
});

afterEach(() => {
  // Restore console methods
  consoleMethods.forEach(method => {
    console[method] = originalConsole[method];
  });
});

describe('Web3AuthStore', () => {
  it('should initialize with default state', () => {
    const state = useWeb3AuthStore.getState();
    
    expect(state.isConnected).toBe(false);
    expect(state.isAuthenticated).toBe(false);
    expect(state.isAuthenticating).toBe(false);
    expect(state.isLoading).toBe(false);
    expect(state.hasInitialized).toBe(false);
    expect(state.permissions).toEqual([]);
    expect(state.userTier).toBe('free');
    expect(state.hasApiAccess).toBe(false);
    expect(state.walletAddress).toBeUndefined();
    expect(state.error).toBeUndefined();
  });

  it('should update connection state', () => {
    const { setConnected, setWalletAddress } = useWeb3AuthStore.getState();
    
    setConnected(true);
    setWalletAddress('0x1234567890123456789012345678901234567890');
    
    const state = useWeb3AuthStore.getState();
    expect(state.isConnected).toBe(true);
    expect(state.walletAddress).toBe('0x1234567890123456789012345678901234567890');
  });

  it('should update authentication state', () => {
    const { setAuthenticated, setUserTier, setApiAccess } = useWeb3AuthStore.getState();
    
    setAuthenticated(true);
    setUserTier('nft');
    setApiAccess(true);
    
    const state = useWeb3AuthStore.getState();
    expect(state.isAuthenticated).toBe(true);
    expect(state.userTier).toBe('nft');
    expect(state.hasApiAccess).toBe(true);
  });

  it('should manage permissions', () => {
    const { setPermissions } = useWeb3AuthStore.getState();
    
    const mockPermissions = [
      {
        permission: 'epsx:trading:access',
        source: 'nft' as const,
        metadata: { nft_collection: 'test-collection' }
      }
    ];
    
    setPermissions(mockPermissions);
    
    const state = useWeb3AuthStore.getState();
    expect(state.permissions).toEqual(mockPermissions);
  });

  it('should reset authentication state', () => {
    const { setAuthenticated, setUserTier, setApiAccess, resetAuthState } = useWeb3AuthStore.getState();
    
    // Set authenticated state
    setAuthenticated(true);
    setUserTier('dao');
    setApiAccess(true);
    
    // Reset
    resetAuthState();
    
    const state = useWeb3AuthStore.getState();
    expect(state.isAuthenticated).toBe(false);
    expect(state.userTier).toBe('free');
    expect(state.hasApiAccess).toBe(false);
    expect(state.permissions).toEqual([]);
  });

  it('should handle errors', () => {
    const { setError } = useWeb3AuthStore.getState();
    
    setError('Test error message');
    
    const state = useWeb3AuthStore.getState();
    expect(state.error).toBe('Test error message');
    
    // Clear error
    setError(undefined);
    expect(useWeb3AuthStore.getState().error).toBeUndefined();
  });

  it('should handle loading states', () => {
    const { setLoading, setAuthenticating, setInitialized } = useWeb3AuthStore.getState();
    
    setLoading(true);
    setAuthenticating(true);
    setInitialized(true);
    
    const state = useWeb3AuthStore.getState();
    expect(state.isLoading).toBe(true);
    expect(state.isAuthenticating).toBe(true);
    expect(state.hasInitialized).toBe(true);
  });
});

describe('Web3Store Utility Functions', () => {
  it('should format address correctly', () => {
    const { formatAddress } = require('./web3-store');
    const address = '0x1234567890123456789012345678901234567890';
    const formatted = formatAddress(address);
    expect(formatted).toBe('0x1234...7890');
  });

  it('should check permission expiry correctly', () => {
    const { isPermissionExpired } = require('./web3-store');
    
    const expiredPermission = {
      permission: 'test',
      source: 'manual' as const,
      expires_at: '2020-01-01T00:00:00Z'
    };
    
    const validPermission = {
      permission: 'test',
      source: 'manual' as const,
      expires_at: '2030-01-01T00:00:00Z'
    };
    
    const noExpiryPermission = {
      permission: 'test',
      source: 'manual' as const
    };
    
    expect(isPermissionExpired(expiredPermission)).toBe(true);
    expect(isPermissionExpired(validPermission)).toBe(false);
    expect(isPermissionExpired(noExpiryPermission)).toBe(false);
  });

  it('should return correct permission icons', () => {
    const { getPermissionIcon } = require('./web3-store');
    
    expect(getPermissionIcon('nft')).toBe('🎨');
    expect(getPermissionIcon('token')).toBe('🪙');
    expect(getPermissionIcon('dao')).toBe('🗳️');
    expect(getPermissionIcon('manual')).toBe('👤');
  });

  it('should return correct tier descriptions', () => {
    const { getTierDescription } = require('./web3-store');
    
    expect(getTierDescription('free')).toBe('Basic access to platform features');
    expect(getTierDescription('nft')).toBe('Enhanced access via NFT ownership');
    expect(getTierDescription('token')).toBe('Token-gated premium features');
    expect(getTierDescription('dao')).toBe('DAO governance access and voting');
    expect(getTierDescription('enterprise')).toBe('Full API access and team management');
  });
});