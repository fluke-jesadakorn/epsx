/**
 * Bundle optimization utilities and configurations
 */

// Lazy load large libraries only when needed
export const lazyLoadLibraries = {
  // Charts - only load when analytics page is accessed
  chart: () => import('recharts'),
  
  // Date utilities - only for payment/subscription features
  date: () => import('date-fns'),
  
  // Form validation - only for complex forms
  validation: () => import('zod'),
  
  // PDF generation - only for reports
  pdf: () => import('jspdf'),
  
  // CSV export - only for data export features
  csv: () => import('papaparse'),
};

// Tree-shakable icon imports
export const lazyIcons = {
  // Only import icons when they're actually used
  loadIcon: (iconName: string) => {
    switch (iconName) {
      case 'chart':
        return import('lucide-react').then(mod => mod.BarChart);
      case 'user':
        return import('lucide-react').then(mod => mod.User);
      case 'settings':
        return import('lucide-react').then(mod => mod.Settings);
      default:
        return import('lucide-react').then(mod => mod.HelpCircle);
    }
  }
};

// Route-based code splitting configuration
export const routeBasedSplitting = {
  // Critical routes - preload
  critical: [
    '/',
    '/login',
    '/dashboard'
  ],
  
  // Heavy routes - lazy load
  heavy: [
    '/analytics',
    '/reports',
    '/admin',
    '/settings'
  ],
  
  // Rarely accessed - lazy load with low priority
  rare: [
    '/help',
    '/privacy',
    '/terms'
  ]
};

// Client hydration optimization
export const hydrationOptimization = {
  // Components that should never hydrate on server
  clientOnly: [
    'ThemeToggle',
    'InteractiveCharts',
    'RealtimeData',
    'WebSocketConnection'
  ],
  
  // Components that can be server-rendered but need client interaction
  hybrid: [
    'Navigation',
    'UserMenu',
    'SearchBox'
  ],
  
  // Components that should be fully server-rendered
  serverOnly: [
    'Footer',
    'StaticContent',
    'SEOMetadata'
  ]
};

// Performance monitoring
export const performanceMetrics = {
  // Track bundle sizes
  bundleSize: {
    js: 0,
    css: 0,
    total: 0
  },
  
  // Track hydration timing
  hydrationTime: 0,
  
  // Track route load times
  routeLoadTimes: new Map<string, number>(),
  
  // Track component render times
  componentRenderTimes: new Map<string, number>()
};

// Bundle size limits (in KB)
export const bundleSizeLimits = {
  // Main bundle should be under 200KB
  main: 200,
  
  // Vendor bundle should be under 300KB
  vendor: 300,
  
  // Route bundles should be under 100KB each
  route: 100,
  
  // Component bundles should be under 50KB each
  component: 50
};