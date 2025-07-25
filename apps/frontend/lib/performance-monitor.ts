'use client';

/**
 * Performance monitoring utilities for client hydration optimization
 */

interface PerformanceMetrics {
  hydrationTime: number;
  firstContentfulPaint: number;
  largestContentfulPaint: number;
  cumulativeLayoutShift: number;
  firstInputDelay: number;
  totalBlockingTime: number;
}

class PerformanceMonitor {
  private metrics: Partial<PerformanceMetrics> = {};
  private observers: Map<string, PerformanceObserver> = new Map();

  constructor() {
    if (typeof window !== 'undefined') {
      this.initializeObservers();
      this.measureHydrationTime();
    }
  }

  private initializeObservers() {
    // Measure Core Web Vitals
    this.observeLCP();
    this.observeFID();
    this.observeCLS();
    this.observeFCP();
    this.observeTBT();
  }

  private observeLCP() {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const lastEntry = entries[entries.length - 1] as PerformanceEntry;
      this.metrics.largestContentfulPaint = lastEntry.startTime;
      this.reportMetric('LCP', lastEntry.startTime);
    });

    observer.observe({ type: 'largest-contentful-paint', buffered: true });
    this.observers.set('lcp', observer);
  }

  private observeFID() {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      entries.forEach((entry: any) => {
        this.metrics.firstInputDelay = entry.processingStart - entry.startTime;
        this.reportMetric('FID', entry.processingStart - entry.startTime);
      });
    });

    observer.observe({ type: 'first-input', buffered: true });
    this.observers.set('fid', observer);
  }

  private observeCLS() {
    let clsValue = 0;
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      entries.forEach((entry: any) => {
        if (!entry.hadRecentInput) {
          clsValue += entry.value;
        }
      });
      this.metrics.cumulativeLayoutShift = clsValue;
      this.reportMetric('CLS', clsValue);
    });

    observer.observe({ type: 'layout-shift', buffered: true });
    this.observers.set('cls', observer);
  }

  private observeFCP() {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      entries.forEach((entry) => {
        if (entry.name === 'first-contentful-paint') {
          this.metrics.firstContentfulPaint = entry.startTime;
          this.reportMetric('FCP', entry.startTime);
        }
      });
    });

    observer.observe({ type: 'paint', buffered: true });
    this.observers.set('fcp', observer);
  }

  private observeTBT() {
    const observer = new PerformanceObserver((list) => {
      let tbt = 0;
      const entries = list.getEntries();
      entries.forEach((entry: any) => {
        if (entry.duration > 50) {
          tbt += entry.duration - 50;
        }
      });
      this.metrics.totalBlockingTime = tbt;
      this.reportMetric('TBT', tbt);
    });

    observer.observe({ type: 'longtask', buffered: true });
    this.observers.set('tbt', observer);
  }

  private measureHydrationTime() {
    const hydrationStart = performance.now();
    
    // Measure when React hydration completes
    const checkHydration = () => {
      const hydrationTime = performance.now() - hydrationStart;
      this.metrics.hydrationTime = hydrationTime;
      this.reportMetric('Hydration', hydrationTime);
    };

    // Check after next tick to ensure hydration is complete
    setTimeout(checkHydration, 0);
  }

  private reportMetric(name: string, value: number) {
    if (process.env.NODE_ENV === 'development') {
      logger.performance(name, value);
    }

    // Report to analytics service in production
    if (process.env.NODE_ENV === 'production' && process.env.PERFORMANCE_MONITORING === 'true') {
      this.sendToAnalytics(name, value);
    }
  }

  private sendToAnalytics(metric: string, value: number) {
    // Send to your analytics service (e.g., Google Analytics, DataDog, etc.)
    if (typeof gtag !== 'undefined') {
      gtag('event', 'performance_metric', {
        metric_name: metric,
        metric_value: Math.round(value),
        custom_map: { metric_1: metric }
      });
    }
  }

  // Public methods
  public getMetrics(): Partial<PerformanceMetrics> {
    return { ...this.metrics };
  }

  public markFeatureLoad(featureName: string) {
    performance.mark(`feature-${featureName}-start`);
  }

  public measureFeatureLoad(featureName: string) {
    performance.mark(`feature-${featureName}-end`);
    performance.measure(
      `feature-${featureName}`,
      `feature-${featureName}-start`,
      `feature-${featureName}-end`
    );

    const measure = performance.getEntriesByName(`feature-${featureName}`)[0];
    if (measure) {
      this.reportMetric(`Feature-${featureName}`, measure.duration);
    }
  }

  public measureComponentRender(componentName: string, renderFn: () => void) {
    const start = performance.now();
    renderFn();
    const end = performance.now();
    this.reportMetric(`Component-${componentName}`, end - start);
  }

  public dispose() {
    this.observers.forEach(observer => observer.disconnect());
    this.observers.clear();
  }
}

// Singleton instance
let performanceMonitor: PerformanceMonitor | null = null;

export function getPerformanceMonitor(): PerformanceMonitor {
  if (!performanceMonitor && typeof window !== 'undefined') {
    performanceMonitor = new PerformanceMonitor();
  }
  return performanceMonitor!;
}

// Hook for React components
export function usePerformanceMonitor() {
  const monitor = getPerformanceMonitor();
  
  return {
    markFeatureLoad: monitor?.markFeatureLoad.bind(monitor),
    measureFeatureLoad: monitor?.measureFeatureLoad.bind(monitor),
    measureComponentRender: monitor?.measureComponentRender.bind(monitor),
    getMetrics: monitor?.getMetrics.bind(monitor),
  };
}

// HOC for measuring component performance
export function withPerformanceMonitoring<T extends {}>(
  WrappedComponent: React.ComponentType<T>,
  componentName: string
) {
  return function PerformanceWrappedComponent(props: T) {
    const monitor = getPerformanceMonitor();
    
    React.useEffect(() => {
      monitor?.markFeatureLoad(componentName);
      return () => {
        monitor?.measureFeatureLoad(componentName);
      };
    }, []);

    return React.createElement(WrappedComponent, props);
  };
}