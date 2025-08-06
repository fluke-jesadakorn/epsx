'use client';

import { createContext, useContext, useEffect, ReactNode } from 'react';
import { performanceMonitor as _performanceMonitor, usePerformanceMonitoring } from '@/lib/monitoring';

interface PerformanceContextType {
  recordMetric: (name: string, value: number, metadata?: Record<string, any>) => void;
  startTiming: (identifier: string) => void;
  endTiming: (identifier: string) => number | null;
  getMetrics: () => any[];
  exportMetrics: () => string;
}

const PerformanceContext = createContext<PerformanceContextType | null>(null);

export function PerformanceProvider({ children }: { children: ReactNode }) {
  const monitoring = usePerformanceMonitoring();

  useEffect(() => {
    // Record initial page load metrics
    if (typeof window !== 'undefined') {
      // Track initial load time
      const loadTime = performance.now();
      monitoring.recordMetric('page.initial-load', loadTime, {
        category: 'performance',
        page: window.location.pathname,
      });

      // Track time to interactive
      let interactionTimer: NodeJS.Timeout;
      
      const trackInteraction = () => {
        clearTimeout(interactionTimer);
        interactionTimer = setTimeout(() => {
          const ttiTime = performance.now();
          monitoring.recordMetric('page.time-to-interactive', ttiTime, {
            category: 'performance',
            page: window.location.pathname,
          });
        }, 100);
      };

      // Listen for user interactions
      const events = ['click', 'keydown', 'touchstart', 'scroll'];
      events.forEach(event => {
        document.addEventListener(event, trackInteraction, { once: true, passive: true });
      });

      return () => {
        clearTimeout(interactionTimer);
        events.forEach(event => {
          document.removeEventListener(event, trackInteraction);
        });
      };
    }
  }, [monitoring]);

  return (
    <PerformanceContext.Provider value={monitoring}>
      {children}
    </PerformanceContext.Provider>
  );
}

export function usePerformance(): PerformanceContextType {
  const context = useContext(PerformanceContext);
  if (!context) {
    throw new Error('usePerformance must be used within a PerformanceProvider');
  }
  return context;
}