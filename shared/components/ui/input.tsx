import { cn } from "@/shared/utils/cn";
import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

const inputVariants = cva(
  "flex w-full rounded-xl border px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200",
  {
    variants: {
      variant: {
        default: "border-input bg-background",
        pancake: "border-2 border-primary/30 bg-gradient-to-r from-background to-primary/5 focus-visible:border-primary hover:border-primary/50 shadow-sm",
        ghost: "border-none shadow-none focus-visible:ring-0 px-0",
        wp: "border-2 border-secondary/30 bg-card focus-visible:border-secondary focus-visible:ring-secondary/20 hover:border-secondary/50 shadow-sm",
      },
      size: {
        default: "h-10",
        sm: "h-8 px-2 text-xs",
        lg: "h-12 px-4 text-base",
        xl: "h-14 px-6 text-lg",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface InputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "size">,
    VariantProps<typeof inputVariants> {
  error?: boolean;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, variant, size, error, type, ...props }, ref) => {
    return (
      <div className="relative w-full">
        <input
          type={type}
          className={cn(
            inputVariants({ variant, size, className }),
            error && "border-destructive focus-visible:ring-destructive/30"
          )}
          ref={ref}
          {...props}
        />
        {/* Decorative elements for Pancake variant */}
        {variant === "pancake" && (
           <div className="absolute top-0 right-0 w-2 h-2 bg-gradient-to-bl from-primary/40 to-transparent rounded-tr-lg pointer-events-none" />
        )}
      </div>
    )
  }
)
Input.displayName = "Input"

export { Input, inputVariants };

