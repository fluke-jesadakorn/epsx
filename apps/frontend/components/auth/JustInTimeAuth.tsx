/**
 * Just-In-Time Authentication Component
 * Triggers authentication only when user attempts a specific action
 */
'use client';

import { useState } from 'react';
import { useProgressiveAuth } from '@/hooks/useProgressiveAuth';
import { WalletConnectAuth } from './WalletConnectAuth';
import { AuthLevel, AuthLevelType } from '@/types/progressive-auth';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Wallet, Shield, AlertCircle } from 'lucide-react';

interface JustInTimeAuthProps {
  /**
   * Authentication level required for this action
   */
  requiredLevel: AuthLevelType;
  
  /**
   * Action to execute once authentication is successful
   */
  onAuthenticated: () => void | Promise<void>;
  
  /**
   * Name of the action for better UX messaging
   */
  actionName: string;
  
  /**
   * The trigger element (button, link, etc.)
   */
  children: React.ReactNode | ((props: {
    onClick: () => void;
    disabled: boolean;
    isAuthRequired: boolean;
    isExecuting: boolean;
  }) => React.ReactNode);
  
  /**
   * Whether to show the trigger as disabled when auth is required
   */
  disableWhenAuthRequired?: boolean;
  
  /**
   * Custom auth prompt message
   */
  authMessage?: string;
}

export function JustInTimeAuth({
  requiredLevel,
  onAuthenticated,
  actionName,
  children,
  disableWhenAuthRequired = false,
  authMessage
}: JustInTimeAuthProps) {
  const auth = useProgressiveAuth();
  const [showAuthModal, setShowAuthModal] = useState(false);
  const [isExecuting, setIsExecuting] = useState(false);

  const handleClick = async () => {
    // If user has sufficient auth, execute action immediately
    if (auth.canAccess(requiredLevel)) {
      setIsExecuting(true);
      try {
        await onAuthenticated();
      } finally {
        setIsExecuting(false);
      }
      return;
    }

    // Otherwise, show auth modal
    setShowAuthModal(true);
  };

  const handleAuthSuccess = async () => {
    setShowAuthModal(false);
    
    // Execute the action after successful authentication
    setIsExecuting(true);
    try {
      await onAuthenticated();
    } finally {
      setIsExecuting(false);
    }
  };

  const message = authMessage || auth.getAuthMessage(requiredLevel, actionName);
  const upgradeAction = auth.getUpgradeAction(requiredLevel);
  const isAuthRequired = !auth.canAccess(requiredLevel);

  // Clone children and add onClick handler
  const triggerElement = typeof children === 'function' 
    ? children({ 
        onClick: handleClick, 
        disabled: (disableWhenAuthRequired && isAuthRequired) || isExecuting,
        isAuthRequired,
        isExecuting
      })
    : <div onClick={handleClick} style={{ cursor: 'pointer' }}>{children}</div>;

  return (
    <>
      {triggerElement}
      
      <Dialog open={showAuthModal} onOpenChange={setShowAuthModal}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <div className="flex items-center gap-3 mb-2">
              {requiredLevel === AuthLevel.AUTHENTICATED ? (
                <Shield className="h-6 w-6 text-orange-500" />
              ) : (
                <Wallet className="h-6 w-6 text-orange-500" />
              )}
              <DialogTitle>Authentication Required</DialogTitle>
            </div>
            <DialogDescription>
              {message}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            {/* Show current status */}
            <div className="flex items-center gap-2 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <AlertCircle className="h-4 w-4 text-slate-500" />
              <div className="text-sm">
                <div className="font-medium">
                  Current: {auth.level === AuthLevel.PUBLIC ? 'Not Connected' : 
                           auth.level === AuthLevel.CONNECTED ? 'Wallet Connected' : 'Authenticated'}
                </div>
                <div className="text-slate-500">
                  Required: {requiredLevel === AuthLevel.CONNECTED ? 'Wallet Connection' : 'Full Authentication'}
                </div>
              </div>
            </div>

            {/* Authentication component */}
            <div className="space-y-3">
              {upgradeAction === 'connect' && (
                <div className="space-y-2">
                  <p className="text-sm text-slate-600 dark:text-slate-400">
                    Connect your wallet to continue
                  </p>
                  <WalletConnectAuth 
                    className="w-full"
                    onAuthSuccess={handleAuthSuccess}
                  />
                </div>
              )}

              {upgradeAction === 'sign' && (
                <div className="space-y-2">
                  <p className="text-sm text-slate-600 dark:text-slate-400">
                    Sign in with your wallet to continue
                  </p>
                  <WalletConnectAuth 
                    className="w-full"
                    onAuthSuccess={handleAuthSuccess}
                  />
                </div>
              )}

              {upgradeAction === 'connect_and_sign' && (
                <div className="space-y-2">
                  <p className="text-sm text-slate-600 dark:text-slate-400">
                    Connect your wallet and sign in to continue
                  </p>
                  <WalletConnectAuth 
                    className="w-full"
                    onAuthSuccess={handleAuthSuccess}
                  />
                </div>
              )}
            </div>

            {/* Cancel button */}
            <div className="flex justify-end">
              <Button 
                variant="ghost" 
                onClick={() => setShowAuthModal(false)}
                className="text-sm"
              >
                Cancel
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}

/**
 * Higher-order component version for wrapping buttons/links
 */
export function withJustInTimeAuth<P extends object>(
  Component: React.ComponentType<P>,
  authConfig: {
    requiredLevel: AuthLevelType;
    actionName: string;
    authMessage?: string;
  }
) {
  return function WrappedComponent(props: P & { 
    onAuthenticated: () => void | Promise<void>;
    disableWhenAuthRequired?: boolean;
  }) {
    const { onAuthenticated, disableWhenAuthRequired, ...componentProps } = props;
    
    return (
      <JustInTimeAuth
        requiredLevel={authConfig.requiredLevel}
        actionName={authConfig.actionName}
        authMessage={authConfig.authMessage}
        onAuthenticated={onAuthenticated}
        disableWhenAuthRequired={disableWhenAuthRequired}
      >
        {({ onClick, disabled }) => (
          <Component 
            {...(componentProps as P)}
            onClick={onClick}
            disabled={disabled}
          />
        )}
      </JustInTimeAuth>
    );
  };
}