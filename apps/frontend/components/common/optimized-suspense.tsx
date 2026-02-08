import { Suspense } from 'react';

interface OptimizedSuspenseProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  name?: string; // For debugging
}

/**
 * Optimized Suspense wrapper with performance-focused fallbacks
 * Uses text-based loading to comply with zero-animation policy
 */
export function OptimizedSuspense({ 
  children, 
  fallback,
  name = 'component' 
}: OptimizedSuspenseProps) {
  const defaultFallback = (
    <div className="flex items-center justify-center p-8 text-gray-600">
      <span>Loading {name}...</span>
    </div>
  );

  return (
    <Suspense fallback={fallback || defaultFallback}>
      {children}
    </Suspense>
  );
}