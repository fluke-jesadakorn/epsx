/**
 * Admin Frontend Animation System
 * 
 * Centralized animation utilities and presets for admin interfaces.
 * Provides consistent motion design across the application.
 * 
 * Features:
 * - Performance-optimized animations
 * - Accessibility-aware motion
 * - Admin-specific interaction patterns
 * - Type-safe animation configurations
 */

// ============================================================================
// ANIMATION PRESETS
// ============================================================================

/**
 * Common animation durations in milliseconds
 */
export const durations = {
  instant: 0,
  fast: 150,
  normal: 300,
  slow: 500,
  slower: 700,
  slowest: 1000,
} as const;

/**
 * Easing functions for natural motion
 */
export const easings = {
  linear: 'linear',
  easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
  easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
  easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
  
  // Custom easing for admin interfaces
  bounceIn: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
  slideOut: 'cubic-bezier(0.55, 0, 0.1, 1)',
  fadeIn: 'cubic-bezier(0.16, 1, 0.3, 1)',
  admin: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)',
} as const;

// ============================================================================
// KEYFRAME DEFINITIONS
// ============================================================================

/**
 * CSS keyframe animations as JavaScript objects
 */
export const keyframes = {
  // Fade animations
  fadeIn: {
    '0%': { opacity: '0', transform: 'translateY(10px)' },
    '100%': { opacity: '1', transform: 'translateY(0)' },
  },
  
  fadeOut: {
    '0%': { opacity: '1', transform: 'translateY(0)' },
    '100%': { opacity: '0', transform: 'translateY(-10px)' },
  },
  
  // Slide animations
  slideInFromRight: {
    '0%': { transform: 'translateX(100%)' },
    '100%': { transform: 'translateX(0)' },
  },
  
  slideOutToRight: {
    '0%': { transform: 'translateX(0)' },
    '100%': { transform: 'translateX(100%)' },
  },
  
  slideInFromLeft: {
    '0%': { transform: 'translateX(-100%)' },
    '100%': { transform: 'translateX(0)' },
  },
  
  slideOutToLeft: {
    '0%': { transform: 'translateX(0)' },
    '100%': { transform: 'translateX(-100%)' },
  },
  
  slideUp: {
    '0%': { opacity: '0', transform: 'translateY(20px)' },
    '100%': { opacity: '1', transform: 'translateY(0)' },
  },
  
  slideDown: {
    '0%': { opacity: '1', transform: 'translateY(0)' },
    '100%': { opacity: '0', transform: 'translateY(20px)' },
  },
  
  // Scale animations
  scaleIn: {
    '0%': { opacity: '0', transform: 'scale(0.9)' },
    '100%': { opacity: '1', transform: 'scale(1)' },
  },
  
  scaleOut: {
    '0%': { opacity: '1', transform: 'scale(1)' },
    '100%': { opacity: '0', transform: 'scale(0.9)' },
  },
  
  // PancakeSwap-style animations
  pancakeSlideUp: {
    '0%': { opacity: '0', transform: 'translateY(30px) scale(0.9)' },
    '100%': { opacity: '1', transform: 'translateY(0) scale(1)' },
  },
  
  pancakeSlideDown: {
    '0%': { opacity: '1', transform: 'translateY(0) scale(1)' },
    '100%': { opacity: '0', transform: 'translateY(30px) scale(0.9)' },
  },
  
  // Admin-specific animations
  permissionGranted: {
    '0%': { transform: 'scale(1)', backgroundColor: 'hsl(142 71% 45% / 0.1)' },
    '50%': { transform: 'scale(1.05)', backgroundColor: 'hsl(142 71% 45% / 0.2)' },
    '100%': { transform: 'scale(1)', backgroundColor: 'hsl(142 71% 45% / 0.1)' },
  },
  
  permissionDenied: {
    '0%': { transform: 'scale(1)', backgroundColor: 'hsl(0 85% 60% / 0.1)' },
    '25%': { transform: 'translateX(-5px)', backgroundColor: 'hsl(0 85% 60% / 0.15)' },
    '75%': { transform: 'translateX(5px)', backgroundColor: 'hsl(0 85% 60% / 0.15)' },
    '100%': { transform: 'scale(1)', backgroundColor: 'hsl(0 85% 60% / 0.1)' },
  },
  
  dataRefresh: {
    '0%': { transform: 'rotate(0deg)', opacity: '1' },
    '50%': { transform: 'rotate(180deg)', opacity: '0.7' },
    '100%': { transform: 'rotate(360deg)', opacity: '1' },
  },
  
  // Loading animations
  shimmer: {
    '0%': { backgroundPosition: '-468px 0' },
    '100%': { backgroundPosition: '468px 0' },
  },
  
  pulse: {
    '0%, 100%': { opacity: '1' },
    '50%': { opacity: '0.5' },
  },
  
  spin: {
    '0%': { transform: 'rotate(0deg)' },
    '100%': { transform: 'rotate(360deg)' },
  },
  
  // Gradient animations
  gradientShift: {
    '0%': { backgroundPosition: '0% 50%' },
    '50%': { backgroundPosition: '100% 50%' },
    '100%': { backgroundPosition: '0% 50%' },
  },
  
  // Menu animations
  menuItemSlideIn: {
    '0%': { opacity: '0', transform: 'translateX(-20px)' },
    '100%': { opacity: '1', transform: 'translateX(0)' },
  },
  
  submenuSlideIn: {
    '0%': { opacity: '0', transform: 'translateX(-100%)' },
    '100%': { opacity: '1', transform: 'translateX(0)' },
  },
  
  submenuSlideOut: {
    '0%': { opacity: '1', transform: 'translateX(0)' },
    '100%': { opacity: '0', transform: 'translateX(-100%)' },
  },
} as const;

// ============================================================================
// ANIMATION CONFIGURATIONS
// ============================================================================

/**
 * Pre-configured animation combinations
 */
export const animations = {
  // Fade animations
  fadeIn: {
    keyframes: keyframes.fadeIn,
    duration: durations.normal,
    easing: easings.fadeIn,
    fillMode: 'both',
  },
  
  fadeOut: {
    keyframes: keyframes.fadeOut,
    duration: durations.fast,
    easing: easings.easeIn,
    fillMode: 'both',
  },
  
  // Modal animations
  modalEnter: {
    keyframes: keyframes.pancakeSlideUp,
    duration: durations.normal,
    easing: easings.bounceIn,
    fillMode: 'both',
  },
  
  modalExit: {
    keyframes: keyframes.pancakeSlideDown,
    duration: durations.fast,
    easing: easings.easeIn,
    fillMode: 'both',
  },
  
  // Drawer animations
  drawerSlideIn: {
    keyframes: keyframes.slideInFromRight,
    duration: durations.normal,
    easing: easings.slideOut,
    fillMode: 'both',
  },
  
  drawerSlideOut: {
    keyframes: keyframes.slideOutToRight,
    duration: durations.fast,
    easing: easings.easeIn,
    fillMode: 'both',
  },
  
  // Card animations
  cardHover: {
    transform: 'translateY(-2px) scale(1.02)',
    transition: `all ${durations.fast}ms ${easings.easeOut}`,
  },
  
  cardPress: {
    transform: 'translateY(0px) scale(0.98)',
    transition: `all ${durations.fast}ms ${easings.easeIn}`,
  },
  
  // Button animations
  buttonHover: {
    transform: 'scale(1.05)',
    transition: `all ${durations.fast}ms ${easings.easeOut}`,
  },
  
  buttonPress: {
    transform: 'scale(0.95)',
    transition: `all ${durations.fast}ms ${easings.easeIn}`,
  },
  
  // Loading animations
  skeletonShimmer: {
    keyframes: keyframes.shimmer,
    duration: durations.slowest,
    easing: easings.linear,
    iterationCount: 'infinite',
  },
  
  spinnerRotate: {
    keyframes: keyframes.spin,
    duration: durations.slowest,
    easing: easings.linear,
    iterationCount: 'infinite',
  },
  
  // Admin-specific animations
  permissionUpdate: {
    keyframes: keyframes.permissionGranted,
    duration: 600,
    easing: easings.easeOut,
    fillMode: 'both',
  },
  
  permissionError: {
    keyframes: keyframes.permissionDenied,
    duration: 600,
    easing: easings.easeOut,
    fillMode: 'both',
  },
  
  dataSync: {
    keyframes: keyframes.dataRefresh,
    duration: durations.slowest,
    easing: easings.easeInOut,
    fillMode: 'both',
  },
} as const;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Create a CSS animation string from configuration
 */
export function createAnimation(config: {
  keyframes: string;
  duration: number;
  easing?: string;
  delay?: number;
  iterationCount?: number | 'infinite';
  fillMode?: 'none' | 'forwards' | 'backwards' | 'both';
}): string {
  const {
    keyframes,
    duration,
    easing = easings.easeOut,
    delay = 0,
    iterationCount = 1,
    fillMode = 'none',
  } = config;
  
  return `${keyframes} ${duration}ms ${easing} ${delay}ms ${iterationCount} ${fillMode}`;
}

/**
 * Generate staggered animation delays for lists
 */
export function createStaggeredDelay(index: number, baseDelay = 50): number {
  return index * baseDelay;
}

/**
 * Check if user prefers reduced motion
 */
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * Get animation duration based on user preferences
 */
export function getAnimationDuration(duration: number): number {
  return prefersReducedMotion() ? 0 : duration;
}

/**
 * Create a spring animation configuration
 */
export function createSpringAnimation(config: {
  tension?: number;
  friction?: number;
  mass?: number;
}) {
  const { tension = 280, friction = 120, mass = 1 } = config;
  
  // Calculate natural frequency and damping ratio
  const w0 = Math.sqrt(tension / mass);
  const zeta = friction / (2 * Math.sqrt(tension * mass));
  
  // Generate cubic-bezier values
  const x1 = zeta / w0;
  const y1 = 1 - Math.exp(-x1);
  const x2 = 1 - zeta / w0;
  const y2 = Math.exp(-x2);
  
  return `cubic-bezier(${x1.toFixed(3)}, ${y1.toFixed(3)}, ${x2.toFixed(3)}, ${y2.toFixed(3)})`;
}

// ============================================================================
// ANIMATION CLASSES
// ============================================================================

/**
 * CSS class utilities for common animations
 */
export const animationClasses = {
  // Entrance animations
  'animate-fade-in': {
    animation: createAnimation({
      keyframes: 'fadeIn',
      duration: durations.normal,
      easing: easings.fadeIn,
      fillMode: 'both',
    }),
  },
  
  'animate-slide-up': {
    animation: createAnimation({
      keyframes: 'slideUp',
      duration: durations.normal,
      easing: easings.easeOut,
      fillMode: 'both',
    }),
  },
  
  'animate-scale-in': {
    animation: createAnimation({
      keyframes: 'scaleIn',
      duration: durations.normal,
      easing: easings.bounceIn,
      fillMode: 'both',
    }),
  },
  
  // Loading animations
  'animate-pulse': {
    animation: createAnimation({
      keyframes: 'pulse',
      duration: durations.slower,
      easing: easings.easeInOut,
      iterationCount: 'infinite',
    }),
  },
  
  'animate-spin': {
    animation: createAnimation({
      keyframes: 'spin',
      duration: durations.slowest,
      easing: easings.linear,
      iterationCount: 'infinite',
    }),
  },
  
  'animate-shimmer': {
    animation: createAnimation({
      keyframes: 'shimmer',
      duration: durations.slowest,
      easing: easings.linear,
      iterationCount: 'infinite',
    }),
  },
  
  // Interaction animations
  'animate-hover-lift': {
    transition: `transform ${durations.fast}ms ${easings.easeOut}`,
    '&:hover': {
      transform: 'translateY(-2px)',
    },
  },
  
  'animate-hover-scale': {
    transition: `transform ${durations.fast}ms ${easings.easeOut}`,
    '&:hover': {
      transform: 'scale(1.02)',
    },
  },
  
  // Admin-specific animations
  'animate-permission-granted': {
    animation: createAnimation({
      keyframes: 'permissionGranted',
      duration: 600,
      easing: easings.easeOut,
      fillMode: 'both',
    }),
  },
  
  'animate-permission-denied': {
    animation: createAnimation({
      keyframes: 'permissionDenied',
      duration: 600,
      easing: easings.easeOut,
      fillMode: 'both',
    }),
  },
  
  'animate-data-refresh': {
    animation: createAnimation({
      keyframes: 'dataRefresh',
      duration: durations.slowest,
      easing: easings.easeInOut,
      fillMode: 'both',
    }),
  },
} as const;

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type Duration = keyof typeof durations;
export type Easing = keyof typeof easings;
export type Keyframe = keyof typeof keyframes;
export type Animation = keyof typeof animations;
export type AnimationClass = keyof typeof animationClasses;