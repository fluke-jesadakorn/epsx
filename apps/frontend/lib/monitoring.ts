'use client';

/**
 * SSR Performance Monitoring System
 * Tracks Core Web Vitals and SSR-specific metrics
 */

export interface PerformanceMetric {
  name: string;
  value: number;
  timestamp: number;
  metadata?: Record<string, any>;
}

export interface SSRMetrics {
  firstContentfulPaint: number;
  largestContentfulPaint: number;
  firstInputDelay: number;
  cumulativeLayoutShift: number;
  timeToInteractive: number;
  hydrationTime: number;
  ssrRenderTime: number;
}

class SSRPerformanceMonitor {
  private static instance: SSRPerformanceMonitor;
  private metrics: PerformanceMetric[] = [];
  private startTime: number = performance.now();
  private hydrationStartTime: number | null = null;

  public static getInstance(): SSRPerformanceMonitor {
    if (!SSRPerformanceMonitor.instance) {
      SSRPerformanceMonitor.instance = new SSRPerformanceMonitor();
    }
    return SSRPerformanceMonitor.instance;
  }

  constructor() {
    if (typeof window !== 'undefined') {
      this.initializeMonitoring();
    }
  }

  private initializeMonitoring(): void {
    // Track hydration start
    this.hydrationStartTime = performance.now();

    // Core Web Vitals
    this.observeWebVitals();

    // Navigation timing
    this.trackNavigationTiming();

    // Track hydration completion
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => {
        this.recordHydrationTime();
      });
    } else {
      this.recordHydrationTime();
    }

    // Resource timing
    this.observeResourceTiming();
  }

  private observeWebVitals(): void {
    // Using web-vitals pattern for Next.js
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        this.recordMetric(entry.name, entry.value, {
          entryType: entry.entryType,
          rating: this.getRating(entry.name, entry.value),
        });
      }
    });

    // Observe paint, layout-shift, first-input entries
    observer.observe({ 
      entryTypes: ['paint', 'layout-shift', 'first-input', 'largest-contentful-paint'] 
    });
  }

  private trackNavigationTiming(): void {
    if (typeof window === 'undefined') return;

    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    if (!navigation) return;

    // SSR-specific metrics
    const ssrMetrics = {
      dns: navigation.domainLookupEnd - navigation.domainLookupStart,
      tcp: navigation.connectEnd - navigation.connectStart,
      request: navigation.responseStart - navigation.requestStart,
      response: navigation.responseEnd - navigation.responseStart,
      processing: navigation.domComplete - navigation.responseEnd,
      onload: navigation.loadEventEnd - navigation.loadEventStart,
    };

    Object.entries(ssrMetrics).forEach(([name, value]) => {
      this.recordMetric(`navigation.${name}`, value, { category: 'navigation' });
    });
  }

  private recordHydrationTime(): void {
    if (this.hydrationStartTime) {
      const hydrationTime = performance.now() - this.hydrationStartTime;
      this.recordMetric('hydration.time', hydrationTime, {
        category: 'ssr',
        rating: hydrationTime < 1000 ? 'good' : hydrationTime < 2500 ? 'needs-improvement' : 'poor',
      });
    }
  }

  private observeResourceTiming(): void {
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        const resource = entry as PerformanceResourceTiming;
        
        // Track key resources
        if (resource.name.includes('.js') || resource.name.includes('.css')) {
          this.recordMetric('resource.load', resource.duration, {
            resource: resource.name,
            type: resource.name.includes('.js') ? 'javascript' : 'stylesheet',
            size: resource.transferSize,
            cached: resource.transferSize === 0,
          });
        }
      }
    });

    observer.observe({ entryTypes: ['resource'] });
  }

  private getRating(metric: string, value: number): 'good' | 'needs-improvement' | 'poor' {
    const thresholds: Record<string, [number, number]> = {
      'first-contentful-paint': [1800, 3000],
      'largest-contentful-paint': [2500, 4000],
      'first-input-delay': [100, 300],
      'cumulative-layout-shift': [0.1, 0.25],
    };

    const threshold = thresholds[metric];
    if (!threshold) return 'good';

    if (value <= threshold[0]) return 'good';
    if (value <= threshold[1]) return 'needs-improvement';
    return 'poor';
  }

  public recordMetric(name: string, value: number, metadata?: Record<string, any>): void {
    const metric: PerformanceMetric = {
      name,
      value,
      timestamp: performance.now(),
      metadata,
    };

    this.metrics.push(metric);

    // Log performance issues in development
    if (process.env.NODE_ENV === 'development') {
      if (metadata?.rating === 'poor') {
        console.warn(`Poor performance: ${name} = ${value}ms`, metadata);
      }
    }

    // Send to analytics service (implement as needed)
    this.sendToAnalytics(metric);
  }

  public startTiming(identifier: string): void {
    performance.mark(`${identifier}-start`);
  }

  public endTiming(identifier: string): number | null {
    try {
      performance.mark(`${identifier}-end`);
      performance.measure(identifier, `${identifier}-start`, `${identifier}-end`);
      
      const measure = performance.getEntriesByName(identifier, 'measure')[0];
      if (measure) {
        this.recordMetric(`custom.${identifier}`, measure.duration, {
          category: 'custom',
        });
        return measure.duration;
      }
    } catch (error) {
      console.warn(`Failed to measure ${identifier}:`, error);
    }
    return null;
  }

  public getSSRMetrics(): Partial<SSRMetrics> {
    const getMetric = (name: string) => {
      const metric = this.metrics.find(m => m.name === name);
      return metric?.value || 0;
    };

    return {
      firstContentfulPaint: getMetric('first-contentful-paint'),
      largestContentfulPaint: getMetric('largest-contentful-paint'),
      firstInputDelay: getMetric('first-input-delay'),
      cumulativeLayoutShift: getMetric('cumulative-layout-shift'),
      hydrationTime: getMetric('hydration.time'),
      ssrRenderTime: getMetric('navigation.processing'),
    };
  }

  public getMetrics(): PerformanceMetric[] {
    return [...this.metrics];
  }

  public getMetricsByCategory(category: string): PerformanceMetric[] {
    return this.metrics.filter(m => m.metadata?.category === category);
  }

  private sendToAnalytics(metric: PerformanceMetric): void {
    // In production, send to your analytics service
    // For now, just store locally or log
    if (typeof window !== 'undefined' && window.gtag && process.env.NODE_ENV === 'production') {
      window.gtag('event', 'performance_metric', {
        custom_map: { metric_name: 'metric_name' },
        metric_name: metric.name,
        metric_value: metric.value,
        metric_rating: metric.metadata?.rating,
      });
    }
  }

  public exportMetrics(): string {
    return JSON.stringify({
      timestamp: Date.now(),
      url: typeof window !== 'undefined' ? window.location.href : '',
      userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '',
      metrics: this.metrics,
      summary: this.getSSRMetrics(),
    }, null, 2);
  }
}

// Global performance monitoring functions
export const performanceMonitor = typeof window !== 'undefined' 
  ? SSRPerformanceMonitor.getInstance() 
  : null;

export function recordMetric(name: string, value: number, metadata?: Record<string, any>): void {
  performanceMonitor?.recordMetric(name, value, metadata);
}

export function startTiming(identifier: string): void {
  performanceMonitor?.startTiming(identifier);
}

export function endTiming(identifier: string): number | null {
  return performanceMonitor?.endTiming(identifier) || null;
}

export function withPerformanceTracking<T extends (...args: any[]) => any>(
  fn: T,
  identifier: string
): T {
  return ((...args: any[]) => {
    startTiming(identifier);
    const result = fn(...args);
    
    if (result instanceof Promise) {
      return result.finally(() => endTiming(identifier));
    } else {
      endTiming(identifier);
      return result;
    }
  }) as T;
}

// React hook for performance monitoring
export function usePerformanceMonitoring() {
  if (typeof window === 'undefined') {
    return {
      recordMetric: () => {},
      startTiming: () => {},
      endTiming: () => null,
      getMetrics: () => [],
      exportMetrics: () => '{}',
    };
  }

  return {
    recordMetric,
    startTiming,
    endTiming,
    getMetrics: () => performanceMonitor?.getMetrics() || [],
    exportMetrics: () => performanceMonitor?.exportMetrics() || '{}',
  };
}

// Declare gtag for TypeScript
declare global {
  interface Window {
    gtag?: (...args: any[]) => void;
  }
}