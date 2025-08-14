'use client';

import { lazy, ComponentType, LazyExoticComponent } from 'react';
import { OptimizedSuspense } from '../common/OptimizedSuspense';

// Lazy load heavy components to reduce initial bundle size
export const LazyThemeToggle = lazy(() => 
  import('./OptimizedThemeToggle').then(mod => ({ default: mod.OptimizedThemeToggle }))
);

export const LazyNavControls = lazy(() => 
  import('./ClientNavControls').then(mod => ({ default: mod.ClientNavControls }))
);

export const LazyAnalyticsDashboard = lazy(() => 
  import('../analytics/AnalyticsRankingDash')
);

export const LazyPaymentSection = lazy(() => 
  import('../features/payment/DashboardPaymentSection')
);

export const LazySettingsPanel = lazy(() => 
  import('../features/settings/ProfileSettings')
);

// HOC for wrapping lazy components with Suspense
export function withLazyLoading<T extends {}>(
  LazyComponent: LazyExoticComponent<ComponentType<T>>,
  name: string,
  fallback?: React.ReactNode
) {
  return function LazyWrapper(props: T) {
    return (
      <OptimizedSuspense name={name} fallback={fallback}>
        <LazyComponent {...props} />
      </OptimizedSuspense>
    );
  };
}

// Pre-configured lazy components with Suspense
export const ThemeToggleWithSuspense = withLazyLoading(LazyThemeToggle, 'theme toggle');
export const NavControlsWithSuspense = withLazyLoading(LazyNavControls, 'navigation controls');
export const AnalyticsDashboardWithSuspense = withLazyLoading(LazyAnalyticsDashboard, 'analytics dashboard');
export const PaymentSectionWithSuspense = withLazyLoading(LazyPaymentSection, 'payment section');
export const SettingsPanelWithSuspense = withLazyLoading(LazySettingsPanel, 'settings panel');