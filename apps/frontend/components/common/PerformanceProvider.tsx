'use client';

import { createContext, useContext, ReactNode } from 'react';

interface PerformanceContextType {
  recordMetric: (name: string, value: number, metadata?: Record<string, any>) => void;
  startTiming: (identifier: string) => void;
  endTiming: (identifier: string) => number | null;
  getMetrics: () => any[];
  exportMetrics: () => string;
}

const PerformanceContext = createContext<PerformanceContextType | null>(null);

export function PerformanceProvider({ children }: { children: ReactNode }) {
  // Performance monitoring disabled for better user experience
  // All functions are no-ops to maintain compatibility
  const monitoring: PerformanceContextType = {
    recordMetric: () => {},
    startTiming: () => {},
    endTiming: () => null,
    getMetrics: () => [],
    exportMetrics: () => '{}'
  };

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