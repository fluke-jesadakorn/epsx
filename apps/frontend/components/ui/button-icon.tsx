'use client';

import * as React from 'react';
import { cva } from 'class-variance-authority';
import { cn } from '@/lib/utils';
import type { VariantProps } from 'class-variance-authority';

const buttonIconVariants = cva(
  'inline-flex items-center justify-center rounded-full text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-ring/10 focus-visible:outline-1 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98] relative overflow-hidden [&_svg:not([class*=\'text-\'])]:text-muted-foreground [&_svg:not([class*=\'size-\'])]:size-4',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-primary/10 text-muted-foreground hover:text-primary',
        link: 'text-primary hover:underline',
      },
      size: {
        default: 'h-10 w-10',
        sm: 'h-8 w-8',
        lg: 'h-12 w-12',
        xl: 'h-14 w-14',
      },
      active: {
        true: 'bg-accent/50 text-accent-foreground',
        false: ''
      }
    },
    defaultVariants: {
      variant: 'ghost',
      size: 'default',
      active: false
    },
  }
);

export interface ButtonIconProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonIconVariants> {
  asChild?: boolean;
  tooltip?: string;
  srLabel?: string; // Screen reader label for accessibility
  active?: boolean;
}

const ButtonIcon = React.forwardRef<HTMLButtonElement, ButtonIconProps>(
  ({ 
    className, 
    variant, 
    size, 
    active,
    asChild = false, 
    children, 
    tooltip,
    srLabel,
    'aria-label': ariaLabel,
    ...props 
  }, ref) => {
    return (
      <button
        className={cn(buttonIconVariants({ variant, size, active, className }))}
        ref={ref}
        aria-label={ariaLabel || srLabel}
        title={tooltip}
        data-active={active}
        {...props}
      >
        {children}
        {srLabel && !ariaLabel && (
          <span className="sr-only">{srLabel}</span>
        )}
      </button>
    );
  }
);

ButtonIcon.displayName = 'ButtonIcon';

export { ButtonIcon, buttonIconVariants };
