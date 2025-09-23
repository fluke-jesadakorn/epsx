# Web3 Authentication Store Migration

## Overview

The Web3 authentication system has been migrated from a React Context-based approach to a Zustand store-based global state management solution. This provides better performance, easier testing, and more predictable state management.

## Architecture

### Previous Architecture (Context-based)
- `Web3AuthProvider.tsx` - React Context Provider with complex state management
- `web3.ts` - Hook with internal useState and useEffect management
- State tied to component lifecycle and React re-renders

### New Architecture (Store-based)
- `web3-store.ts` - Zustand store with persistence and middleware
- `use-web3-auth.ts` - Hook that bridges Wagmi with the Zustand store
- `Web3AuthProvider.tsx` - Simplified provider that uses the new hook
- Global state management independent of React component lifecycle

## Key Benefits

### 1. **Performance Improvements**
- Reduced unnecessary re-renders through selective subscriptions
- Store state persists across component unmounts
- Optimized selector hooks for specific state slices

### 2. **Better State Management**
- Centralized state in a single store
- Predictable state updates through actions
- Easy to debug and trace state changes

### 3. **Improved Developer Experience**
- Clear separation of concerns
- Easier testing with direct store access
- Better TypeScript support and autocompletion

### 4. **Enhanced Persistence**
- Automatic state persistence to localStorage
- Configurable persistence strategy
- State recovery after page refreshes

## File Structure

```
lib/auth/
├── web3-store.ts           # Main Zustand store with state and actions
├── use-web3-auth.ts        # Hook that bridges Wagmi with store
├── web3-store.test.ts      # Unit tests for store functionality
├── store.ts                # Existing OIDC auth store (unchanged)
└── README.md               # This documentation

providers/
└── Web3AuthProvider.tsx    # Simplified provider using new hook

components/
├── auth/
│   ├── WalletConnectAuth.tsx      # Updated imports
│   ├── WalletDisconnectModal.tsx  # Updated imports
│   ├── Web3PermissionsDisplay.tsx # Updated imports
│   └── LogoutForm.tsx             # Updated imports
└── nav/
    ├── WalletProviderIcon.tsx     # Updated imports
    └── UserManagementDropdown.tsx # Updated imports
```

## Usage

### Basic Usage

```typescript
import { useWeb3Auth } from '@/lib/auth/use-web3-auth';

function MyComponent() {
  const {
    isConnected,
    isAuthenticated,
    walletAddress,
    authenticate,
    disconnect
  } = useWeb3Auth();

  return (
    <div>
      {isConnected ? (
        <button onClick={authenticate}>
          Sign In with Wallet
        </button>
      ) : (
        <span>Connect wallet first</span>
      )}
    </div>
  );
}
```

### Direct Store Access

```typescript
import { useWeb3AuthStore } from '@/lib/auth/web3-store';

function MyComponent() {
  // Select only the state you need to prevent unnecessary re-renders
  const isAuthenticated = useWeb3AuthStore(state => state.isAuthenticated);
  const permissions = useWeb3AuthStore(state => state.permissions);
  
  return <div>Authentication status: {isAuthenticated}</div>;
}
```

### Optimized Selectors

```typescript
import { 
  useWeb3ConnectedState,
  useWeb3AuthenticatedState,
  useWeb3LoadingState 
} from '@/lib/auth/web3-store';

function MyComponent() {
  const { isConnected, walletAddress } = useWeb3ConnectedState();
  const { isAuthenticated, permissions } = useWeb3AuthenticatedState();
  const { isLoading, error } = useWeb3LoadingState();
  
  // Component only re-renders when relevant state changes
}
```

### Permission Checks

```typescript
import { useWeb3Permission, useWeb3Tier } from '@/lib/auth/use-web3-auth';

function ProtectedComponent() {
  const canTrade = useWeb3Permission('epsx:trading:access');
  const hasNFTAccess = useWeb3Tier('nft');
  
  if (!canTrade) {
    return <div>Trading access required</div>;
  }
  
  return <div>Trading interface</div>;
}
```

## State Schema

```typescript
interface Web3AuthState {
  // Connection state
  isConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  isLoading: boolean;
  hasInitialized: boolean;
  
  // User data
  walletAddress?: string;
  permissions: Web3Permission[];
  userTier: 'free' | 'nft' | 'token' | 'dao' | 'enterprise';
  hasApiAccess: boolean;
  error?: string;
}

interface Web3Permission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  metadata?: Record<string, any>;
}
```

## Persistence

The store automatically persists essential connection state to localStorage:

```typescript
// Persisted state
{
  walletAddress: string;
  hasInitialized: boolean;
}

// Non-persisted state (security)
{
  isAuthenticated: false;  // Always requires fresh authentication
  permissions: [];         // Always fetched fresh
  userTier: 'free';       // Always reset
}
```

## Migration Guide

### For Component Updates

Replace old imports:
```typescript
// OLD
import { useWeb3Auth } from '@/lib/auth/web3';
import { formatAddress } from '@/lib/auth/web3';

// NEW
import { useWeb3Auth } from '@/lib/auth/use-web3-auth';
import { formatAddress } from '@/lib/auth/web3-store';
```

### For Custom Hook Usage

The API remains largely the same:
```typescript
// Works the same as before
const { isAuthenticated, authenticate, disconnect } = useWeb3Auth();
```

### For Provider Usage

No changes needed - the provider API is unchanged:
```typescript
// Still works the same
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';
const { isAuthenticated } = useWeb3AuthContext();
```

## Testing

The store includes comprehensive unit tests:

```bash
npm test web3-store.test.ts
```

Tests cover:
- State initialization
- State updates
- Authentication flow
- Permission management
- Utility functions
- Error handling

## Future Enhancements

### Planned Improvements

1. **Enhanced Persistence**
   - Encrypted storage for sensitive data
   - Cross-tab synchronization improvements
   - Better storage error handling

2. **Advanced State Management**
   - Optimistic updates for better UX
   - Automatic retry logic for failed requests
   - Better caching strategies

3. **Developer Tools**
   - Redux DevTools integration
   - Store debugging utilities
   - Performance monitoring

4. **Enhanced Security**
   - Token rotation management
   - Session timeout handling
   - Cross-origin security improvements

## Troubleshooting

### Common Issues

1. **State not persisting**
   - Check localStorage availability
   - Verify persistence configuration

2. **Component not re-rendering**
   - Ensure proper selector usage
   - Check for state mutation

3. **Authentication errors**
   - Verify Wagmi hooks are properly configured
   - Check wallet connection status

### Debug Mode

Enable debug logging:
```typescript
// Add to store configuration
const store = useWeb3AuthStore.getState();
console.log('Current state:', store);
```

## Performance Considerations

### Best Practices

1. **Use Selective Subscriptions**
   ```typescript
   // Good - only subscribes to specific state
   const isAuthenticated = useWeb3AuthStore(state => state.isAuthenticated);
   
   // Avoid - subscribes to entire store
   const store = useWeb3AuthStore();
   ```

2. **Batch State Updates**
   ```typescript
   // Good - single state update
   setState({ isAuthenticated: true, userTier: 'nft' });
   
   // Avoid - multiple updates
   setAuthenticated(true);
   setUserTier('nft');
   ```

3. **Use Optimized Selectors**
   ```typescript
   // Use pre-built optimized selectors
   const { isConnected } = useWeb3ConnectedState();
   ```

### Memory Usage

The store is designed to be memory-efficient:
- Minimal persisted state
- Automatic cleanup on disconnect
- Optimized selector performance
- Garbage collection friendly

## Security Considerations

### Data Handling

1. **Sensitive Data**: Authentication tokens are never persisted
2. **Wallet Address**: Only persisted for reconnection convenience
3. **Permissions**: Always fetched fresh for security
4. **Cross-tab**: Secure session invalidation

### Best Practices

1. Always validate permissions server-side
2. Never rely on client-side state for security decisions
3. Implement proper session timeout handling
4. Use secure communication channels

---

## Summary

The Web3 authentication store migration provides a robust, performant, and maintainable solution for Web3 authentication state management. The new architecture improves developer experience while maintaining security and reliability standards.

For questions or issues, please refer to the test files or create an issue in the project repository.