/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    './pages/**/*.{ts,tsx}',
    './components/**/*.{ts,tsx}',
    './app/**/*.{ts,tsx}',
    './src/**/*.{ts,tsx}',
    './lib/**/*.{ts,tsx}',
    './utils/**/*.{ts,tsx}',
    './hooks/**/*.{ts,tsx}',
    './context/**/*.{ts,tsx}',
    './middleware/**/*.{ts,tsx}',
    './design-system/**/*.{ts,tsx}',
    './styles/**/*.css',
  ],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        
        // Core brand colors
        primary: {
          50: "hsl(31 100% 95%)",
          100: "hsl(31 90% 90%)",
          200: "hsl(31 95% 85%)",
          300: "hsl(31 95% 75%)",
          400: "hsl(31 95% 65%)",
          500: "hsl(var(--primary))", // Main primary
          600: "hsl(31 95% 45%)",
          700: "hsl(31 90% 40%)",
          800: "hsl(31 85% 35%)",
          900: "hsl(31 80% 30%)",
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        
        secondary: {
          50: "hsl(42 100% 95%)",
          100: "hsl(42 95% 88%)",
          200: "hsl(42 90% 80%)",
          300: "hsl(42 95% 75%)",
          400: "hsl(42 100% 70%)",
          500: "hsl(var(--secondary))", // Main secondary
          600: "hsl(42 95% 65%)",
          700: "hsl(42 90% 60%)",
          800: "hsl(42 85% 55%)",
          900: "hsl(42 80% 50%)",
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        
        // System status colors
        success: {
          50: "hsl(142 76% 95%)",
          100: "hsl(142 71% 88%)",
          200: "hsl(142 71% 75%)",
          300: "hsl(142 71% 65%)",
          400: "hsl(142 71% 55%)",
          500: "hsl(142 71% 45%)", // Main success
          600: "hsl(142 66% 40%)",
          700: "hsl(142 61% 35%)",
          800: "hsl(142 56% 30%)",
          900: "hsl(142 51% 25%)",
        },
        
        warning: {
          50: "hsl(45 98% 95%)",
          100: "hsl(45 93% 88%)",
          200: "hsl(45 93% 75%)",
          300: "hsl(45 93% 65%)",
          400: "hsl(45 93% 57%)",
          500: "hsl(45 93% 47%)", // Main warning
          600: "hsl(45 88% 42%)",
          700: "hsl(45 83% 37%)",
          800: "hsl(45 78% 32%)",
          900: "hsl(45 73% 27%)",
        },
        
        error: {
          50: "hsl(0 93% 95%)",
          100: "hsl(0 88% 88%)",
          200: "hsl(0 88% 75%)",
          300: "hsl(0 88% 65%)",
          400: "hsl(0 88% 58%)",
          500: "hsl(0 85% 60%)", // Main error
          600: "hsl(0 83% 55%)",
          700: "hsl(0 80% 50%)",
          800: "hsl(0 77% 45%)",
          900: "hsl(0 74% 40%)",
        },
        
        info: {
          50: "hsl(217 96% 95%)",
          100: "hsl(217 91% 88%)",
          200: "hsl(217 91% 75%)",
          300: "hsl(217 91% 65%)",
          400: "hsl(217 91% 58%)",
          500: "hsl(217 91% 60%)", // Main info
          600: "hsl(217 86% 55%)",
          700: "hsl(217 81% 50%)",
          800: "hsl(217 76% 45%)",
          900: "hsl(217 71% 40%)",
        },
        
        // Neutral grays
        neutral: {
          50: "hsl(217 32% 96%)",
          100: "hsl(217 32% 91%)",
          200: "hsl(215 28% 85%)",
          300: "hsl(215 28% 75%)",
          400: "hsl(215 20% 65%)",
          500: "hsl(215 16% 47%)", // Main neutral
          600: "hsl(215 19% 35%)",
          700: "hsl(215 25% 27%)",
          800: "hsl(215 28% 17%)",
          900: "hsl(220 26% 14%)",
        },
        
        // Legacy Tailwind compatibility
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
        xl: "0.75rem",
        "2xl": "1rem",
        "3xl": "1.5rem",
      },
      
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
        '112': '28rem',
        '128': '32rem',
      },
      
      zIndex: {
        '60': '60',
        '70': '70',
        '80': '80',
        '90': '90',
        '100': '100',
      },
      
      keyframes: {
        // Legacy accordion animations
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        
        // Enhanced admin animations
        "fade-in": {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "slide-up": {
          "0%": { opacity: "0", transform: "translateY(20px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "pancake-slide-up": {
          "0%": { opacity: "0", transform: "translateY(30px) scale(0.9)" },
          "100%": { opacity: "1", transform: "translateY(0) scale(1)" },
        },
        "pancake-slide-down": {
          "0%": { opacity: "1", transform: "translateY(0) scale(1)" },
          "100%": { opacity: "0", transform: "translateY(30px) scale(0.9)" },
        },
        "slide-in-from-right": {
          "0%": { transform: "translateX(100%)" },
          "100%": { transform: "translateX(0)" },
        },
        "slide-out-to-right": {
          "0%": { transform: "translateX(0)" },
          "100%": { transform: "translateX(100%)" },
        },
        "slide-in-from-left": {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(0)" },
        },
        "slide-out-to-left": {
          "0%": { transform: "translateX(0)" },
          "100%": { transform: "translateX(-100%)" },
        },
        "menu-item-slide-in": {
          "0%": { opacity: "0", transform: "translateX(-20px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
        "submenu-slide-in": {
          "0%": { opacity: "0", transform: "translateX(-100%)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
        "submenu-slide-out": {
          "0%": { opacity: "1", transform: "translateX(0)" },
          "100%": { opacity: "0", transform: "translateX(-100%)" },
        },
        "permission-granted": {
          "0%": { transform: "scale(1)", backgroundColor: "hsl(142 71% 45% / 0.1)" },
          "50%": { transform: "scale(1.05)", backgroundColor: "hsl(142 71% 45% / 0.2)" },
          "100%": { transform: "scale(1)", backgroundColor: "hsl(142 71% 45% / 0.1)" },
        },
        "permission-denied": {
          "0%": { transform: "scale(1)", backgroundColor: "hsl(0 85% 60% / 0.1)" },
          "25%": { transform: "translateX(-5px)", backgroundColor: "hsl(0 85% 60% / 0.15)" },
          "75%": { transform: "translateX(5px)", backgroundColor: "hsl(0 85% 60% / 0.15)" },
          "100%": { transform: "scale(1)", backgroundColor: "hsl(0 85% 60% / 0.1)" },
        },
        "data-refresh": {
          "0%": { transform: "rotate(0deg)", opacity: "1" },
          "50%": { transform: "rotate(180deg)", opacity: "0.7" },
          "100%": { transform: "rotate(360deg)", opacity: "1" },
        },
        "shimmer": {
          "0%": { backgroundPosition: "-468px 0" },
          "100%": { backgroundPosition: "468px 0" },
        },
        "gradient-shift": {
          "0%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
          "100%": { backgroundPosition: "0% 50%" },
        },
      },
      
      animation: {
        // Legacy animations
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        
        // Enhanced admin animations
        "fade-in": "fade-in 0.5s ease-out both",
        "fade-in-delayed": "fade-in 0.5s ease-out 0.1s both",
        "fade-in-delayed-2": "fade-in 0.5s ease-out 0.2s both",
        "fade-in-delayed-3": "fade-in 0.5s ease-out 0.3s both",
        
        "slide-up": "slide-up 0.3s ease-out both",
        "pancake-slide-up": "pancake-slide-up 0.3s cubic-bezier(0.34, 1.56, 0.64, 1) both",
        "pancake-slide-down": "pancake-slide-down 0.2s ease-in both",
        
        "slide-in-from-right": "slide-in-from-right 0.3s ease-out",
        "slide-out-to-right": "slide-out-to-right 0.3s ease-in",
        "slide-in-from-left": "slide-in-from-left 0.3s ease-out",
        "slide-out-to-left": "slide-out-to-left 0.3s ease-in",
        
        "menu-item": "menu-item-slide-in 0.3s ease-out both",
        "submenu-enter": "submenu-slide-in 0.3s ease-out both",
        "submenu-exit": "submenu-slide-out 0.2s ease-in both",
        
        "permission-granted": "permission-granted 0.6s ease-out both",
        "permission-denied": "permission-denied 0.6s ease-out both",
        "data-refresh": "data-refresh 1s ease-in-out both",
        
        "shimmer": "shimmer 1s linear infinite",
        "gradient-shift": "gradient-shift 3s ease-in-out infinite",
      },
      
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
      },
      
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
        display: ['Cal Sans', 'Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}