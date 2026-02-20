import { cva } from 'class-variance-authority';

export { cn } from '@/shared/utils/cn';
export { PancakePhoneTheme } from './pancake-phone-theme';

export const adminCardVariants = cva(
  'rounded-2xl border bg-white dark:bg-card transition-all duration-200',
  {
    variants: {
      variant: {
        default: 'border-gray-200 dark:border-border',
        pancake: 'border-gray-200 dark:border-border shadow-sm',
        glass: 'border-gray-200/50 dark:border-border backdrop-blur-sm',
      },
      hover: {
        none: '',
        glow: 'hover:shadow-lg hover:shadow-cyan-500/10 dark:hover:shadow-cyan-400/5',
        lift: 'hover:-translate-y-0.5 hover:shadow-md',
      },
      size: {
        default: '',
        sm: 'p-4',
        lg: 'p-8',
      },
    },
    defaultVariants: {
      variant: 'default',
      hover: 'none',
      size: 'default',
    },
  }
);
