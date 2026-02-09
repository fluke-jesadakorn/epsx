import { cn } from "@shared/utils/cn";
import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

const inputVariants = cva(
  "flex w-full rounded-xl border px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200",
  {
    variants: {
      variant: {
        default: "border-input bg-background",
        dark: "border-gray-700 bg-[#1a1a1a] text-foreground placeholder:text-gray-500 focus-visible:border-purple-500 focus-visible:ring-purple-500/20",
        ghost: "border-none shadow-none focus-visible:ring-0 px-0",
        search: "border-gray-700 bg-slate-950/50 text-slate-200 placeholder:text-slate-600 focus-visible:border-blue-500/50 h-8 text-xs",
        glass: "bg-white/5 backdrop-blur-sm border-white/20 text-foreground placeholder:text-muted-foreground focus-visible:border-purple-500/50 focus-visible:ring-purple-500/20",
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
      <input
        type={type}
        className={cn(
          inputVariants({ variant, size, className }),
          Boolean(error) && "border-destructive focus-visible:ring-destructive/30"
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input, inputVariants };
