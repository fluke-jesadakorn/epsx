// Main component export
export { default as FinancialDataTable } from './FinancialDataTable';

// Sub-components
export { FinancialCard } from './components/FinancialCard';
export {
  GrowthIndicator,
  TrendIcon,
  AnimatedBadge,
} from './components/GrowthIndicators';
export { MetricCard, QuarterRow } from './components/MetricComponents';
export {
  FinancialDataLoading,
  FinancialDataHeader,
} from './components/LayoutComponents';

// PancakeSwap-style components
export {
  FloatingElements,
  PancakeButton,
  GlowCard,
  EmojiBadge,
} from './components/PancakeElements';

// Hooks
export {
  useFinancialData,
  getValidQuarters,
  isPositiveGrowth,
  getGrowthIndicator,
} from './hooks/useFinancialData';
export type { UseFinancialDataResult } from './hooks/useFinancialData';

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
} from './utils/financialCalculations';
export type { PerformanceMetrics } from './utils/financialCalculations';

// Constants
export {
  GRADIENTS,
  ANIMATIONS,
  COLORS,
  SPACING,
  TYPOGRAPHY,
  BREAKPOINTS,
} from './constants/styles';
