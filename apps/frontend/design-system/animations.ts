/**
 * Animation System for EPSX Platform
 * 
 * This file consolidates all animation definitions, keyframes, and utilities
 * from the existing CSS into a more organized, manageable system.
 * 
 * Benefits:
 * - Centralized animation management
 * - Type-safe animation utilities
 * - Better performance (only load needed animations)
 * - Consistent animation timing and easing
 * - Easy to extend and modify
 */

// ============================================================================
// ANIMATION KEYFRAMES
// ============================================================================

/**
 * CSS keyframe definitions for animations
 * These will be injected into the CSS at build time
 */
export const keyframes = {
  // Float animations
  float: {
    '0%, 100%': { transform: 'translateY(0)' },
    '50%': { transform: 'translateY(-20px)' },
  },
  
  floatGentle: {
    '0%, 100%': { transform: 'translateY(0) rotate(0deg)' },
    '33%': { transform: 'translateY(-8px) rotate(1deg)' },
    '66%': { transform: 'translateY(8px) rotate(-1deg)' },
  },
  
  floatReverse: {
    '0%, 100%': { transform: 'translateY(0) rotate(0deg)' },
    '50%': { transform: 'translateY(15px) rotate(2deg)' },
  },
  
  // Bounce animations
  bounceGentle: {
    '0%, 100%': { transform: 'translateY(0)' },
    '50%': { transform: 'translateY(-8px)' },
  },
  
  bounceSlow: {
    '0%, 100%': { transform: 'translateY(0)' },
    '50%': { transform: 'translateY(-15px)' },
  },
  
  // Pulse animations
  pulseGentle: {
    '0%, 100%': { opacity: '0.6', transform: 'scale(1)' },
    '50%': { opacity: '1', transform: 'scale(1.05)' },
  },
  
  pulseSlow: {
    '0%, 100%': { opacity: '0.4' },
    '50%': { opacity: '0.8' },
  },
  
  pulsePancake: {
    '0%, 100%': { opacity: '1' },
    '50%': { opacity: '0.7' },
  },
  
  // Scale animations
  scalePulse: {
    '0%, 100%': { transform: 'scale(1)' },
    '50%': { transform: 'scale(1.05)' },
  },
  
  // Rotation animations
  spinSlow: {
    from: { transform: 'rotate(0deg)' },
    to: { transform: 'rotate(360deg)' },
  },
  
  rotateSlow: {
    from: { transform: 'rotate(0deg)' },
    to: { transform: 'rotate(360deg)' },
  },
  
  // Wiggle animation
  wiggle: {
    '0%': { transform: 'rotate(0deg)' },
    '25%': { transform: 'rotate(1deg)' },
    '75%': { transform: 'rotate(-1deg)' },
    '100%': { transform: 'rotate(0deg)' },
  },
  
  // Gradient animations
  gradient: {
    '0%': { backgroundPosition: '0% 50%' },
    '50%': { backgroundPosition: '100% 50%' },
    '100%': { backgroundPosition: '0% 50%' },
  },
  
  gradientX: {
    '0%, 100%': { backgroundPosition: '0% 50%' },
    '50%': { backgroundPosition: '100% 50%' },
  },
  
  gradientShift: {
    '0%, 100%': { backgroundPosition: '0% 50%' },
    '50%': { backgroundPosition: '100% 50%' },
  },
  
  // Shine animation
  shine: {
    '0%': { transform: 'translateX(-100%)' },
    '100%': { transform: 'translateX(100%)' },
  },
  
  shimmer: {
    from: { transform: 'translateX(-100%)' },
    to: { transform: 'translateX(100%)' },
  },
  
  // Glow animations
  glow: {
    from: { boxShadow: '0 0 5px hsl(236 72 153 / 0.2)' },
    to: { boxShadow: '0 0 20px hsl(236 72 153 / 0.6)' },
  },
  
  pulseGlow: {
    '0%, 100%': { boxShadow: '0 0 5px hsl(255 133 27 / 0.3)' },
    '50%': { 
      boxShadow: '0 0 25px hsl(255 133 27 / 0.6), 0 0 35px hsl(255 133 27 / 0.4)' 
    },
  },
  
  // Slide animations
  slideUp: {
    from: { opacity: '0', transform: 'translateY(30px)' },
    to: { opacity: '1', transform: 'translateY(0)' },
  },
  
  slideInLeft: {
    from: { opacity: '0', transform: 'translateX(-20px)' },
    to: { opacity: '1', transform: 'translateX(0)' },
  },
  
  slideInRight: {
    from: { opacity: '0', transform: 'translateX(20px)' },
    to: { opacity: '1', transform: 'translateX(0)' },
  },
  
  // Fade animations
  fadeIn: {
    from: { opacity: '0', transform: 'translateY(20px)' },
    to: { opacity: '1', transform: 'translateY(0)' },
  },
  
  fadeInDelayed: {
    from: { opacity: '0', transform: 'translateY(20px)' },
    to: { opacity: '1', transform: 'translateY(0)' },
  },
  
  // Blob animations for decorative elements
  blob1: {
    '0%, 100%': { transform: 'scale(1) translate(0, 0)' },
    '33%': { transform: 'scale(1.1, 0.9) translate(10px, -10px)' },
    '66%': { transform: 'scale(0.95, 1.05) translate(-10px, 10px)' },
  },
  
  blob2: {
    '0%, 100%': { transform: 'scale(1) translate(0, 0)' },
    '33%': { transform: 'scale(1.05, 0.95) translate(-12px, 8px)' },
    '66%': { transform: 'scale(0.9, 1.1) translate(12px, -8px)' },
  },
  
  blob3: {
    '0%, 100%': { transform: 'scale(1) translate(0, 0)' },
    '33%': { transform: 'scale(1.08, 0.92) translate(8px, 12px)' },
    '66%': { transform: 'scale(0.92, 1.08) translate(-8px, -12px)' },
  },
  
  // Accordion animations
  accordionDown: {
    from: { height: '0' },
    to: { height: 'var(--radix-accordion-content-height)' },
  },
  
  accordionUp: {
    from: { height: 'var(--radix-accordion-content-height)' },
    to: { height: '0' },
  },
  
  // Collapsible animations
  collapsibleDown: {
    from: { height: '0' },
    to: { height: 'var(--collapsible-content-height)' },
  },
  
  collapsibleUp: {
    from: { height: 'var(--collapsible-content-height)' },
    to: { height: '0' },
  },
} as const;

// ============================================================================
// ANIMATION CONFIGURATIONS
// ============================================================================

/**
 * Pre-configured animation classes with timing and easing
 */
export const animations = {
  // Float animations
  float: 'float 6s ease-in-out infinite',
  floatGentle: 'floatGentle 4s ease-in-out infinite',
  floatReverse: 'floatReverse 5s ease-in-out infinite',
  floatDelayed: 'float 6s ease-in-out infinite -3s',
  
  // Bounce animations
  bounce: 'bounce 1s infinite',
  bounceGentle: 'bounceGentle 2s ease-in-out infinite',
  bounceSlow: 'bounceSlow 3s ease-in-out infinite',
  
  // Pulse animations
  pulse: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
  pulseGentle: 'pulseGentle 3s ease-in-out infinite',
  pulseSlow: 'pulseSlow 4s ease-in-out infinite',
  pulsePancake: 'pulsePancake 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
  
  // Scale animations
  scalePulse: 'scalePulse 3s ease-in-out infinite',
  
  // Rotation animations
  spin: 'spin 1s linear infinite',
  spinSlow: 'spinSlow 20s linear infinite',
  rotateSlow: 'rotateSlow 20s linear infinite',
  
  // Movement animations
  wiggle: 'wiggle 1s ease-in-out infinite',
  
  // Gradient animations
  gradient: 'gradient 8s ease infinite',
  gradientX: 'gradientX 3s ease infinite',
  gradientShift: 'gradientShift 3s ease-in-out infinite',
  
  // Shine and shimmer
  shine: 'shine 2s linear infinite',
  shimmer: 'shimmer 2s linear infinite',
  
  // Glow effects
  glow: 'glow 2s ease-in-out infinite alternate',
  pulseGlow: 'pulseGlow 3s ease-in-out infinite',
  
  // Slide animations
  slideUp: 'slideUp 0.8s ease-out',
  slideUpDelayed: 'slideUp 0.8s ease-out 0.2s both',
  slideUpDelayed2: 'slideUp 0.8s ease-out 0.4s both',
  slideInLeft: 'slideInLeft 0.6s ease-out',
  slideInRight: 'slideInRight 0.6s ease-out',
  
  // Fade animations
  fadeIn: 'fadeIn 1s ease-out forwards',
  fadeInDelayed: 'fadeInDelayed 1s ease-out 0.3s forwards',
  fadeInDelayed2: 'fadeInDelayed 1s ease-out 0.6s forwards',
  fadeInDelayed3: 'fadeInDelayed 1s ease-out 0.9s forwards',
  
  // Blob decorative animations
  blob1: 'blob1 12s ease-in-out infinite',
  blob2: 'blob2 14s ease-in-out infinite',
  blob3: 'blob3 16s ease-in-out infinite',
  
  // UI component animations
  accordionDown: 'accordionDown 0.2s ease-out',
  accordionUp: 'accordionUp 0.2s ease-out',
  collapsibleDown: 'collapsibleDown 0.2s ease-out',
  collapsibleUp: 'collapsibleUp 0.2s ease-out',
} as const;

// ============================================================================
// ANIMATION UTILITIES
// ============================================================================

/**
 * Animation utility classes
 */
export const animationClasses = {
  // Hover effects
  hoverScale: 'hover:scale-105 transition-transform duration-200',
  hoverScaleSmall: 'hover:scale-[1.02] transition-transform duration-200',
  hoverLift: 'hover:-translate-y-1 hover:scale-[1.02] transition-all duration-300',
  hoverGlow: 'hover:shadow-lg transition-shadow duration-300',
  
  // Interactive states
  interactiveScale: 'hover:scale-105 active:scale-95 transition-transform duration-200',
  interactivePancake: 'hover:scale-105 active:scale-95 transition-all duration-300',
  
  // Loading states
  loadingPulse: 'animate-pulse',
  loadingSpin: 'animate-spin',
  loadingBounce: 'animate-bounce',
  
  // Performance optimizations
  gpuAccelerated: 'transform-gpu will-change-transform',
  reducedMotion: 'motion-reduce:animate-none motion-reduce:transition-none',
} as const;

// ============================================================================
// TRANSITION PRESETS
// ============================================================================

/**
 * Common transition presets
 */
export const transitions = {
  // Basic transitions
  fast: 'transition-all duration-150 ease-in-out',
  normal: 'transition-all duration-300 ease-in-out',
  slow: 'transition-all duration-500 ease-in-out',
  
  // Specific property transitions
  colors: 'transition-colors duration-300 ease-in-out',
  opacity: 'transition-opacity duration-300 ease-in-out',
  transform: 'transition-transform duration-300 ease-in-out',
  shadow: 'transition-shadow duration-300 ease-in-out',
  
  // PancakeSwap style transitions
  pancake: 'transition-all duration-300 cubic-bezier(0.4, 0, 0.2, 1)',
  pancakeFast: 'transition-all duration-200 cubic-bezier(0.4, 0, 0.2, 1)',
  
  // Smooth easing functions
  smooth: 'transition-all duration-300 cubic-bezier(0.25, 0.1, 0.25, 1)',
  bouncy: 'transition-all duration-300 cubic-bezier(0.68, -0.55, 0.265, 1.55)',
} as const;

// ============================================================================
// MOTION PREFERENCES
// ============================================================================

/**
 * Responsive animation preferences
 * Respects user's motion preferences
 */
export const motionPreferences = {
  // Respect reduced motion preference
  respectReducedMotion: 'motion-reduce:animate-none motion-reduce:transition-none',
  
  // Performance considerations
  performanceMode: 'transform-gpu will-change-transform',
  
  // Mobile optimizations
  mobileOptimized: 'md:hover:scale-105 md:active:scale-95',
} as const;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

export type AnimationName = keyof typeof animations;
export type KeyframeName = keyof typeof keyframes;
export type TransitionPreset = keyof typeof transitions;
export type AnimationClass = keyof typeof animationClasses;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get animation class by name
 */
export function getAnimation(name: AnimationName): string {
  return `animate-[${animations[name]}]`;
}

/**
 * Get transition class by preset
 */
export function getTransition(preset: TransitionPreset): string {
  return transitions[preset];
}

/**
 * Create custom animation with duration and easing
 */
export function createAnimation(
  keyframe: KeyframeName,
  duration: string = '1s',
  easing: string = 'ease-in-out',
  iteration: string = 'infinite'
): string {
  return `animate-[${keyframe}_${duration}_${easing}_${iteration}]`;
}

/**
 * Combine multiple animation utilities
 */
export function combineAnimations(...classes: string[]): string {
  return classes.filter(Boolean).join(' ');
}

// ============================================================================
// EXPORTS
// ============================================================================

export {
  keyframes,
  animations,
  animationClasses,
  transitions,
  motionPreferences,
};

// Re-export utility functions
export {
  getAnimation,
  getTransition,
  createAnimation,
  combineAnimations,
};