/**
 * Critical CSS Integration for Next.js
 * 
 * This module provides utilities to integrate critical CSS with Next.js
 * for optimal performance and Core Web Vitals scores.
 * 
 * Features:
 * - Route-specific critical CSS loading
 * - Common critical CSS for all pages
 * - Automatic caching and optimization
 * - TypeScript support
 * - SSR-compatible
 */

import { readFileSync } from 'fs';
import { join } from 'path';
import { cache } from 'react';

// ============================================================================
// TYPES
// ============================================================================

export interface CriticalCSSConfig {
  route?: string;
  inline?: boolean;
  preload?: boolean;
  priority?: 'high' | 'normal' | 'low';
}

export interface CriticalCSSMetrics {
  size: number;
  loadTime: number;
  route: string;
  timestamp: number;
}

// ============================================================================
// CACHE MANAGEMENT
// ============================================================================

const criticalCSSCache = new Map<string, string>();
const metricsCache = new Map<string, CriticalCSSMetrics>();

/**
 * Clear the critical CSS cache
 */
export function clearCriticalCSSCache(): void {
  criticalCSSCache.clear();
  metricsCache.clear();
}

/**
 * Get cache statistics
 */
export function getCacheStats() {
  return {
    entries: criticalCSSCache.size,
    totalSize: Array.from(criticalCSSCache.values()).reduce((sum, css) => sum + css.length, 0),
    routes: Array.from(criticalCSSCache.keys()),
  };
}

// ============================================================================
// CSS LOADING UTILITIES
// ============================================================================

/**
 * Load critical CSS for a specific route (cached)
 */
export const getCriticalCSS = cache((route: string): string => {
  const cacheKey = `route-${route}`;
  
  if (criticalCSSCache.has(cacheKey)) {
    return criticalCSSCache.get(cacheKey)!;
  }
  
  try {
    const startTime = performance.now();
    const filePath = join(process.cwd(), 'styles', 'critical', `critical-${route}.css`);
    const css = readFileSync(filePath, 'utf8');
    const loadTime = performance.now() - startTime;
    
    // Cache the CSS
    criticalCSSCache.set(cacheKey, css);
    
    // Store metrics
    metricsCache.set(cacheKey, {
      size: css.length,
      loadTime,
      route,
      timestamp: Date.now(),
    });
    
    return css;
  } catch (error) {
    if (process.env.NODE_ENV === 'development') {
      console.warn(`Could not load critical CSS for route: ${route}`, error);
    }
    return '';
  }
});

/**
 * Load common critical CSS (cached)
 */
export const getCommonCriticalCSS = cache((): string => {
  const cacheKey = 'common';
  
  if (criticalCSSCache.has(cacheKey)) {
    return criticalCSSCache.get(cacheKey)!;
  }
  
  try {
    const startTime = performance.now();
    const filePath = join(process.cwd(), 'styles', 'critical', 'common.css');
    const css = readFileSync(filePath, 'utf8');
    const loadTime = performance.now() - startTime;
    
    // Cache the CSS
    criticalCSSCache.set(cacheKey, css);
    
    // Store metrics
    metricsCache.set(cacheKey, {
      size: css.length,
      loadTime,
      route: 'common',
      timestamp: Date.now(),
    });
    
    return css;
  } catch (error) {
    if (process.env.NODE_ENV === 'development') {
      console.warn('Could not load common critical CSS', error);
    }
    return '';
  }
});

/**
 * Get critical CSS for multiple routes
 */
export function getCriticalCSSForRoutes(routes: string[]): string {
  const cssArray = routes.map(route => getCriticalCSS(route)).filter(Boolean);
  return cssArray.join('\n');
}

// ============================================================================
// REACT COMPONENTS
// ============================================================================

/**
 * Critical CSS component that inlines critical styles
 */
export function CriticalCSS({ route, inline = true, priority = 'high' }: CriticalCSSConfig) {
  const commonCSS = getCommonCriticalCSS();
  const routeCSS = route ? getCriticalCSS(route) : '';
  
  const combinedCSS = [commonCSS, routeCSS].filter(Boolean).join('\n');
  
  if (!combinedCSS) {
    return null;
  }
  
  if (inline) {
    return (
      <style
        dangerouslySetInnerHTML={{ __html: combinedCSS }}
        data-critical-css
        data-route={route || 'common'}
        data-priority={priority}
        data-size={combinedCSS.length}
      />
    );
  }
  
  // For non-inline, create a data URL or external reference
  const dataUrl = `data:text/css;charset=utf-8,${encodeURIComponent(combinedCSS)}`;
  
  return (
    <link
      rel="stylesheet"
      href={dataUrl}
      data-critical-css
      data-route={route || 'common'}
      data-priority={priority}
    />
  );
}

/**
 * Preload critical CSS for better performance
 */
export function PreloadCriticalCSS({ routes }: { routes: string[] }) {
  return (
    <>
      {routes.map(route => {
        const css = getCriticalCSS(route);
        if (!css) return null;
        
        const dataUrl = `data:text/css;charset=utf-8,${encodeURIComponent(css)}`;
        
        return (
          <link
            key={route}
            rel="preload"
            as="style"
            href={dataUrl}
            data-route={route}
            onLoad={(e) => {
              // Convert preload to stylesheet after load
              const link = e.target as HTMLLinkElement;
              link.rel = 'stylesheet';
            }}
          />
        );
      })}
    </>
  );
}

/**
 * Critical CSS with fallback for progressive enhancement
 */
export function CriticalCSSWithFallback({ 
  route, 
  fallbackHref 
}: { 
  route?: string; 
  fallbackHref: string;
}) {
  const criticalCSS = route ? getCriticalCSS(route) : getCommonCriticalCSS();
  
  return (
    <>
      {/* Inline critical CSS for immediate render */}
      {criticalCSS && (
        <style
          dangerouslySetInnerHTML={{ __html: criticalCSS }}
          data-critical-css
        />
      )}
      
      {/* Fallback external stylesheet */}
      <link
        rel="stylesheet"
        href={fallbackHref}
        media="print"
        onLoad={(e) => {
          // Load the full stylesheet after critical CSS is rendered
          const link = e.target as HTMLLinkElement;
          link.media = 'all';
        }}
      />
      
      {/* No-JS fallback */}
      <noscript>
        <link rel="stylesheet" href={fallbackHref} />
      </noscript>
    </>
  );
}

// ============================================================================
// PERFORMANCE UTILITIES
// ============================================================================

/**
 * Get performance metrics for critical CSS
 */
export function getCriticalCSSMetrics(route?: string): CriticalCSSMetrics | null {
  const key = route ? `route-${route}` : 'common';
  return metricsCache.get(key) || null;
}

/**
 * Get all performance metrics
 */
export function getAllCriticalCSSMetrics(): CriticalCSSMetrics[] {
  return Array.from(metricsCache.values());
}

/**
 * Calculate potential performance improvement
 */
export function calculatePerformanceGains(route: string): {
  criticalSize: number;
  estimatedFCPImprovement: number;
  estimatedLCPImprovement: number;
} {
  const metrics = getCriticalCSSMetrics(route);
  
  if (!metrics) {
    return {
      criticalSize: 0,
      estimatedFCPImprovement: 0,
      estimatedLCPImprovement: 0,
    };
  }
  
  // Rough estimates based on typical performance gains
  const criticalSize = metrics.size;
  const estimatedFCPImprovement = Math.min(criticalSize / 1000 * 50, 500); // ~50ms per KB
  const estimatedLCPImprovement = Math.min(criticalSize / 1000 * 30, 300); // ~30ms per KB
  
  return {
    criticalSize,
    estimatedFCPImprovement,
    estimatedLCPImprovement,
  };
}

// ============================================================================
// DEVELOPMENT UTILITIES
// ============================================================================

/**
 * Development helper to analyze critical CSS usage
 */
export function analyzeCriticalCSS() {
  if (process.env.NODE_ENV !== 'development') {
    return null;
  }
  
  const metrics = getAllCriticalCSSMetrics();
  const cacheStats = getCacheStats();
  
  return {
    cache: cacheStats,
    metrics: metrics.map(m => ({
      route: m.route,
      size: `${(m.size / 1024).toFixed(2)} KB`,
      loadTime: `${m.loadTime.toFixed(2)}ms`,
      age: `${((Date.now() - m.timestamp) / 1000).toFixed(0)}s`,
    })),
    recommendations: generateRecommendations(metrics),
  };
}

/**
 * Generate performance recommendations
 */
function generateRecommendations(metrics: CriticalCSSMetrics[]): string[] {
  const recommendations: string[] = [];
  
  metrics.forEach(metric => {
    if (metric.size > 50000) { // 50KB
      recommendations.push(`Route "${metric.route}" has large critical CSS (${(metric.size / 1024).toFixed(1)}KB). Consider optimization.`);
    }
    
    if (metric.loadTime > 10) {
      recommendations.push(`Route "${metric.route}" has slow critical CSS loading (${metric.loadTime.toFixed(1)}ms). Check file system performance.`);
    }
  });
  
  const totalSize = metrics.reduce((sum, m) => sum + m.size, 0);
  if (totalSize > 200000) { // 200KB total
    recommendations.push('Total critical CSS size is large. Consider reducing overall CSS complexity.');
  }
  
  return recommendations;
}

/**
 * Debug component that shows critical CSS information
 */
export function CriticalCSSDebugInfo({ route }: { route?: string }) {
  if (process.env.NODE_ENV !== 'development') {
    return null;
  }
  
  const analysis = analyzeCriticalCSS();
  
  return (
    <details style={{ 
      position: 'fixed', 
      bottom: '10px', 
      right: '10px', 
      background: 'rgba(0,0,0,0.8)', 
      color: 'white', 
      padding: '10px',
      borderRadius: '5px',
      fontSize: '12px',
      zIndex: 9999,
      maxWidth: '400px'
    }}>
      <summary>Critical CSS Debug Info</summary>
      <pre style={{ whiteSpace: 'pre-wrap', fontSize: '10px' }}>
        {JSON.stringify(analysis, null, 2)}
      </pre>
    </details>
  );
}

// ============================================================================
// HOOKS
// ============================================================================

/**
 * Hook to get critical CSS for the current route
 */
export function useCriticalCSS(route?: string) {
  const commonCSS = getCommonCriticalCSS();
  const routeCSS = route ? getCriticalCSS(route) : '';
  const metrics = route ? getCriticalCSSMetrics(route) : null;
  
  return {
    commonCSS,
    routeCSS,
    combinedCSS: [commonCSS, routeCSS].filter(Boolean).join('\n'),
    metrics,
    analysis: analyzeCriticalCSS(),
  };
}

// ============================================================================
// EXPORTS
// ============================================================================

export {
  getCriticalCSS,
  getCommonCriticalCSS,
  getCriticalCSSForRoutes,
  CriticalCSS,
  PreloadCriticalCSS,
  CriticalCSSWithFallback,
  CriticalCSSDebugInfo,
  useCriticalCSS,
  clearCriticalCSSCache,
  getCacheStats,
  getCriticalCSSMetrics,
  getAllCriticalCSSMetrics,
  calculatePerformanceGains,
  analyzeCriticalCSS,
};