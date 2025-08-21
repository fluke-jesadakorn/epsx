/**
 * EPSX Design System
 * 
 * A unified, type-safe design system that replaces the fragmented CSS approach
 * with a centralized, maintainable solution.
 * 
 * This design system provides:
 * - Type-safe design tokens
 * - Composable component variants
 * - Centralized animation system
 * - Performance optimizations
 * - Consistent styling patterns
 * 
 * Usage:
 * ```tsx
 * import { tokens, buttonVariants, createButtonClass } from '@/design-system';
 * 
 * // Use design tokens
 * const primaryColor = tokens.colors.primary[500];
 * 
 * // Use component variants
 * const buttonClass = buttonVariants({ variant: 'primary', size: 'lg' });
 * 
 * // Use utility functions
 * const customButtonClass = createButtonClass(
 *   { variant: 'primary', glow: true },
 *   'custom-additional-classes'
 * );
 * ```
 */

// ============================================================================
// TOKENS EXPORT
// ============================================================================

export {
  // Main tokens object
  tokens,
  
  // Type helpers
  type ColorScale,
  type SpacingScale,
  type FontSize,
  type FontWeight,
  type BorderRadius as BorderRadiusType,
  type Shadow,
  type Breakpoint,
  type ZIndex,
  type DesignTokens,
} from './tokens';

// Re-export individual token categories through tokens object
export const { colors, spacing, typography, borderRadius, shadows, breakpoints, zIndex, transitions } = tokens;

// ============================================================================
// COMPONENT VARIANTS EXPORT
// ============================================================================

export {
  // Variant functions
  buttonVariants,
  cardVariants,
  badgeVariants,
  gradientTextVariants,
  animationVariants,
  
  // Utility functions
  createButtonClass,
  createCardClass,
  createBadgeClass,
  createGradientTextClass,
  createAnimationClass,
  
  // Variant types
  type ButtonVariants,
  type CardVariants,
  type BadgeVariants,
  type GradientTextVariants,
  type AnimationVariants,
} from './components';

// ============================================================================
// ANIMATIONS EXPORT
// ============================================================================

export {
  // Animation definitions
  keyframes,
  animations,
  animationClasses,
  transitions as animationTransitions,
  motionPreferences,
  
  // Utility functions
  getAnimation,
  getTransition,
  createAnimation,
  combineAnimations,
  
  // Animation types
  type AnimationName,
  type KeyframeName,
  type TransitionPreset,
  type AnimationClass,
} from './animations';

// ============================================================================
// DESIGN SYSTEM UTILITIES
// ============================================================================

/**
 * Get a design token value by path
 * Example: getToken('colors.primary.500') -> 'hsl(31 100% 50%)'
 */
export function getToken(path: string): string | undefined {
  const keys = path.split('.');
  let current: any = tokens;
  
  for (const key of keys) {
    if (current && typeof current === 'object' && key in current) {
      current = current[key];
    } else {
      return undefined;
    }
  }
  
  return typeof current === 'string' ? current : undefined;
}

/**
 * Get responsive spacing classes
 * Example: getResponsiveSpacing('md') -> 'p-4 sm:p-6 md:p-8'
 */
export function getResponsiveSpacing(
  base: string,
  sm?: string,
  md?: string,
  lg?: string
): string {
  const classes = [`p-${base}`];
  
  if (sm) classes.push(`sm:p-${sm}`);
  if (md) classes.push(`md:p-${md}`);
  if (lg) classes.push(`lg:p-${lg}`);
  
  return classes.join(' ');
}

/**
 * Get responsive text classes
 * Example: getResponsiveText('lg') -> 'text-base sm:text-lg md:text-xl'
 */
export function getResponsiveText(
  base: string,
  sm?: string,
  md?: string,
  lg?: string
): string {
  const classes = [`text-${base}`];
  
  if (sm) classes.push(`sm:text-${sm}`);
  if (md) classes.push(`md:text-${md}`);
  if (lg) classes.push(`lg:text-${lg}`);
  
  return classes.join(' ');
}

/**
 * Create CSS custom properties for dynamic theming
 */
export function createCSSCustomProperties(theme: 'light' | 'dark' = 'light') {
  const properties: Record<string, string> = {};
  
  // Background and foreground
  properties['--background'] = theme === 'light' ? 'hsl(252 100% 99%)' : 'hsl(240 10% 4%)';
  properties['--foreground'] = theme === 'light' ? 'hsl(223 84% 10%)' : 'hsl(0 0% 95%)';
  
  // Card colors
  properties['--card'] = theme === 'light' ? 'hsl(0 0% 100%)' : 'hsl(240 10% 6%)';
  properties['--card-foreground'] = theme === 'light' ? 'hsl(223 84% 10%)' : 'hsl(0 0% 95%)';
  
  // Primary colors
  properties['--primary'] = 'hsl(31 100% 50%)';
  properties['--primary-foreground'] = 'hsl(0 0% 100%)';
  
  return properties;
}

/**
 * Get theme-aware color
 */
export function getThemeColor(
  colorPath: string,
  theme: 'light' | 'dark' = 'light'
): string | undefined {
  // Handle theme-specific color paths
  if (colorPath.includes('.')) {
    const [category, ...rest] = colorPath.split('.');
    
    if (category === 'background') {
      return theme === 'light' ? 'hsl(252 100% 99%)' : 'hsl(240 10% 4%)';
    }
    
    if (category === 'foreground') {
      return theme === 'light' ? 'hsl(223 84% 10%)' : 'hsl(0 0% 95%)';
    }
    
    if (category === 'card' && rest[0] === 'foreground') {
      return theme === 'light' ? 'hsl(223 84% 10%)' : 'hsl(0 0% 95%)';
    }
  }
  
  return getToken(`colors.${colorPath}`);
}

// ============================================================================
// THEME CONSTANTS
// ============================================================================

/**
 * Common component class combinations for quick usage
 */
export const commonClasses = {
  // Container classes
  container: 'w-full mx-auto px-4 sm:px-6 lg:px-8 max-w-7xl',
  containerTight: 'w-full mx-auto px-4 sm:px-6 max-w-4xl',
  
  // Card classes
  card: createCardClass({ variant: 'default', padding: 'md' }),
  cardPancake: createCardClass({ variant: 'pancake', padding: 'md', glow: true }),
  cardGlass: createCardClass({ variant: 'glass', padding: 'md' }),
  
  // Button classes
  btnPrimary: createButtonClass({ variant: 'primary', size: 'md' }),
  btnSecondary: createButtonClass({ variant: 'secondary', size: 'md' }),
  btnOutline: createButtonClass({ variant: 'outline', size: 'md' }),
  
  // Text classes
  textPrimary: createGradientTextClass({ gradient: 'primary' }),
  textSecondary: createGradientTextClass({ gradient: 'secondary' }),
  
  // Animation classes
  floatAnimation: createAnimationClass({ float: 'gentle', scale: 'hover' }),
  interactiveAnimation: createAnimationClass({ scale: 'pancake' }),
} as const;

/**
 * Breakpoint utilities for responsive design
 */
export const responsive = {
  // Common breakpoint checks
  isMobile: '(max-width: 640px)',
  isTablet: '(min-width: 640px) and (max-width: 1024px)',
  isDesktop: '(min-width: 1024px)',
  
  // Utility classes
  mobileOnly: 'block sm:hidden',
  tabletOnly: 'hidden sm:block lg:hidden',
  desktopOnly: 'hidden lg:block',
  
  // Responsive visibility
  hideMobile: 'hidden sm:block',
  hideDesktop: 'block lg:hidden',
} as const;

// ============================================================================
// VERSION AND METADATA
// ============================================================================

export const designSystemMeta = {
  version: '1.0.0',
  lastUpdated: new Date().toISOString(),
  description: 'EPSX unified design system with type-safe tokens and components',
  migrationNotes: [
    'Replaces custom CSS classes like btn-pancake-*, card-pancake-*',
    'Consolidates PancakeSwap theme with standard design tokens',
    'Provides type-safe component variants with CVA',
    'Enables tree-shaking for better performance',
  ],
} as const;

// ============================================================================
// DEFAULT EXPORT
// ============================================================================

/**
 * Default export with most commonly used utilities
 */
const designSystem = {
  // Core systems
  tokens,
  variants: {
    button: buttonVariants,
    card: cardVariants,
    badge: badgeVariants,
    text: gradientTextVariants,
    animation: animationVariants,
  },
  
  // Utilities
  utils: {
    getToken,
    getResponsiveSpacing,
    getResponsiveText,
    createCSSCustomProperties,
    getThemeColor,
  },
  
  // Quick access
  common: commonClasses,
  responsive,
  
  // Metadata
  meta: designSystemMeta,
} as const;

export default designSystem;