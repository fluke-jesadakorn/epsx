import { cn } from '@/lib/utils';
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';
import * as React from 'react';

const buttonVariants = cva(
  'inline-flex items-center justify-center whitespace-nowrap rounded-lg text-sm font-medium ring-offset-background transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98] relative overflow-hidden',
  {
    variants: {
      variant: {
        default:
          'bg-gradient-to-r from-primary to-primary/90 text-primary-foreground shadow-md hover:shadow-lg hover:from-primary/90 hover:to-primary/80',
        destructive:
          'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-md hover:shadow-lg',
        outline:
          'border border-input bg-background hover:bg-accent hover:text-accent-foreground shadow-sm hover:shadow-md',
        secondary:
          'bg-secondary text-secondary-foreground hover:bg-secondary/80 shadow-sm hover:shadow-md',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
        gradient:
          'bg-gradient-to-r from-primary via-secondary to-primary text-primary-foreground shadow-lg hover:shadow-xl hover:scale-105',
        pancake:
          'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg hover:shadow-xl hover:from-orange-600 hover:to-yellow-600 hover:scale-105 transition-all duration-300',
        'pancake-secondary':
          'bg-gradient-to-r from-blue-500 to-purple-500 text-white shadow-lg hover:shadow-xl hover:from-blue-600 hover:to-purple-600 hover:scale-105 transition-all duration-300',
        'pancake-outline':
          'border-2 border-orange-500 text-orange-600 dark:text-orange-400 hover:bg-orange-500 hover:text-white shadow-md hover:shadow-lg hover:scale-105 transition-all duration-300',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-lg px-8',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  },
);
Button.displayName = 'Button';

export { Button, buttonVariants };
