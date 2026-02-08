import { Suspense } from 'react';
import { LoadingSpinner } from './loading-spinner';

interface OptimizedSuspenseBoundaryProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  identifier?: string;
}

export function OptimizedSuspenseBoundary({ 
  children, 
  fallback,
  identifier 
}: OptimizedSuspenseBoundaryProps) {
  const defaultFallback = (
    <div className="flex items-center justify-center p-8">
      <LoadingSpinner size="sm" />
      {identifier && (
        <span className="ml-2 text-sm text-muted-foreground">
          Loading {identifier}...
        </span>
      )}
    </div>
  );

  return (
    <Suspense fallback={fallback || defaultFallback}>
      {children}
    </Suspense>
  );
}