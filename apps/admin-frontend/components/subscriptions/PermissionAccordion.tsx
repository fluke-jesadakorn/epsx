/**
 * PermissionAccordion Component
 * 
 * Collapsible section for permission registry:
 * - Shows summary when collapsed (count + platforms)
 * - Expands to show the full PermissionRegistry component
 * - Lazy loads content on expand
 */
'use client';

import { ChevronRight, Key, Settings } from 'lucide-react';
import { useState } from 'react';


import { cn } from '@/lib/utils';

// PermissionRegistry was removed as part of client-side permission cleanup.
// Real permissions are managed on the backend.
const PermissionRegistry = () => (
  <div className="p-8 text-center text-muted-foreground border-2 border-dashed border-border rounded-xl">
    Permission Management has moved to the unified backend authority.
  </div>
);


interface PermissionAccordionProps {
  count: number;
  platformCount: number;
  className?: string;
}

export function PermissionAccordion({ count, platformCount, className }: PermissionAccordionProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div className={cn('rounded-2xl border border-border overflow-hidden', className)}>
      {/* Collapsed Header / Toggle */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className={cn(
          'w-full flex items-center justify-between p-4 bg-card',
          'hover:bg-muted/50 transition-colors duration-200',
          'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-inset'
        )}
      >
        <div className="flex items-center gap-3">
          <div className={cn(
            'p-2 rounded-xl transition-colors duration-200',
            isExpanded ? 'bg-purple-500/20' : 'bg-muted'
          )}>
            <Key className={cn(
              'h-5 w-5 transition-colors duration-200',
              isExpanded ? 'text-purple-500' : 'text-muted-foreground'
            )} />
          </div>
          <div className="text-left">
            <h2 className="text-base font-semibold text-foreground flex items-center gap-2">
              Permission Registry
              <span className="text-xs font-normal text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
                {count} definitions
              </span>
            </h2>
            <p className="text-xs text-muted-foreground">
              {platformCount} platforms • Define system-wide permissions
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {!isExpanded && (
            <span className="text-xs text-muted-foreground hidden sm:inline">
              Click to manage
            </span>
          )}
          <div className={cn(
            'p-1.5 rounded-lg bg-muted transition-all duration-200',
            isExpanded && 'bg-purple-500/10'
          )}>
            <ChevronRight className={cn(
              'h-4 w-4 text-muted-foreground transition-transform duration-200',
              isExpanded && 'rotate-90 text-purple-500'
            )} />
          </div>
        </div>
      </button>

      {/* Expanded Content */}
      {isExpanded && (
        <div className="border-t border-border bg-card/50">
          <div className="p-4 sm:p-6">
            <PermissionRegistry />
          </div>
        </div>
      )}


      {/* Quick Actions Bar (always visible) */}
      {!isExpanded && (
        <div className="border-t border-border bg-muted/30 px-4 py-2 flex items-center justify-between">
          <div className="flex items-center gap-4 text-xs text-muted-foreground">
            <span className="flex items-center gap-1">
              <Settings className="h-3 w-3" />
              System permissions are protected
            </span>
          </div>
          <button
            onClick={() => setIsExpanded(true)}
            className="text-xs text-purple-500 hover:text-purple-600 font-medium"
          >
            Expand to manage →
          </button>
        </div>
      )}
    </div>
  );
}

export default PermissionAccordion;
