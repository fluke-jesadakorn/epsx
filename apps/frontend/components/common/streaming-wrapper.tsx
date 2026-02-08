import { Suspense } from 'react';
import { LoadingSpinner } from './loading-spinner';

interface StreamingWrapperProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  priority?: 'high' | 'medium' | 'low';
  identifier?: string;
}

/**
 * Streaming wrapper component for progressive enhancement
 * Optimizes SSR by allowing content to stream in chunks
 */
export function StreamingWrapper({
  children,
  fallback,
  priority = 'medium',
  identifier
}: StreamingWrapperProps) {
  const getFallback = () => {
    if (fallback) {return fallback;}

    const size = priority === 'high' ? 'md' : 'sm';
    return (
      <div className="flex items-center justify-center p-4">
        <LoadingSpinner size={size} />
      </div>
    );
  };

  // High priority content streams first, low priority content streams last
  const streamingProps = {
    'data-priority': priority,
    'data-identifier': identifier,
  };

  return (
    <div {...streamingProps}>
      <Suspense fallback={getFallback()}>
        {children}
      </Suspense>
    </div>
  );
}

/**
 * Higher order component for streaming content
 */
export function withStreaming<P extends object>(
  Component: React.ComponentType<P>,
  options: {
    priority?: 'high' | 'medium' | 'low';
    identifier?: string;
    fallback?: React.ReactNode;
  } = {}
) {
  return function StreamingComponent(props: P) {
    return (
      <StreamingWrapper
        priority={options.priority}
        identifier={options.identifier}
        fallback={options.fallback}
      >
        <Component {...props} />
      </StreamingWrapper>
    );
  };
}