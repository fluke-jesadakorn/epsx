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
        },
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
        xl: 'calc(var(--radius) + 4px)',
        '2xl': 'calc(var(--radius) + 8px)',
        '3xl': 'calc(var(--radius) + 12px)',
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
        'pancake-gradient': 'var(--gradient-primary)',
        'pancake-gradient-secondary': 'var(--gradient-secondary)',
        'pancake-gradient-accent': 'var(--gradient-accent)',
        'pancake-gradient-soft-highlight': 'var(--gradient-soft-highlight)',
      },
      animation: {
        'float-smooth': 'floatSmooth 4s ease-in-out infinite',
        'bounce-gentle': 'bounceGentle 3s ease-in-out infinite',
        'pulse-glow': 'pulseGlow 3s ease-in-out infinite',
        'slide-up': 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'slide-up-delayed': 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards 0.2s',
        'slide-up-delayed-2': 'slideUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards 0.4s',
        'gradient-x': 'gradient-x 15s ease infinite',
        'gradient-y': 'gradient-y 15s ease infinite',
        'gradient-xy': 'gradient-xy 15s ease infinite',
      },
      screens: {
        'xs': '475px',
        '3xl': '1600px',
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
        '128': '32rem',
      },
      fontFamily: {
        sans: ['Kanit', 'Inter', 'system-ui', 'sans-serif'],
        pancake: ['Kanit', 'system-ui', 'sans-serif'],
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
