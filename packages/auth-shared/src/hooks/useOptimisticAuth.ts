import { useCallback, useState, useRef, useTransition } from 'react';
import type { AuthenticatedUser } from '@epsx/types';

export interface OptimisticUpdate<TUser extends AuthenticatedUser = AuthenticatedUser> {
  id: string;
  type: 'user_update' | 'permissions_update' | 'role_update' | 'login' | 'logout';
  timestamp: number;
  optimisticData: {
    user?: TUser | null;
    permissions?: string[];
    roles?: string[];
    profiles?: string[];
  };
  rollbackData: {
    user?: TUser | null;
    permissions?: string[];
    roles?: string[];
    profiles?: string[];
  };
}

export interface OptimisticAuthState<TUser extends AuthenticatedUser = AuthenticatedUser> {
  user: TUser | null;
  permissions: string[];
  roles: string[];
  profiles: string[];
  isAuthenticated: boolean;
  isLoading: boolean;
}

export interface UseOptimisticAuthOptions {
  /** Maximum number of optimistic updates to track */
  maxUpdates?: number;
  /** Auto-rollback timeout in milliseconds */
  rollbackTimeout?: number;
  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Optimistic authentication hook
 * Provides optimistic UI updates for auth operations with rollback capability
 */
export function useOptimisticAuth<TUser extends AuthenticatedUser = AuthenticatedUser>(
  serverState: OptimisticAuthState<TUser>,
  options: UseOptimisticAuthOptions = {}
) {
  const {
    maxUpdates = 10,
    rollbackTimeout = 5000,
    debug = false,
  } = options;

  const timeoutRefs = useRef<Map<string, NodeJS.Timeout>>(new Map());
  const [isPending, startTransition] = useTransition();
  
  // Track optimistic updates
  const [optimisticUpdates, setOptimisticUpdates] = useState<OptimisticUpdate<TUser>[]>([]);

  // Generate unique update ID
  const generateUpdateId = useCallback(() => {
    return `update_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }, []);

  // Apply optimistic updates to server state
  const computeOptimisticState = useCallback((): OptimisticAuthState<TUser> => {
    let currentState = { ...serverState };

    for (const update of optimisticUpdates) {
      const { optimisticData } = update;
      
      if (optimisticData.user !== undefined) {
        currentState.user = optimisticData.user;
        currentState.isAuthenticated = !!optimisticData.user;
      }
      
      if (optimisticData.permissions !== undefined) {
        currentState.permissions = optimisticData.permissions;
      }
      
      if (optimisticData.roles !== undefined) {
        currentState.roles = optimisticData.roles;
      }
      
      if (optimisticData.profiles !== undefined) {
        currentState.profiles = optimisticData.profiles;
      }
    }

    return currentState;
  }, [serverState, optimisticUpdates]);

  // Get current optimistic state
  const optimisticState = computeOptimisticState();

  // Clear timeout for an update
  const clearUpdateTimeout = useCallback((updateId: string) => {
    const timeout = timeoutRefs.current.get(updateId);
    if (timeout) {
      clearTimeout(timeout);
      timeoutRefs.current.delete(updateId);
    }
  }, []);

  // Remove an optimistic update
  const removeOptimisticUpdate = useCallback((updateId: string) => {
    setOptimisticUpdates(prev => prev.filter(update => update.id !== updateId));
    clearUpdateTimeout(updateId);
    
    if (debug) {
      console.log('Removed optimistic update:', updateId);
    }
  }, [clearUpdateTimeout, debug]);

  // Rollback an optimistic update
  const rollbackOptimisticUpdate = useCallback((updateId: string) => {
    const update = optimisticUpdates.find(u => u.id === updateId);
    if (!update) return;

    if (debug) {
      console.log('Rolling back optimistic update:', updateId, update.rollbackData);
    }

    // Remove the update to trigger rollback
    removeOptimisticUpdate(updateId);
  }, [optimisticUpdates, removeOptimisticUpdate, debug]);

  // Apply optimistic update
  const applyOptimisticUpdate = useCallback((
    type: OptimisticUpdate<TUser>['type'],
    optimisticData: OptimisticUpdate<TUser>['optimisticData'],
    rollbackData?: OptimisticUpdate<TUser>['rollbackData']
  ) => {
    const updateId = generateUpdateId();
    
    const update: OptimisticUpdate<TUser> = {
      id: updateId,
      type,
      timestamp: Date.now(),
      optimisticData,
      rollbackData: rollbackData || {
        user: serverState.user,
        permissions: serverState.permissions,
        roles: serverState.roles,
        profiles: serverState.profiles,
      },
    };

    startTransition(() => {
      setOptimisticUpdates(prev => {
        // Remove oldest updates if we exceed maxUpdates
        const newUpdates = [...prev, update];
        if (newUpdates.length > maxUpdates) {
          const removed = newUpdates.splice(0, newUpdates.length - maxUpdates);
          removed.forEach(removedUpdate => clearUpdateTimeout(removedUpdate.id));
        }
        return newUpdates;
      });
    });

    // Set rollback timeout
    if (rollbackTimeout > 0) {
      const timeout = setTimeout(() => {
        rollbackOptimisticUpdate(updateId);
      }, rollbackTimeout);
      
      timeoutRefs.current.set(updateId, timeout);
    }

    if (debug) {
      console.log('Applied optimistic update:', updateId, optimisticData);
    }

    return updateId;
  }, [
    generateUpdateId,
    serverState,
    maxUpdates,
    rollbackTimeout,
    debug,
    clearUpdateTimeout,
    rollbackOptimisticUpdate,
    startTransition,
  ]);

  // Confirm optimistic update (prevents rollback)
  const confirmOptimisticUpdate = useCallback((updateId: string) => {
    const update = optimisticUpdates.find(u => u.id === updateId);
    if (!update) return;

    if (debug) {
      console.log('Confirmed optimistic update:', updateId);
    }

    removeOptimisticUpdate(updateId);
  }, [optimisticUpdates, removeOptimisticUpdate, debug]);

  // Optimistic login
  const optimisticLogin = useCallback(async (
    user: TUser,
    loginAction: () => Promise<void>
  ) => {
    const updateId = applyOptimisticUpdate('login', {
      user,
      permissions: user.permissions || [],
      roles: [user.isAdmin ? 'admin' : 'user'],
      profiles: [],
    });

    try {
      await loginAction();
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [applyOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  // Optimistic logout
  const optimisticLogout = useCallback(async (
    logoutAction: () => Promise<void>
  ) => {
    const updateId = applyOptimisticUpdate('logout', {
      user: null,
      permissions: [],
      roles: [],
      profiles: [],
    });

    try {
      await logoutAction();
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [applyOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  // Optimistic user update
  const optimisticUserUpdate = useCallback(async (
    userUpdates: Partial<TUser>,
    updateAction: () => Promise<TUser>
  ) => {
    const optimisticUser = { ...optimisticState.user, ...userUpdates } as TUser;
    
    const updateId = applyOptimisticUpdate('user_update', {
      user: optimisticUser,
    });

    try {
      const updatedUser = await updateAction();
      
      // Apply the real server response
      applyOptimisticUpdate('user_update', {
        user: updatedUser,
      });
      
      confirmOptimisticUpdate(updateId);
      return updatedUser;
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [
    optimisticState.user,
    applyOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate,
  ]);

  // Optimistic permissions update
  const optimisticPermissionsUpdate = useCallback(async (
    newPermissions: string[],
    updateAction: () => Promise<string[]>
  ) => {
    const updateId = applyOptimisticUpdate('permissions_update', {
      permissions: newPermissions,
    });

    try {
      const updatedPermissions = await updateAction();
      
      // Apply the real server response
      applyOptimisticUpdate('permissions_update', {
        permissions: updatedPermissions,
      });
      
      confirmOptimisticUpdate(updateId);
      return updatedPermissions;
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [applyOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  // Clear all optimistic updates
  const clearAllOptimisticUpdates = useCallback(() => {
    optimisticUpdates.forEach(update => clearUpdateTimeout(update.id));
    setOptimisticUpdates([]);
    
    if (debug) {
      console.log('Cleared all optimistic updates');
    }
  }, [optimisticUpdates, clearUpdateTimeout, debug]);

  // Get pending update count
  const pendingUpdateCount = optimisticUpdates.length;
  const hasPendingUpdates = pendingUpdateCount > 0;

  return {
    // Current state (with optimistic updates applied)
    user: optimisticState.user,
    permissions: optimisticState.permissions,
    roles: optimisticState.roles,
    profiles: optimisticState.profiles,
    isAuthenticated: optimisticState.isAuthenticated,
    isLoading: optimisticState.isLoading || isPending,
    
    // Optimistic update methods
    optimisticLogin,
    optimisticLogout,
    optimisticUserUpdate,
    optimisticPermissionsUpdate,
    
    // Update management
    applyOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate,
    clearAllOptimisticUpdates,
    
    // State info
    pendingUpdateCount,
    hasPendingUpdates,
    optimisticUpdates: debug ? optimisticUpdates : undefined,
  };
}