/**
 * Reusable style constants for home components
 */

export const GRADIENTS = {
  primary: 'from-orange-500 via-yellow-400 to-orange-600',
  secondary: 'from-blue-500 via-cyan-400 to-teal-500',
  pancake: 'from-orange-400 via-yellow-400 to-amber-500',
  background:
    'from-orange-50/30 via-yellow-50/20 to-amber-50/30 dark:from-slate-950 dark:via-orange-950/20 dark:to-yellow-950/10',
  card: 'from-white via-orange-50/20 to-yellow-50/30 dark:from-slate-900 dark:via-slate-800 dark:to-orange-900/20',
  cardHover:
    'hover:from-white hover:via-orange-50/40 hover:to-yellow-100/50 dark:hover:from-slate-800 dark:hover:via-slate-700 dark:hover:to-orange-800/30',
  badge: 'from-orange-500 to-yellow-500',
  button: 'from-orange-500 to-yellow-500',
  buttonHover: 'hover:from-yellow-500 hover:to-orange-500',
  glow: 'from-orange-400/20 via-yellow-400/30 to-amber-400/20',
  metric: {
    price:
      'from-orange-50 to-yellow-100/50 dark:from-orange-900/20 dark:to-yellow-800/20',
    eps: 'from-amber-50 to-orange-100/50 dark:from-amber-900/20 dark:to-orange-800/20',
    growth:
      'from-emerald-50 to-green-100/50 dark:from-emerald-900/20 dark:to-green-800/20',
  },
} as const;

export const ANIMATIONS = {
  float: 'animate-float',
  floatReverse: 'animate-float-reverse',
  bounceGentle: 'animate-bounce-gentle',
  pulsePancake: 'animate-pulse-pancake',
  spinSlow: 'animate-spin-slow',
  scaleHover: 'hover:scale-[1.02]',
  scalePancake: 'hover:scale-[1.05]',
  scalePress: 'scale-[0.98]',
  fadeIn: 'animate-in slide-in-from-bottom-4 fade-in',
  slideInLeft: 'animate-in slide-in-from-left-8 fade-in',
  slideInRight: 'animate-in slide-in-from-right-8 fade-in',
  wiggle: 'animate-wiggle',
  shine: 'animate-shine',
} as const;

export const COLORS = {
  positive: {
    bg: 'bg-emerald-100 dark:bg-emerald-900/50',
    text: 'text-emerald-600 dark:text-emerald-400',
    border: 'border-emerald-200/50 dark:border-emerald-700/30',
  },
  negative: {
    bg: 'bg-red-100 dark:bg-red-900/50',
    text: 'text-red-600 dark:text-red-400',
    border: 'border-red-200/50 dark:border-red-700/30',
  },
  neutral: {
    bg: 'bg-slate-100 dark:bg-slate-800/50',
    text: 'text-slate-600 dark:text-slate-400',
    border: 'border-slate-200/50 dark:border-slate-700/50',
  },
  primary: {
    bg: 'bg-blue-100 dark:bg-blue-900/50',
    text: 'text-blue-600 dark:text-blue-400',
    border: 'border-blue-200/50 dark:border-blue-700/30',
  },
  secondary: {
    bg: 'bg-purple-100 dark:bg-purple-900/50',
    text: 'text-purple-600 dark:text-purple-400',
    border: 'border-purple-200/50 dark:border-purple-700/30',
  },
} as const;

export const SPACING = {
  cardPadding: 'p-5 pt-6',
  sectionGap: 'gap-8',
  gridGap: 'gap-4',
  itemGap: 'gap-3',
  containerPadding: 'px-6 sm:px-12',
  verticalSpacing: 'py-12',
  responsiveGap: 'gap-4 sm:gap-6 md:gap-8',
  responsivePadding: 'p-4 sm:p-6 md:p-8',
  mobileContainer: 'px-4 sm:px-6 md:px-12',
} as const;

export const TYPOGRAPHY = {
  hero: 'text-3xl sm:text-4xl md:text-5xl lg:text-6xl xl:text-7xl font-black',
  title: 'text-xl sm:text-2xl md:text-3xl font-black',
  subtitle: 'text-base sm:text-lg md:text-xl',
  body: 'text-sm font-semibold',
  caption: 'text-xs sm:text-sm',
  price: 'text-sm sm:text-base font-bold',
  metric: 'text-base sm:text-lg md:text-xl font-bold',
  cardTitle: 'text-lg sm:text-xl md:text-2xl font-bold',
  sectionTitle: 'text-2xl sm:text-3xl md:text-4xl font-bold',
} as const;

export const BREAKPOINTS = {
  mobile: 'sm:',
  tablet: 'md:',
  desktop: 'lg:',
  largeDesktop: 'xl:',
} as const;
