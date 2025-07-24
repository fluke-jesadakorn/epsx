import { Suspense } from 'react';
import { OptimizedLoadingSpinner } from './OptimizedLoadingSpinner';

interface OptimizedSuspenseProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  name?: string; // For debugging
}

/**
 * Optimized Suspense wrapper with performance-focused fallbacks
 */
export function OptimizedSuspense({ 
  children, 
  fallback,
  name = 'component' 
}: OptimizedSuspenseProps) {
  const defaultFallback = (
    <div className="flex items-center justify-center p-8">
      <OptimizedLoadingSpinner size="md" />
      <span className="sr-only">Loading {name}...</span>
    </div>
  );

  return (
    <Suspense fallback={fallback || defaultFallback}>
      {children}
    </Suspense>
  );
}