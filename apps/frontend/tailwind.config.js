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
        // CSS variable-based colors (maintain compatibility)
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
          50: "hsl(31 100% 95%)",
          100: "hsl(31 90% 90%)",
          200: "hsl(31 95% 85%)",
          300: "hsl(31 95% 75%)",
          400: "hsl(31 95% 65%)",
          500: "hsl(31 100% 50%)",
          600: "hsl(31 95% 45%)",
          700: "hsl(31 90% 40%)",
          800: "hsl(31 85% 35%)",
          900: "hsl(31 80% 30%)",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
          50: "hsl(42 100% 95%)",
          100: "hsl(42 95% 88%)",
          200: "hsl(42 90% 80%)",
          300: "hsl(42 95% 75%)",
          400: "hsl(42 100% 70%)",
          500: "hsl(42 100% 70%)",
          600: "hsl(42 95% 65%)",
          700: "hsl(42 90% 60%)",
          800: "hsl(42 85% 55%)",
          900: "hsl(42 80% 50%)",
        },
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
        
        // PancakeSwap-inspired color extensions
        pancake: {
          primary: "hsl(31 100% 50%)",
          secondary: "hsl(42 100% 70%)",
          success: "hsl(142 71% 45%)",
          warning: "hsl(45 93% 47%)",
          error: "hsl(0 85% 60%)",
          info: "hsl(217 91% 60%)",
        },
        
        // Extended status colors
        success: {
          50: "hsl(142 76% 95%)",
          100: "hsl(142 71% 88%)",
          200: "hsl(142 71% 75%)",
          300: "hsl(142 71% 65%)",
          400: "hsl(142 71% 55%)",
          500: "hsl(142 71% 45%)",
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
          500: "hsl(45 93% 47%)",
          600: "hsl(45 88% 42%)",
          700: "hsl(45 83% 37%)",
          800: "hsl(45 78% 32%)",
          900: "hsl(45 73% 27%)",
        },
        
        error: {
          50: "hsl(0 90% 95%)",
          100: "hsl(0 85% 88%)",
          200: "hsl(0 85% 75%)",
          300: "hsl(0 85% 70%)",
          400: "hsl(0 85% 65%)",
          500: "hsl(0 85% 60%)",
          600: "hsl(0 80% 55%)",
          700: "hsl(0 75% 50%)",
          800: "hsl(0 70% 45%)",
          900: "hsl(0 65% 40%)",
        },
        
        info: {
          50: "hsl(217 96% 95%)",
          100: "hsl(217 91% 88%)",
          200: "hsl(217 91% 75%)",
          300: "hsl(217 91% 70%)",
          400: "hsl(217 91% 65%)",
          500: "hsl(217 91% 60%)",
          600: "hsl(217 86% 55%)",
          700: "hsl(217 81% 50%)",
          800: "hsl(217 76% 45%)",
          900: "hsl(217 71% 40%)",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        // UI component animations
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        "collapsible-down": {
          from: { height: "0" },
          to: { height: "var(--collapsible-content-height)" },
        },
        "collapsible-up": {
          from: { height: "var(--collapsible-content-height)" },
          to: { height: "0" },
        },
        
        // Float animations
        "float": {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-20px)" },
        },
        "float-gentle": {
          "0%, 100%": { transform: "translateY(0) rotate(0deg)" },
          "33%": { transform: "translateY(-8px) rotate(1deg)" },
          "66%": { transform: "translateY(8px) rotate(-1deg)" },
        },
        "float-reverse": {
          "0%, 100%": { transform: "translateY(0) rotate(0deg)" },
          "50%": { transform: "translateY(15px) rotate(2deg)" },
        },
        
        // Bounce animations
        "bounce-gentle": {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-8px)" },
        },
        "bounce-slow": {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-15px)" },
        },
        
        // Pulse animations
        "pulse-gentle": {
          "0%, 100%": { opacity: "0.6", transform: "scale(1)" },
          "50%": { opacity: "1", transform: "scale(1.05)" },
        },
        "pulse-slow": {
          "0%, 100%": { opacity: "0.4" },
          "50%": { opacity: "0.8" },
        },
        "pulse-pancake": {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.7" },
        },
        
        // Scale animations
        "scale-pulse": {
          "0%, 100%": { transform: "scale(1)" },
          "50%": { transform: "scale(1.05)" },
        },
        
        // Rotation animations
        "spin-slow": {
          from: { transform: "rotate(0deg)" },
          to: { transform: "rotate(360deg)" },
        },
        
        // Movement animations
        "wiggle": {
          "0%": { transform: "rotate(0deg)" },
          "25%": { transform: "rotate(1deg)" },
          "75%": { transform: "rotate(-1deg)" },
          "100%": { transform: "rotate(0deg)" },
        },
        
        // Gradient animations
        "gradient": {
          "0%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
          "100%": { backgroundPosition: "0% 50%" },
        },
        "gradient-x": {
          "0%, 100%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
        },
        "gradient-shift": {
          "0%, 100%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
        },
        
        // Shine and shimmer
        "shine": {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(100%)" },
        },
        "shimmer": {
          from: { transform: "translateX(-100%)" },
          to: { transform: "translateX(100%)" },
        },
        
        // Slide animations
        "slide-up": {
          from: { opacity: "0", transform: "translateY(30px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        "slide-in-left": {
          from: { opacity: "0", transform: "translateX(-20px)" },
          to: { opacity: "1", transform: "translateX(0)" },
        },
        "slide-in-right": {
          from: { opacity: "0", transform: "translateX(20px)" },
          to: { opacity: "1", transform: "translateX(0)" },
        },
        
        // Fade animations
        "fade-in": {
          from: { opacity: "0", transform: "translateY(20px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        
        // Blob animations for decorative elements
        "blob1": {
          "0%, 100%": { transform: "scale(1) translate(0, 0)" },
          "33%": { transform: "scale(1.1, 0.9) translate(10px, -10px)" },
          "66%": { transform: "scale(0.95, 1.05) translate(-10px, 10px)" },
        },
        "blob2": {
          "0%, 100%": { transform: "scale(1) translate(0, 0)" },
          "33%": { transform: "scale(1.05, 0.95) translate(-12px, 8px)" },
          "66%": { transform: "scale(0.9, 1.1) translate(12px, -8px)" },
        },
        "blob3": {
          "0%, 100%": { transform: "scale(1) translate(0, 0)" },
          "33%": { transform: "scale(1.08, 0.92) translate(8px, 12px)" },
          "66%": { transform: "scale(0.92, 1.08) translate(-8px, -12px)" },
        },
      },
      animation: {
        // UI component animations
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "collapsible-down": "collapsible-down 0.2s ease-out",
        "collapsible-up": "collapsible-up 0.2s ease-out",
        
        // Float animations
        "float": "float 6s ease-in-out infinite",
        "float-gentle": "float-gentle 4s ease-in-out infinite",
        "float-reverse": "float-reverse 5s ease-in-out infinite",
        "float-delayed": "float 6s ease-in-out infinite -3s",
        
        // Bounce animations
        "bounce-gentle": "bounce-gentle 2s ease-in-out infinite",
        "bounce-slow": "bounce-slow 3s ease-in-out infinite",
        
        // Pulse animations
        "pulse-gentle": "pulse-gentle 3s ease-in-out infinite",
        "pulse-slow": "pulse-slow 4s ease-in-out infinite",
        "pulse-pancake": "pulse-pancake 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        
        // Scale animations
        "scale-pulse": "scale-pulse 3s ease-in-out infinite",
        
        // Rotation animations
        "spin-slow": "spin-slow 20s linear infinite",
        
        // Movement animations
        "wiggle": "wiggle 1s ease-in-out infinite",
        
        // Gradient animations
        "gradient": "gradient 8s ease infinite",
        "gradient-x": "gradient-x 3s ease infinite",
        "gradient-shift": "gradient-shift 3s ease-in-out infinite",
        
        // Shine and shimmer
        "shine": "shine 2s linear infinite",
        "shimmer": "shimmer 2s linear infinite",
        
        // Slide animations
        "slide-up": "slide-up 0.8s ease-out",
        "slide-up-delayed": "slide-up 0.8s ease-out 0.2s both",
        "slide-up-delayed-2": "slide-up 0.8s ease-out 0.4s both",
        "slide-in-left": "slide-in-left 0.6s ease-out",
        "slide-in-right": "slide-in-right 0.6s ease-out",
        
        // Fade animations
        "fade-in": "fade-in 1s ease-out forwards",
        "fade-in-delayed": "fade-in 1s ease-out 0.3s forwards",
        "fade-in-delayed-2": "fade-in 1s ease-out 0.6s forwards",
        "fade-in-delayed-3": "fade-in 1s ease-out 0.9s forwards",
        
        // Blob decorative animations
        "blob1": "blob1 12s ease-in-out infinite",
        "blob2": "blob2 14s ease-in-out infinite",
        "blob3": "blob3 16s ease-in-out infinite",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}