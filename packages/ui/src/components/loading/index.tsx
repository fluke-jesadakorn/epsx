"use client";

import * as React from "react";
import { cn } from "../../lib/utils";

interface LoadingProps {
  size?: 'sm' | 'md' | 'lg';
  className?: string;
  text?: string;
}

export function Loading({ size = 'md', className, text }: LoadingProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  };

  return (
    <div className={cn('flex flex-col items-center justify-center', className)}>
      <div
        className={cn(
          'animate-spin rounded-full border-2 border-primary border-t-transparent',
          sizeClasses[size]
        )}
        role="status"
        aria-label="Loading"
      />
      {text && <p className="mt-2 text-sm text-muted-foreground">{text}</p>}
    </div>
  );
}

export function LoadingSpinner({ size = 'md', className }: Omit<LoadingProps, 'text'>) {
  return <Loading size={size} className={className} />;
}

export function LoadingCard({ className }: { className?: string }) {
  return (
    <div className={cn('rounded-lg border bg-card p-6', className)}>
      <div className="space-y-3">
        <div className="h-4 w-2/3 bg-muted animate-pulse rounded" />
        <div className="h-4 w-full bg-muted animate-pulse rounded" />
        <div className="h-4 w-1/2 bg-muted animate-pulse rounded" />
      </div>
    </div>
  );
}

export function LoadingTable({ rows = 5, className }: { rows?: number; className?: string }) {
  return (
    <div className={cn("space-y-2", className)}>
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="flex items-center space-x-4">
          <div className="h-4 w-4 bg-muted animate-pulse rounded" />
          <div className="h-4 flex-1 bg-muted animate-pulse rounded" />
          <div className="h-4 w-20 bg-muted animate-pulse rounded" />
        </div>
      ))}
    </div>
  );
}

export function LoadingForm({ className }: { className?: string }) {
  return (
    <div className={cn("space-y-4", className)}>
      <div className="space-y-2">
        <div className="h-4 w-20 bg-muted animate-pulse rounded" />
        <div className="h-10 w-full bg-muted animate-pulse rounded" />
      </div>
      <div className="space-y-2">
        <div className="h-4 w-24 bg-muted animate-pulse rounded" />
        <div className="h-10 w-full bg-muted animate-pulse rounded" />
      </div>
      <div className="space-y-2">
        <div className="h-4 w-16 bg-muted animate-pulse rounded" />
        <div className="h-20 w-full bg-muted animate-pulse rounded" />
      </div>
      <div className="h-10 w-32 bg-muted animate-pulse rounded" />
    </div>
  );
}

// Optimized loading components for performance
export const OptimizedLoadingSpinner = React.memo(LoadingSpinner);
export const OptimizedLoading = React.memo(Loading);