/**
 * Simplified Action Component
 * No authentication checks - just executes actions directly
 */
'use client';

import { useState } from 'react';

interface JustInTimeAuthProps {
  /**
   * Action to execute when clicked
   */
  onAuthenticated: () => void | Promise<void>;
  
  /**
   * The trigger element (button, link, etc.)
   */
  children: React.ReactNode | ((props: {
    onClick: () => void;
    disabled: boolean;
    isExecuting: boolean;
  }) => React.ReactNode);
}

export function JustInTimeAuth({
  onAuthenticated,
  children,
}: JustInTimeAuthProps) {
  const [isExecuting, setIsExecuting] = useState(false);

  const handleClick = async () => {
    // Execute action directly - no authentication checks
    setIsExecuting(true);
    try {
      await onAuthenticated();
    } finally {
      setIsExecuting(false);
    }
  };

  // Clone children and add onClick handler
  const triggerElement = typeof children === 'function' 
    ? children({ 
        onClick: handleClick, 
        disabled: isExecuting,
        isExecuting
      })
    : <div onClick={handleClick} style={{ cursor: 'pointer' }}>{children}</div>;

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