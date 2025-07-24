/**
 * Shared Tailwind CSS configuration for EPSX monorepo
 * PancakeSwap-inspired design system configuration
 */

import { designTokens } from './design-tokens';
import { themeConfig } from './theme-config';

// Helper function to convert design tokens to Tailwind format
const convertTokensForTailwind = () => {
  // Convert color tokens
  const colors = {
    // Semantic colors with CSS variables
    border: 'hsl(var(--border))',
    input: 'hsl(var(--input))',
    ring: 'hsl(var(--ring))',
    background: 'hsl(var(--background))',
    foreground: 'hsl(var(--foreground))',
    primary: {
      DEFAULT: 'hsl(var(--primary))',
      foreground: 'hsl(var(--primary-foreground))',
      ...designTokens.colors.primary,
    },
    secondary: {
      DEFAULT: 'hsl(var(--secondary))',
      foreground: 'hsl(var(--secondary-foreground))',
      ...designTokens.colors.secondary,
    },
    destructive: {
      DEFAULT: 'hsl(var(--destructive))',
      foreground: 'hsl(var(--destructive-foreground))',
    },
    muted: {
      DEFAULT: 'hsl(var(--muted))',
      foreground: 'hsl(var(--muted-foreground))',
    },
    accent: {
      DEFAULT: 'hsl(var(--accent))',
      foreground: 'hsl(var(--accent-foreground))',
    },
    popover: {
      DEFAULT: 'hsl(var(--popover))',
      foreground: 'hsl(var(--popover-foreground))',
    },
    card: {
      DEFAULT: 'hsl(var(--card))',
      foreground: 'hsl(var(--card-foreground))',
    },
    
    // PancakeSwap colors with CSS variables
    pancake: {
      primary: 'hsl(var(--pancake-primary))',
      secondary: 'hsl(var(--pancake-secondary))',
      success: 'hsl(var(--pancake-success))',
      warning: 'hsl(var(--pancake-warning))',
      error: 'hsl(var(--pancake-error))',
      info: 'hsl(var(--pancake-info))',
      // Direct token access
      orange: designTokens.colors.pancake.orange,
      yellow: designTokens.colors.pancake.yellow,
      blue: designTokens.colors.pancake.blue,
      purple: designTokens.colors.pancake.purple,
      green: designTokens.colors.pancake.green,
      red: designTokens.colors.pancake.red,
    },
    
    // Standard color palette
    gray: designTokens.colors.gray,
  };
  
  return { colors };
};

// Base Tailwind configuration
export const baseTailwindConfig = {
  darkMode: ['class', '[data-theme="dark"]'] as const,
  theme: {
    container: {
      center: true,
      padding: {
        DEFAULT: '1rem',
        sm: '1.5rem',
        lg: '2rem',
      },
      screens: {
        '2xl': '1400px',
      },
    },
    extend: {
      ...convertTokensForTailwind(),
      
      // Spacing
      spacing: designTokens.spacing,
      
      // Typography
      fontFamily: designTokens.typography.fontFamily,
      fontSize: designTokens.typography.fontSize,
      fontWeight: designTokens.typography.fontWeight,
      letterSpacing: designTokens.typography.letterSpacing,
      
      // Border radius
      borderRadius: {
        ...designTokens.borderRadius,
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
        xl: 'calc(var(--radius) + 4px)',
        '2xl': 'calc(var(--radius) + 8px)',
        '3xl': 'calc(var(--radius) + 12px)',
      },
      
      // Box shadows
      boxShadow: designTokens.boxShadow,
      
      // Screens (breakpoints)
      screens: designTokens.screens,
      
      // Background images (gradients)
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
        'pancake-gradient': 'var(--gradient-primary)',
        'pancake-gradient-secondary': 'var(--gradient-secondary)',
        'pancake-gradient-accent': 'var(--gradient-accent)',
        'pancake-gradient-soft-highlight': 'var(--gradient-soft-highlight)',
        'pancake-rainbow': 'var(--gradient-rainbow)',
      },
      
      // Animations
      animation: {
        // Existing animations
        'float-smooth': 'floatSmooth 4s ease-in-out infinite',
        'bounce-gentle': 'bounceGentle 3s ease-in-out infinite',
        'pulse-glow': 'pulseGlow 3s ease-in-out infinite',
        'slide-up': 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'slide-up-delayed': 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards 0.2s',
        'slide-up-delayed-2': 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards 0.4s',
        'gradient-x': 'gradient-x 15s ease infinite',
        'gradient-y': 'gradient-y 15s ease infinite',
        'gradient-xy': 'gradient-xy 15s ease infinite',
        // PancakeSwap-style animations
        'wiggle': 'wiggle 1s ease-in-out infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'rotate-slow': 'rotateSlow 20s linear infinite',
        'scale-pulse': 'scalePulse 3s ease-in-out infinite',
        'shimmer': 'shimmer 2s linear infinite',
        'blob1': 'blob1 12s ease-in-out infinite',
        'blob2': 'blob2 14s ease-in-out infinite',
        'blob3': 'blob3 16s ease-in-out infinite',
        'fade-in': 'fadeIn 1s ease-out forwards',
        'fade-in-delayed': 'fadeIn 1s ease-out forwards 0.3s',
        'collapsible-down': 'collapsible-down 0.2s ease-out',
        'collapsible-up': 'collapsible-up 0.2s ease-out',
      },
      
      // Keyframes
      keyframes: {
        // Slide animations
        slideUp: {
          from: { opacity: '0', transform: 'translateY(30px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        // Float animations
        floatSmooth: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-15px)' },
        },
        bounceGentle: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-8px)' },
        },
        // Glow effect
        pulseGlow: {
          '0%, 100%': { boxShadow: '0 0 5px hsl(255 133 27 / 0.3)' },
          '50%': { boxShadow: '0 0 25px hsl(255 133 27 / 0.6), 0 0 35px hsl(255 133 27 / 0.4)' },
        },
        // Gradient animations
        'gradient-x': {
          '0%, 100%': { backgroundPosition: '0% 50%' },
          '50%': { backgroundPosition: '100% 50%' },
        },
        'gradient-y': {
          '0%, 100%': { transform: 'translateY(-100%)' },
          '50%': { transform: 'translateY(100%)' },
        },
        'gradient-xy': {
          '0%, 100%': { transform: 'translate(-100%, -100%)' },
          '25%': { transform: 'translate(100%, -100%)' },
          '50%': { transform: 'translate(100%, 100%)' },
          '75%': { transform: 'translate(-100%, 100%)' },
        },
        // PancakeSwap-style animations
        wiggle: {
          '0%': { transform: 'rotate(0deg)' },
          '25%': { transform: 'rotate(1deg)' },
          '75%': { transform: 'rotate(-1deg)' },
          '100%': { transform: 'rotate(0deg)' },
        },
        glow: {
          from: { boxShadow: '0 0 5px hsl(236 72 153 / 0.2)' },
          to: { boxShadow: '0 0 20px hsl(236 72 153 / 0.6)' },
        },
        rotateSlow: {
          from: { transform: 'rotate(0deg)' },
          to: { transform: 'rotate(360deg)' },
        },
        scalePulse: {
          '0%, 100%': { transform: 'scale(1)' },
          '50%': { transform: 'scale(1.05)' },
        },
        shimmer: {
          from: { transform: 'translateX(-100%)' },
          to: { transform: 'translateX(100%)' },
        },
        // Blob animations
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
        // Fade animations
        fadeIn: {
          from: { opacity: '0', transform: 'translateY(20px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        // Collapsible animations
        'collapsible-down': {
          from: { height: '0' },
          to: { height: 'var(--collapsible-content-height)' },
        },
        'collapsible-up': {
          from: { height: 'var(--collapsible-content-height)' },
          to: { height: '0' },
        },
      },
      
      // Z-index
      zIndex: designTokens.zIndex,
    },
  },
  plugins: [
    // Custom utility plugin
    function ({ addUtilities }: any) {
      // PancakeSwap utility classes
      addUtilities({
        '.pancake-gradient': {
          background: 'var(--gradient-primary)',
        },
        '.pancake-gradient-secondary': {
          background: 'var(--gradient-secondary)',
        },
        '.pancake-gradient-accent': {
          background: 'var(--gradient-accent)',
        },
        '.pancake-gradient-soft-highlight': {
          background: 'var(--gradient-soft-highlight)',
        },
        '.pancake-gradient-text': {
          background: 'linear-gradient(135deg, hsl(31 100% 50%) 0%, hsl(42 100% 70%) 100%)',
          backgroundSize: '200% 200%',
          '-webkit-background-clip': 'text',
          'background-clip': 'text',
          '-webkit-text-fill-color': 'transparent',
        },
        '.pancake-shadow': {
          boxShadow: '0 4px 12px hsl(255 133 27 / 0.15)',
        },
        '.pancake-glow': {
          boxShadow: '0 0 20px hsl(255 133 27 / 0.3)',
        },
      });
      
      // Responsive container utilities
      addUtilities({
        '.container-responsive': {
          width: '100%',
          marginLeft: 'auto',
          marginRight: 'auto',
          paddingLeft: '1rem',
          paddingRight: '1rem',
          maxWidth: '80rem',
        },
        '.container-section': {
          paddingTop: '3rem',
          paddingBottom: '3rem',
        },
      });
      
      // Enhanced card utilities
      addUtilities({
        '.card-pancake': {
          backgroundColor: 'hsl(var(--card) / 0.8)',
          backdropFilter: 'blur(8px)',
          border: '1px solid hsl(var(--border) / 0.5)',
          borderRadius: '1rem',
          padding: '1.5rem',
          transition: 'all 300ms ease',
          '&:hover': {
            boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
            borderColor: 'hsl(var(--primary) / 0.2)',
            transform: 'translateY(-0.25rem)',
          },
        },
      });
      
      // Glassmorphism utility
      addUtilities({
        '.glassmorphism': {
          backdropFilter: 'blur(16px)',
          '-webkit-backdrop-filter': 'blur(16px)',
          background: 'rgba(255, 255, 255, 0.1)',
          border: '1px solid rgba(255, 255, 255, 0.2)',
          boxShadow: '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
        },
      });
      
      // Interactive utilities
      addUtilities({
        '.interactive-scale': {
          transition: 'transform 0.2s ease-in-out',
          '&:hover': {
            transform: 'scale(1.05)',
          },
          '&:active': {
            transform: 'scale(0.95)',
          },
        },
      });
      
      // Custom scrollbar
      addUtilities({
        '.custom-scrollbar': {
          scrollbarWidth: 'thin',
          scrollbarColor: 'hsl(var(--muted)) transparent',
          '&::-webkit-scrollbar': {
            width: '0.5rem',
            height: '0.5rem',
          },
          '&::-webkit-scrollbar-track': {
            backgroundColor: 'rgb(243 244 246)',
            borderRadius: '0.25rem',
          },
          '&::-webkit-scrollbar-thumb': {
            backgroundColor: 'rgb(156 163 175)',
            borderRadius: '0.25rem',
            '&:hover': {
              backgroundColor: 'rgb(107 114 128)',
            },
          },
        },
      });
    },
  ],
} as const;

// Mobile-first responsive configuration
export const responsiveConfig = {
  // Mobile optimizations
  mobile: {
    fontSize: {
      'xs-responsive': ['0.75rem', { lineHeight: '1rem' }],
      'sm-responsive': ['0.875rem', { lineHeight: '1.25rem' }],
      'base-responsive': ['1rem', { lineHeight: '1.5rem' }],
    },
    spacing: {
      'mobile-tight': '0.5rem',
      'mobile-normal': '1rem',
      'mobile-loose': '1.5rem',
    },
    components: {
      'mobile-button': 'h-9 px-3 text-sm',
      'mobile-input': 'h-9 px-3 text-sm',
      'mobile-card': 'p-4 rounded-lg',
    },
  },
  
  // Tablet optimizations
  tablet: {
    spacing: {
      'tablet-section': '4rem',
      'tablet-gap': '2rem',
    },
    layout: {
      'tablet-grid': 'grid-cols-2 gap-6',
      'tablet-max-width': 'max-w-4xl',
    },
  },
  
  // Desktop optimizations
  desktop: {
    spacing: {
      'desktop-section': '6rem',
      'desktop-gap': '3rem',
    },
    layout: {
      'desktop-grid': 'grid-cols-3 gap-8',
      'desktop-max-width': 'max-w-7xl',
    },
  },
} as const;

// Theme-specific configurations
export const themeSpecificConfigs = {
  // Light theme configuration
  light: {
    colors: {
      background: 'hsl(252 100% 99%)',
      foreground: 'hsl(223 84% 10%)',
      primary: 'hsl(31 100% 50%)',
      secondary: 'hsl(42 100% 70%)',
    },
  },
  
  // Dark theme configuration
  dark: {
    colors: {
      background: 'hsl(240 10% 4%)',
      foreground: 'hsl(0 0% 95%)',
      primary: 'hsl(31 100% 55%)',
      secondary: 'hsl(42 100% 75%)',
    },
  },
} as const;

// Export complete configuration
export const sharedTailwindConfig = {
  ...baseTailwindConfig,
  responsive: responsiveConfig,
  themes: themeSpecificConfigs,
} as const;

export type TailwindConfig = typeof sharedTailwindConfig;
export type ResponsiveConfig = typeof responsiveConfig;
export type ThemeSpecificConfig = typeof themeSpecificConfigs;