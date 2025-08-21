'use client';

import React, { 
  lazy, 
  Suspense, 
  useEffect, 
  useRef, 
  useState, 
  ComponentType,
  ReactNode 
} from 'react';

/**
 * Mobile Performance Optimization Utilities
 * 
 * This module provides utilities for optimizing performance specifically for mobile devices:
 * - Lazy loading with Intersection Observer
 * - Touch-optimized interactions
 * - Mobile-specific component loading
 * - Performance monitoring hooks
 */

interface LazyLoadWrapperProps {
  children: ReactNode;
  fallback?: ReactNode;
  threshold?: number;
  rootMargin?: string;
  className?: string;
}

/**
 * Lazy load wrapper that uses Intersection Observer for mobile performance
 */
export function LazyLoadWrapper({
  children,
  fallback = <MobileSkeleton />,
  threshold = 0.1,
  rootMargin = '50px',
  className = '',
}: LazyLoadWrapperProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [hasLoaded, setHasLoaded] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasLoaded) {
          setIsVisible(true);
          setHasLoaded(true);
          observer.unobserve(entry.target);
        }
      },
      {
        threshold,
        rootMargin,
      }
    );

    const currentRef = ref.current;
    if (currentRef) {
      observer.observe(currentRef);
    }

    return () => {
      if (currentRef) {
        observer.unobserve(currentRef);
      }
    };
  }, [threshold, rootMargin, hasLoaded]);

  return (
    <div ref={ref} className={className}>
      {isVisible ? children : fallback}
    </div>
  );
}

/**
 * Mobile-optimized skeleton loading component
 */
export function MobileSkeleton() {
  return (
    <div className="animate-pulse space-y-4 p-4">
      <div className="h-4 bg-muted rounded w-3/4"></div>
      <div className="space-y-2">
        <div className="h-4 bg-muted rounded"></div>
        <div className="h-4 bg-muted rounded w-5/6"></div>
      </div>
      <div className="h-32 bg-muted rounded"></div>
    </div>
  );
}

/**
 * Touch-optimized button wrapper for better mobile interactions
 */
interface TouchOptimizedButtonProps {
  children: ReactNode;
  onClick?: () => void;
  className?: string;
  disabled?: boolean;
  hapticFeedback?: boolean;
}

export function TouchOptimizedButton({
  children,
  onClick,
  className = '',
  disabled = false,
  hapticFeedback = true,
}: TouchOptimizedButtonProps) {
  const handleTouchStart = () => {
    if (hapticFeedback && 'vibrate' in navigator) {
      navigator.vibrate(10); // Subtle haptic feedback
    }
  };

  const handleClick = () => {
    if (!disabled && onClick) {
      onClick();
    }
  };

  return (
    <button
      className={`
        touch-manipulation 
        select-none 
        transition-all 
        duration-150 
        active:scale-95 
        focus:outline-none 
        focus:ring-2 
        focus:ring-primary 
        focus:ring-offset-2
        disabled:opacity-50 
        disabled:cursor-not-allowed
        ${className}
      `}
      onTouchStart={handleTouchStart}
      onClick={handleClick}
      disabled={disabled}
      style={{
        minHeight: '44px', // Apple's recommended minimum touch target
        minWidth: '44px',
      }}
    >
      {children}
    </button>
  );
}

/**
 * Mobile-specific image component with lazy loading and optimization
 */
interface MobileImageProps {
  src: string;
  alt: string;
  className?: string;
  priority?: boolean;
  sizes?: string;
  onLoad?: () => void;
}

export function MobileImage({
  src,
  alt,
  className = '',
  priority = false,
  sizes = '(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw',
  onLoad,
}: MobileImageProps) {
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasError, setHasError] = useState(false);

  const handleLoad = () => {
    setIsLoaded(true);
    onLoad?.();
  };

  const handleError = () => {
    setHasError(true);
  };

  if (hasError) {
    return (
      <div className={`bg-muted flex items-center justify-center ${className}`}>
        <span className="text-muted-foreground text-sm">Image unavailable</span>
      </div>
    );
  }

  return (
    <div className={`relative overflow-hidden ${className}`}>
      {!isLoaded && (
        <div className="absolute inset-0 bg-muted animate-pulse" />
      )}
      <img
        src={src}
        alt={alt}
        loading={priority ? 'eager' : 'lazy'}
        decoding="async"
        sizes={sizes}
        onLoad={handleLoad}
        onError={handleError}
        className={`
          w-full h-full object-cover
          transition-opacity duration-300
          ${isLoaded ? 'opacity-100' : 'opacity-0'}
        `}
      />
    </div>
  );
}

/**
 * Performance monitoring hook for mobile devices
 */
export function usePerformanceMonitoring() {
  const [metrics, setMetrics] = useState({
    loadTime: 0,
    domContentLoaded: 0,
    firstContentfulPaint: 0,
    largestContentfulPaint: 0,
  });

  useEffect(() => {
    // Monitor performance metrics
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (entry.entryType === 'navigation') {
          const navEntry = entry as PerformanceNavigationTiming;
          setMetrics(prev => ({
            ...prev,
            loadTime: navEntry.loadEventEnd - navEntry.navigationStart,
            domContentLoaded: navEntry.domContentLoadedEventEnd - navEntry.navigationStart,
          }));
        } else if (entry.entryType === 'paint') {
          if (entry.name === 'first-contentful-paint') {
            setMetrics(prev => ({
              ...prev,
              firstContentfulPaint: entry.startTime,
            }));
          }
        } else if (entry.entryType === 'largest-contentful-paint') {
          setMetrics(prev => ({
            ...prev,
            largestContentfulPaint: entry.startTime,
          }));
        }
      }
    });

    // Observe different types of performance entries
    observer.observe({ entryTypes: ['navigation', 'paint', 'largest-contentful-paint'] });

    return () => observer.disconnect();
  }, []);

  return metrics;
}

/**
 * Hook to detect mobile device and connection quality
 */
export function useMobileOptimization() {
  const [isMobile, setIsMobile] = useState(false);
  const [connectionSpeed, setConnectionSpeed] = useState<'slow' | 'fast' | 'unknown'>('unknown');

  useEffect(() => {
    // Detect mobile device
    const checkMobile = () => {
      const userAgent = navigator.userAgent.toLowerCase();
      const mobileKeywords = ['mobile', 'android', 'iphone', 'ipad', 'ipod', 'blackberry', 'windows phone'];
      setIsMobile(mobileKeywords.some(keyword => userAgent.includes(keyword)));
    };

    // Detect connection speed
    const checkConnection = () => {
      if ('connection' in navigator) {
        const connection = (navigator as any).connection;
        if (connection) {
          const effectiveType = connection.effectiveType;
          setConnectionSpeed(effectiveType === '4g' ? 'fast' : 'slow');
        }
      }
    };

    checkMobile();
    checkConnection();

    // Listen for connection changes
    if ('connection' in navigator) {
      const connection = (navigator as any).connection;
      if (connection) {
        connection.addEventListener('change', checkConnection);
        return () => connection.removeEventListener('change', checkConnection);
      }
    }
  }, []);

  return { isMobile, connectionSpeed };
}

/**
 * Factory for creating mobile-optimized lazy components
 */
export function createMobileLazyComponent<T extends ComponentType<any>>(
  componentLoader: () => Promise<{ default: T }>,
  fallback?: ReactNode
) {
  const LazyComponent = lazy(componentLoader);

  return function MobileLazyComponent(props: Parameters<T>[0]) {
    return (
      <Suspense fallback={fallback || <MobileSkeleton />}>
        <LazyComponent {...props} />
      </Suspense>
    );
  };
}

/**
 * Preload component for critical resources
 */
export function PreloadCriticalResources() {
  useEffect(() => {
    // Preload critical fonts
    const preloadFont = (font: string) => {
      const link = document.createElement('link');
      link.rel = 'preload';
      link.as = 'font';
      link.href = font;
      link.type = 'font/woff2';
      link.crossOrigin = 'anonymous';
      document.head.appendChild(link);
    };

    // Preload critical images
    const preloadImage = (src: string) => {
      const link = document.createElement('link');
      link.rel = 'preload';
      link.as = 'image';
      link.href = src;
      document.head.appendChild(link);
    };

    // Example critical resources (adjust based on your app)
    preloadFont('/fonts/kanit-v15-latin-regular.woff2');
    preloadImage('/images/hero-background.webp');

    // Prefetch next page resources based on user behavior
    const prefetchNextPage = () => {
      const link = document.createElement('link');
      link.rel = 'prefetch';
      link.href = '/dashboard';
      document.head.appendChild(link);
    };

    // Prefetch after a delay to avoid blocking critical resources
    const timer = setTimeout(prefetchNextPage, 3000);

    return () => clearTimeout(timer);
  }, []);

  return null;
}