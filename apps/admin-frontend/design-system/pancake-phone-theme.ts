/**
 * PancakeSwap x Windows Phone Fusion Design System
 * Modern tile-based UI with DeFi aesthetics and mobile-first approach
 */

export const PancakePhoneTheme = {
  colors: {
    // PancakeSwap brand colors
    primary: {
      50: '#FFF8E1',
      100: '#FFECB3',
      200: '#FFE082',
      300: '#FFD54F',
      400: '#FFCA28',
      500: '#FFC107', // Main PancakeSwap yellow
      600: '#FFB300',
      700: '#FFA000',
      800: '#FF8F00',
      900: '#FF6F00'
    },

    // Windows Phone accent colors
    accent: {
      blue: '#0078D4',
      purple: '#8764B8',
      green: '#107C10',
      red: '#D13438',
      orange: '#FF8C00',
      pink: '#E3008C'
    },

    // Dark theme (Windows Phone black)
    dark: {
      background: '#000000',
      surface: '#1A1A1A',
      card: '#2D2D2D',
      text: '#FFFFFF',
      textSecondary: '#B3B3B3',
      border: '#404040'
    },

    // PancakeSwap gradients
    gradients: {
      pancake: 'linear-gradient(135deg, #FFD800 0%, #FFA726 100%)',
      success: 'linear-gradient(135deg, #31D0AA 0%, #11CDEF 100%)',
      warning: 'linear-gradient(135deg, #FFB237 0%, #F5A623 100%)',
      error: 'linear-gradient(135deg, #FF6838 0%, #FF3838 100%)'
    }
  },

  // Windows Phone typography
  typography: {
    // Light weight for Windows Phone aesthetic
    weights: {
      light: 200,
      normal: 300,
      medium: 400,
      semibold: 500,
      bold: 600
    },

    sizes: {
      xs: '0.75rem',   // 12px
      sm: '0.875rem',  // 14px  
      base: '1rem',    // 16px
      lg: '1.125rem',  // 18px
      xl: '1.25rem',   // 20px
      '2xl': '1.5rem', // 24px
      '3xl': '2rem',   // 32px
      '4xl': '2.5rem', // 40px (Windows Phone header)
      '5xl': '3rem'    // 48px (Hero text)
    }
  },

  // Live Tile system
  tiles: {
    sizes: {
      small: 'w-20 h-20',      // 80x80px
      medium: 'w-40 h-20',     // 160x80px  
      wide: 'w-40 h-40',       // 160x160px
      large: 'w-80 h-80'       // 320x320px
    },

    states: {
      active: 'opacity-100',
      inactive: 'opacity-75',
      disabled: 'opacity-50',
      hidden: 'opacity-0'
    }
  },

  // Spacing (Windows Phone 8px grid)
  spacing: {
    xs: '0.5rem',  // 8px
    sm: '1rem',    // 16px
    md: '1.5rem',  // 24px
    lg: '2rem',    // 32px
    xl: '3rem',    // 48px
    xxl: '4rem'    // 64px
  },

  // Component styles
  components: {
    tile: {
      base: 'cursor-pointer shadow-lg hover:shadow-xl',
      padding: 'p-4',
      rounded: 'rounded-none' // Windows Phone sharp corners
    },

    pivot: {
      base: 'flex overflow-x-auto gap-6 border-b border-gray-700',
      item: 'font-light text-lg pb-2 whitespace-nowrap',
      active: 'text-white border-b-2 border-primary-500',
      inactive: 'text-gray-400 hover:text-white'
    },

    button: {
      primary: 'bg-gradient-to-r from-primary-500 to-primary-600 text-black font-medium hover:from-primary-400 hover:to-primary-500',
      secondary: 'bg-accent-blue text-white font-light hover:bg-blue-600',
      ghost: 'text-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900'
    }
  }
} as const

export type PancakePhoneThemeType = typeof PancakePhoneTheme