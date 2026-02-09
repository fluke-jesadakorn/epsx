'use client';

import * as React from 'react';
import { cn } from '@/lib/utils';

interface CollapsibleContextType {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const CollapsibleContext = React.createContext<CollapsibleContextType | undefined>(undefined);

interface CollapsibleProps {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: React.ReactNode;
  className?: string;
}

const Collapsible = React.forwardRef<HTMLDivElement, CollapsibleProps>(
  ({ open, defaultOpen = false, onOpenChange, children, className, ...props }, ref) => {
    const [internalOpen, setInternalOpen] = React.useState(defaultOpen);
    const currentOpen = open ?? internalOpen;
    
    const handleOpenChange = React.useCallback((newOpen: boolean) => {
      if (open === undefined) {
        setInternalOpen(newOpen);
      }
      onOpenChange?.(newOpen);
    }, [open, onOpenChange]);

    return (
      <CollapsibleContext.Provider value={{ open: currentOpen, onOpenChange: handleOpenChange }}>
        <div ref={ref} className={cn('', className)} {...props}>
          {children}
        </div>
      </CollapsibleContext.Provider>
    );
  }
);
Collapsible.displayName = 'Collapsible';

const CollapsibleTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement>
>(({ className, children, onClick, ...props }, ref) => {
  const context = React.useContext(CollapsibleContext);
  
  if (!context) {
    throw new Error('CollapsibleTrigger must be used within a Collapsible component');
  }

  const { open, onOpenChange } = context;

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    onOpenChange(!open);
    onClick?.(event);
  };

  return (
    <button
      ref={ref}
      className={cn(
        'flex w-full items-center justify-between py-4 font-medium transition-all hover:underline [&[data-state=open]>svg]:rotate-180',
        className
      )}
      onClick={handleClick}
      data-state={open ? 'open' : 'closed'}
      {...props}
    >
      {children}
    </button>
  );
});
CollapsibleTrigger.displayName = 'CollapsibleTrigger';

const CollapsibleContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, children, ...props }, ref) => {
  const context = React.useContext(CollapsibleContext);
  
  if (!context) {
    throw new Error('CollapsibleContent must be used within a Collapsible component');
  }

  const { open } = context;

  return (
    <div
      ref={ref}
      className={cn(
        'overflow-hidden text-sm transition-all',
        open ? 'animate-collapsible-down' : 'animate-collapsible-up',
        !open && 'hidden',
        className
      )}
      {...props}
    >
      <div className="pb-4 pt-0">
        {children}
      </div>
    </div>
  );
});
CollapsibleContent.displayName = 'CollapsibleContent';

export { Collapsible, CollapsibleTrigger, CollapsibleContent };
