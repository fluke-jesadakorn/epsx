/**
 * Just-In-Time Authentication Component
 * Triggers wallet authentication when needed for protected actions
 * 
 * Usage:
 * <JustInTimeAuth onAuthenticated={() => performProtectedAction()}>
 *   <button>Protected Action</button>
 * </JustInTimeAuth>
 */
'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import { useConnectModal } from '@rainbow-me/rainbowkit';
import { useCallback, useEffect, useRef, useState } from 'react';

interface JustInTimeAuthProps {
  /**
   * Action to execute when user is authenticated
   */
  onAuthenticated: () => void | Promise<void>;

  /**
   * The trigger element (button, link, etc.)
   */
  children: React.ReactNode | ((props: {
    onClick: () => void;
    disabled: boolean;
    isExecuting: boolean;
    isAuthenticating: boolean;
  }) => React.ReactNode);

  /**
   * Custom message to show while authenticating
   */
  authMessage?: string;
}

export function JustInTimeAuth({
  onAuthenticated,
  children,
  authMessage: _authMessage,
}: JustInTimeAuthProps) {
  const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
  const { openConnectModal } = useConnectModal();
  const [isExecuting, setIsExecuting] = useState(false);
  const [pendingAction, setPendingAction] = useState(false);
  const pendingActionRef = useRef(false);

  // Execute pending action when user becomes authenticated
  useEffect(() => {
    if (pendingActionRef.current && isAuthenticated && !authLoading) {
      pendingActionRef.current = false;
      setPendingAction(false);

      // Execute the authenticated action
      const executeAction = async () => {
        setIsExecuting(true);
        try {
          await onAuthenticated();
        } finally {
          setIsExecuting(false);
        }
      };
      executeAction();
    }
  }, [isAuthenticated, authLoading, onAuthenticated]);

  const handleClick = useCallback(async () => {
    // If already authenticated, execute action directly
    if (isAuthenticated) {
      setIsExecuting(true);
      try {
        await onAuthenticated();
      } finally {
        setIsExecuting(false);
      }
      return;
    }

    // If not authenticated, open wallet connect modal and set pending action
    if (openConnectModal) {
      pendingActionRef.current = true;
      setPendingAction(true);
      openConnectModal();
    }
  }, [isAuthenticated, onAuthenticated, openConnectModal]);

  // Clone children and add onClick handler
  const triggerElement = typeof children === 'function'
    ? children({
      onClick: handleClick,
      disabled: isExecuting || authLoading,
      isExecuting,
      isAuthenticating: pendingAction && !isAuthenticated
    })
    : <div onClick={handleClick} style={{ cursor: isExecuting ? 'wait' : 'pointer' }}>{children}</div>;

  return triggerElement;
}

/**
 * Higher-order component version for wrapping buttons/links
 */
export function withJustInTimeAuth<P extends object>(
  Component: React.ComponentType<P>
) {
  return function WrappedComponent(props: P & {
    onAuthenticated: () => void | Promise<void>;
  }) {
    const { onAuthenticated, ...componentProps } = props;

    return (
      <JustInTimeAuth
        onAuthenticated={onAuthenticated}
      >
        {({ onClick, disabled, isExecuting, isAuthenticating }) => (
          <Component
            {...(componentProps as P)}
            onClick={onClick}
            disabled={disabled || isExecuting || isAuthenticating}
          />
        )}
      </JustInTimeAuth>
    );
  };
}

/**
 * Hook for just-in-time authentication
 * Can be used to programmatically trigger auth before an action
 */
export function useJustInTimeAuth() {
  const { isAuthenticated, isLoading } = useSharedAuth();
  const { openConnectModal } = useConnectModal();

  const requireAuth = useCallback(async (action: () => void | Promise<void>) => {
    if (isAuthenticated) {
      await action();
      return true;
    }

    if (openConnectModal) {
      openConnectModal();
    }
    return false;
  }, [isAuthenticated, openConnectModal]);

  return {
    isAuthenticated,
    isLoading,
    requireAuth,
    openConnectModal
  };
}