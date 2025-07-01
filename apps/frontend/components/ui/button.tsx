'use client';

import { Slot } from '@radix-ui/react-slot';
import { cva  } from 'class-variance-authority';
import * as React from 'react';

import { cn } from '@/lib/utils';

import type {VariantProps} from 'class-variance-authority';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98]',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm hover:shadow-md',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm hover:shadow-md',
        outline: 'border-2 border-input bg-background hover:bg-accent hover:text-accent-foreground',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80 shadow-sm hover:shadow-md',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
        gradient: 'bg-gradient-to-r from-primary to-secondary text-primary-foreground hover:brightness-110 shadow-sm hover:shadow-md',
        success: 'bg-green-500 text-white hover:bg-green-600 shadow-sm hover:shadow-md',
        warning: 'bg-yellow-500 text-white hover:bg-yellow-600 shadow-sm hover:shadow-md',
        info: 'bg-blue-500 text-white hover:bg-blue-600 shadow-sm hover:shadow-md',
      },
      size: {
        default: 'h-10 px-4 py-2',
        xs: 'h-7 rounded px-2 text-xs',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
        xl: 'h-12 rounded-lg px-10 text-base',
        '2xl': 'h-14 rounded-lg px-12 text-lg',
        icon: 'h-10 w-10',
      },
      loading: {
        true: 'relative !text-transparent transition-none hover:!text-transparent [&>span]:visible',
        false: ''
      }
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
      loading: false
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  loading?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, loading, asChild = false, children, disabled, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, loading, className }))}
        ref={ref}
        disabled={disabled || loading}
        aria-disabled={disabled || loading}
        {...props}
      >
        {children}
        {loading && (
          <span className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 invisible">
            <svg
              className="animate-spin h-5 w-5"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          </span>
        )}
      </Comp>
    );
  }
);
Button.displayName = 'Button';

export { Button, buttonVariants };
