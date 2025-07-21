/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './app/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  darkMode: ['class', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        secondary: {
          DEFAULT: 'hsl(var(--secondary))',
          foreground: 'hsl(var(--secondary-foreground))',
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
        // PancakeSwap inspired colors
        pancake: {
          primary: 'hsl(var(--pancake-primary))',
          secondary: 'hsl(var(--pancake-secondary))',
          success: 'hsl(var(--pancake-success))',
          warning: 'hsl(var(--pancake-warning))',
          error: 'hsl(var(--pancake-error))',
          info: 'hsl(var(--pancake-info))',
          purple: 'hsl(var(--pancake-purple))',
        },
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
    },
  },
  plugins: [
    function ({ addUtilities }) {
      addUtilities({
        '.custom-scrollbar::-webkit-scrollbar': {
          width: '0.5rem',
          height: '0.5rem',
        },
        '.custom-scrollbar::-webkit-scrollbar-track': {
          backgroundColor: 'rgb(243 244 246)',
          borderRadius: '0.25rem',
          '.dark &': {
            backgroundColor: 'rgb(31 41 55)',
          },
        },
        '.custom-scrollbar::-webkit-scrollbar-thumb': {
          backgroundColor: 'rgb(156 163 175)',
          borderRadius: '0.25rem',
          '&:hover': {
            backgroundColor: 'rgb(107 114 128)',
          },
          '.dark &': {
            backgroundColor: 'rgb(75 85 99)',
            '&:hover': {
              backgroundColor: 'rgb(107 114 128)',
            },
          },
        },
        '.scroll-shadow-container': {
          position: 'relative',
          overflow: 'hidden',
        },
        '.scroll-shadow-container::before': {
          content: '""',
          position: 'absolute',
          left: '0',
          right: '0',
          top: '0',
          height: '1.25rem',
          pointerEvents: 'none',
          zIndex: '10',
          background:
            'linear-gradient(to bottom, rgba(0,0,0,0.1) 0%, transparent 100%)',
          '.dark &': {
            background:
              'linear-gradient(to bottom, rgba(255,255,255,0.05) 0%, transparent 100%)',
          },
        },
        '.scroll-shadow-container::after': {
          content: '""',
          position: 'absolute',
          left: '0',
          right: '0',
          bottom: '0',
          height: '1.25rem',
          pointerEvents: 'none',
          zIndex: '10',
          background:
            'linear-gradient(to top, rgba(0,0,0,0.1) 0%, transparent 100%)',
          '.dark &': {
            background:
              'linear-gradient(to top, rgba(255,255,255,0.05) 0%, transparent 100%)',
          },
        },
        '.table-mask': {
          maskImage:
            'linear-gradient(to right, transparent, black 20px, black calc(100% - 20px), transparent)',
        },
      });
    },
  ],
};
