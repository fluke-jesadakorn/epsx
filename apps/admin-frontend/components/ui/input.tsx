/**
 * Enhanced Input Component - Windows Phone + PancakeSwap Design System
 * Features: adminInputVariants with Windows Phone styling and PancakeSwap accents
 */

import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const adminInputVariants = cva(
  "flex w-full transition-all duration-300 font-light ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 relative",
  {
    variants: {
      variant: {
        default: 
          "border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 hover:border-gray-400 dark:hover:border-gray-500",
        wp: 
          "border-2 border-secondary/30 bg-card text-foreground focus-visible:border-secondary focus-visible:ring-2 focus-visible:ring-secondary/20 hover:border-secondary/50 shadow-sm hover:shadow-md",
        pancake: 
          "border-2 border-primary/30 bg-gradient-to-r from-background to-primary/5 text-foreground focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 hover:border-primary/50 shadow-lg hover:shadow-xl",
        ghost: 
          "border-0 border-b-2 border-muted-foreground/30 bg-transparent rounded-none text-foreground focus-visible:border-primary focus-visible:ring-0 hover:border-muted-foreground/50 px-0",
        tile: 
          "border-0 bg-gradient-to-br from-card to-muted rounded-xl text-foreground focus-visible:ring-2 focus-visible:ring-primary shadow-lg hover:shadow-xl hover:scale-[1.01] active:scale-[0.99]",
        outlined:
          "border-2 border-primary bg-transparent text-foreground focus-visible:bg-primary/5 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/30 hover:border-primary/80",
      },
      size: {
        sm: "h-9 px-3 py-2 text-sm rounded-md",
        default: "h-11 px-4 py-3 text-sm rounded-lg",
        lg: "h-14 px-6 py-4 text-base rounded-xl",
        xl: "h-16 px-8 py-5 text-lg rounded-2xl",
        tile: "h-20 px-6 py-5 text-lg rounded-2xl", // Windows Phone live tile size
      },
      state: {
        default: "",
        error: "border-destructive focus-visible:border-destructive focus-visible:ring-destructive/20 text-destructive-foreground",
        success: "border-success focus-visible:border-success focus-visible:ring-success/20",
        warning: "border-warning focus-visible:border-warning focus-visible:ring-warning/20",
      }
    },
    defaultVariants: {
      variant: "wp",
      size: "default",
      state: "default",
    },
  }
)

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement>,
    VariantProps<typeof adminInputVariants> {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, variant, size, state, type, ...props }, ref) => {
    return (
      <div className="relative">
        <input
          type={type}
          className={cn(adminInputVariants({ variant, size, state, className }))}
          ref={ref}
          {...props}
        />
        
        {/* Windows Phone accent dot for certain variants */}
        {(variant === "pancake" || variant === "tile") && (
          <div className="absolute bottom-2 right-2 w-1 h-1 bg-primary/60 rounded-full pointer-events-none" />
        )}
        
        {/* PancakeSwap corner accent */}
        {variant === "pancake" && (
          <div className="absolute top-0 right-0 w-3 h-3 bg-gradient-to-bl from-primary/30 to-transparent rounded-tr-lg pointer-events-none" />
        )}
      </div>
    )
  }
)
Input.displayName = "Input"

export { Input, adminInputVariants }