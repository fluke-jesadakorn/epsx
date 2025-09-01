import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap text-sm font-light ring-offset-background transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 uppercase tracking-wider border-0",
  {
    variants: {
      variant: {
        default: "bg-gradient-to-r from-yellow-400 to-orange-500 text-black hover:from-yellow-300 hover:to-orange-400 hover:scale-[1.02] active:scale-[0.98] shadow-lg hover:shadow-xl",
        destructive:
          "bg-gradient-to-r from-red-500 to-red-700 text-white hover:from-red-400 hover:to-red-600 hover:scale-[1.02] active:scale-[0.98] shadow-lg hover:shadow-xl",
        outline:
          "border-2 border-yellow-400 bg-transparent text-yellow-600 dark:text-yellow-400 hover:bg-yellow-400 hover:text-black hover:scale-[1.02] active:scale-[0.98]",
        secondary:
          "bg-gradient-to-r from-blue-600 to-blue-700 text-white hover:from-blue-500 hover:to-blue-600 hover:scale-[1.02] active:scale-[0.98] shadow-lg hover:shadow-xl",
        ghost: "hover:bg-yellow-400/20 hover:text-yellow-600 dark:hover:text-yellow-400 hover:scale-[1.02] active:scale-[0.98]",
        link: "text-yellow-600 dark:text-yellow-400 underline-offset-4 hover:underline hover:text-yellow-500 dark:hover:text-yellow-300",
        pancake: "bg-gradient-to-r from-yellow-400 via-yellow-500 to-orange-500 text-black hover:from-yellow-300 hover:to-orange-400 hover:scale-[1.02] active:scale-[0.98] shadow-xl hover:shadow-2xl font-normal",
        wp: "bg-gradient-to-r from-blue-600 to-blue-700 text-white hover:from-blue-500 hover:to-blue-600 hover:scale-[1.02] active:scale-[0.98] shadow-xl hover:shadow-2xl font-normal",
      },
      size: {
        default: "h-11 px-6 py-3",
        sm: "h-9 px-4 py-2",
        lg: "h-14 px-8 py-4",
        icon: "h-11 w-11",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button"
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }