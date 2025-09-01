import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap text-sm font-light ring-offset-background transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 uppercase tracking-wider border-0",
  {
    variants: {
      variant: {
        default: "bg-gradient-to-r from-[#FFC107] to-[#FF8F00] text-black hover:from-[#FFD54F] hover:to-[#FFA000] hover:scale-[1.02] active:scale-[0.98] shadow-lg hover:shadow-xl",
        destructive:
          "bg-gradient-to-r from-[#D13438] to-[#B71C1C] text-white hover:from-[#E53935] hover:to-[#C62828] hover:scale-[1.02] active:scale-[0.98] shadow-lg hover:shadow-xl",
        outline:
          "border-2 border-[#FFC107] bg-transparent text-[#FFC107] hover:bg-[#FFC107] hover:text-black hover:scale-[1.02] active:scale-[0.98]",
        secondary:
          "bg-gradient-to-r from-[#0078D4] to-[#106EBE] text-white hover:from-[#1E88E5] hover:to-[#1976D2] hover:scale-[1.02] active:scale-[0.98] shadow-lg hover:shadow-xl",
        ghost: "hover:bg-[#FFC107]/20 hover:text-[#FFC107] hover:scale-[1.02] active:scale-[0.98]",
        link: "text-[#FFC107] underline-offset-4 hover:underline hover:text-[#FFD54F]",
        pancake: "bg-gradient-to-r from-[#FFC107] via-[#FFB300] to-[#FF8F00] text-black hover:from-[#FFD54F] hover:to-[#FFA000] hover:scale-[1.02] active:scale-[0.98] shadow-xl hover:shadow-2xl font-normal",
        wp: "bg-gradient-to-r from-[#0078D4] to-[#106EBE] text-white hover:from-[#1E88E5] hover:to-[#1976D2] hover:scale-[1.02] active:scale-[0.98] shadow-xl hover:shadow-2xl font-normal",
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