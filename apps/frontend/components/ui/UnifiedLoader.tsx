/**
 * UNIFIED LOADER - Re-export from Shared
 */
export * from '@/shared/components/loaders/UnifiedLoader';

// Keep legacy aliases for backward compatibility in this app
import {
  UnifiedLoader,
  UnifiedLoading,
  UnifiedProgressBar,
  UnifiedSkeleton,
  type UnifiedLoaderProps
} from '@/shared/components/loaders/UnifiedLoader';

export const PancakeSwapLoader = ({ variant = 'pancake' as const, ...props }: Omit<UnifiedLoaderProps, 'variant'> & { variant?: 'pancake' | 'admin' | 'analytics' }) =>
  <UnifiedLoader variant={variant} type="stack" {...props} />;

export const EPSXLoader = UnifiedLoader;
export const ProfessionalLoader = UnifiedLoader;
export const MetroProgressBar = UnifiedProgressBar;
export const ProfessionalProgressBar = UnifiedProgressBar;
export const PancakeFlip = ({ variant = 'pancake' as const, size = 'md' as const }: { variant?: 'pancake' | 'admin' | 'analytics', size?: any }) =>
  <UnifiedLoader variant={variant} size={size} type="stack" />;
export const ProfessionalSkeleton = UnifiedSkeleton;
export const ProfessionalLoading = UnifiedLoading;