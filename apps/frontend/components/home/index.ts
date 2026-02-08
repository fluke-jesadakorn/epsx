// Main component export
export { default as FinancialDataTable } from './financial-data-table';

// Sub-components
export { FinancialCard } from './components/financial-card';
export {
  GrowthIndicator,
  TrendIcon,
  AnimatedBadge,
} from './components/growth-indicators';
export { MetricCard, QuarterRow } from './components/metric-components';
export {
  FinancialDataLoading,
  FinancialDataHeader,
} from './components/layout-components';

// Hooks
export {
  useFinancialData,
  getValidQuarters,
  isPositiveGrowth,
  getGrowthIndicator,
} from './hooks/use-financial-data';
export type { UseFinancialDataResult } from './hooks/use-financial-data';

// Utilities
export {
  calculatePercentageChange,
  formatLargeNumber,
  getBestAvailablePrice,
  calculateVolatility,
  getRiskLevel,
  getTrendDirection,
  calculateCAGR,
  getPerformanceMetrics,
} from './utils/financial-calculations';
export type { PerformanceMetrics } from './utils/financial-calculations';

// Constants
export {
  GRADIENTS,
  ANIMATIONS,
  COLORS,
  SPACING,
  TYPOGRAPHY,
  BREAKPOINTS,
} from './constants/styles';
