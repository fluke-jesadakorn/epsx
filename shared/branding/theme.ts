// EPSX Unified Branding Theme
// Professional design system for all EPSX applications

export const epsxTheme = {
  // Brand Identity
  brand: {
    name: 'EPSX',
    fullName: 'EPSX Analytics',
    tagline: 'Professional Market Analytics Platform',
    description: 'Advanced analytics platform for professional analysts',
  },

  // Color Palette
  colors: {
    // Primary Brand Colors
    primary: {
      50: '#eff6ff',
      100: '#dbeafe',
      200: '#bfdbfe',
      300: '#93c5fd',
      400: '#60a5fa',
      500: '#3b82f6',
      600: '#2563eb',
      700: '#1d4ed8',
      800: '#1e40af',
      900: '#1e3a8a',
    },

    // Secondary Colors
    secondary: {
      50: '#f8fafc',
      100: '#f1f5f9',
      200: '#e2e8f0',
      300: '#cbd5e1',
      400: '#94a3b8',
      500: '#64748b',
      600: '#475569',
      700: '#334155',
      800: '#1e293b',
      900: '#0f172a',
    },

    // Semantic Colors
    success: {
      50: '#ecfdf5',
      500: '#10b981',
      600: '#059669',
    },
    warning: {
      50: '#fffbeb',
      500: '#f59e0b',
      600: '#d97706',
    },
    error: {
      50: '#fef2f2',
      500: '#ef4444',
      600: '#dc2626',
    },
    info: {
      50: '#eff6ff',
      500: '#3b82f6',
      600: '#2563eb',
    },
  },

  // Typography
  typography: {
    fontFamily: {
      sans: ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      mono: ['JetBrains Mono', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
    },
    fontSize: {
      xs: '0.75rem',
      sm: '0.875rem',
      base: '1rem',
      lg: '1.125rem',
      xl: '1.25rem',
      '2xl': '1.5rem',
      '3xl': '1.875rem',
      '4xl': '2.25rem',
    },
    fontWeight: {
      normal: '400',
      medium: '500',
      semibold: '600',
      bold: '700',
    },
    lineHeight: {
      tight: '1.25',
      normal: '1.5',
      relaxed: '1.75',
    },
  },

  // Spacing
  spacing: {
    0: '0',
    1: '0.25rem',
    2: '0.5rem',
    3: '0.75rem',
    4: '1rem',
    5: '1.25rem',
    6: '1.5rem',
    8: '2rem',
    10: '2.5rem',
    12: '3rem',
    16: '4rem',
    20: '5rem',
    24: '6rem',
  },

  // Border Radius
  borderRadius: {
    none: '0',
    sm: '0.25rem',
    md: '0.5rem',
    lg: '0.75rem',
    xl: '1rem',
    full: '9999px',
  },

  // Shadows
  boxShadow: {
    sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
    md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
    lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
    xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  },

  // Gradients
  gradients: {
    primary: 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
    secondary: 'linear-gradient(135deg, #64748b 0%, #475569 100%)',
    analytics: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
    success: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
    warning: 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)',
    error: 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)',
  },

  // Component Styles
  components: {
    button: {
      primary: 'bg-gradient-to-r from-blue-500 to-blue-600 text-white hover:from-blue-600 hover:to-blue-700',
      secondary: 'bg-white text-gray-700 border border-gray-300 hover:bg-gray-50',
      ghost: 'text-gray-600 hover:text-gray-900 hover:bg-gray-100',
    },
    card: {
      default: 'bg-white border border-gray-200 shadow-sm',
      analytics: 'bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200',
      premium: 'bg-gradient-to-br from-purple-50 to-pink-50 border-purple-200',
    },
    input: {
      default: 'border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent',
    },
  },

  // Application-specific branding
  applications: {
    frontend: {
      name: 'EPSX Analytics',
      description: 'Professional Market Analytics',
      primaryColor: 'primary',
      logo: 'E',
    },
    admin: {
      name: 'EPSX Admin',
      description: 'Administrative Dashboard',
      primaryColor: 'secondary',
      logo: 'E',
    },
    backend: {
      name: 'EPSX API',
      description: 'Analytics API Server',
      primaryColor: 'primary',
      logo: 'E',
    },
  },

  // Layout
  layout: {
    maxWidth: {
      sm: '640px',
      md: '768px',
      lg: '1024px',
      xl: '1280px',
      '2xl': '1536px',
    },
    breakpoints: {
      sm: '640px',
      md: '768px',
      lg: '1024px',
      xl: '1280px',
      '2xl': '1536px',
    },
  },

  // Animation & Transitions
  animation: {
    duration: {
      fast: '150ms',
      normal: '200ms',
      slow: '300ms',
    },
    easing: {
      default: 'cubic-bezier(0.4, 0, 0.2, 1)',
      in: 'cubic-bezier(0.4, 0, 1, 1)',
      out: 'cubic-bezier(0, 0, 0.2, 1)',
      inOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    },
  },
} as const;

// Theme utilities
export const getApplicationTheme = (app: keyof typeof epsxTheme.applications) => {
  return {
    ...epsxTheme,
    current: epsxTheme.applications[app],
  };
};

export const getCSSVariables = () => {
  const variables: Record<string, string> = {};

  // Convert color palette to CSS variables
  Object.entries(epsxTheme.colors).forEach(([colorName, colorValues]) => {
    if (typeof colorValues === 'object') {
      Object.entries(colorValues).forEach(([shade, value]) => {
        variables[`--epsx-${colorName}-${shade}`] = value;
      });
    }
  });

  // Add spacing variables
  Object.entries(epsxTheme.spacing).forEach(([key, value]) => {
    variables[`--epsx-spacing-${key}`] = value;
  });

  // Add border radius variables
  Object.entries(epsxTheme.borderRadius).forEach(([key, value]) => {
    variables[`--epsx-radius-${key}`] = value;
  });

  return variables;
};

export type EPSXTheme = typeof epsxTheme;
export type ApplicationTheme = ReturnType<typeof getApplicationTheme>;