/**
 * Collapsible UI Component
 * Provides collapsible content functionality
 */

'use client'

import * as React from 'react'

import { cn } from '@/lib/shared'

interface CollapsibleContextValue {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const CollapsibleContext = React.createContext<CollapsibleContextValue | null>(null)

function useCollapsible() {
  const context = React.useContext(CollapsibleContext)
  if (!context) {
    throw new Error('useCollapsible must be used within a Collapsible component')
  }
  return context
}

interface CollapsibleProps {
  children: React.ReactNode
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
  className?: string
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.open
 * @param root0.defaultOpen
 * @param root0.onOpenChange
 * @param root0.className
 */
export function Collapsible({ 
  children, 
  open: controlledOpen, 
  defaultOpen = false, 
  onOpenChange,
  className 
}: CollapsibleProps) {
  const [internalOpen, setInternalOpen] = React.useState(defaultOpen)
  
  const isControlled = controlledOpen !== undefined
  const open = isControlled ? controlledOpen : internalOpen
  
  const handleOpenChange = React.useCallback((newOpen: boolean) => {
    if (!isControlled) {
      setInternalOpen(newOpen)
    }
    onOpenChange?.(newOpen)
  }, [isControlled, onOpenChange])

  const contextValue = React.useMemo(() => ({
    open,
    onOpenChange: handleOpenChange
  }), [open, handleOpenChange])

  return (
    <CollapsibleContext.Provider value={contextValue}>
      <div className={cn('', className)}>
        {children}
      </div>
    </CollapsibleContext.Provider>
  )
}

interface CollapsibleTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode
  asChild?: boolean
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.asChild
 * @param root0.onClick
 */
export function CollapsibleTrigger({ 
  children, 
  asChild = false, 
  onClick, 
  ...props 
}: CollapsibleTriggerProps) {
  const { open, onOpenChange } = useCollapsible()

  const handleClick = React.useCallback((event: React.MouseEvent<HTMLButtonElement>) => {
    onOpenChange(!open)
    onClick?.(event)
  }, [open, onOpenChange, onClick])

  if (asChild) {
    // Clone the child element and add the click handler
    return React.cloneElement(children as React.ReactElement<any>, {
      onClick: handleClick,
      'aria-expanded': open,
      'data-state': open ? 'open' : 'closed'
    } as any)
  }

  return (
    <button
      onClick={handleClick}
      aria-expanded={open}
      data-state={open ? 'open' : 'closed'}
      {...props}
    >
      {children}
    </button>
  )
}

interface CollapsibleContentProps {
  children: React.ReactNode
  className?: string
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.className
 */
export function CollapsibleContent({ children, className }: CollapsibleContentProps) {
  const { open } = useCollapsible()
  if (!open) {
    return null
  }

  return (
    <div
      className={cn('overflow-hidden', className)}
      data-state={open ? 'open' : 'closed'}
    >
      {children}
    </div>
  )
}

// Export individual components for convenience
export { CollapsibleTrigger as Trigger, CollapsibleContent as Content }