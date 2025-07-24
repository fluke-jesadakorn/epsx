import { cva } from "class-variance-authority";

export const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap rounded-lg text-sm font-medium ring-offset-background transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98] relative overflow-hidden",
  {
    variants: {
      variant: {
        default:
          "bg-gradient-to-r from-primary to-primary/90 text-primary-foreground shadow-md hover:shadow-lg hover:from-primary/90 hover:to-primary/80",
        destructive:
          "bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-md hover:shadow-lg",
        outline:
          "border border-input bg-background hover:bg-accent hover:text-accent-foreground shadow-sm hover:shadow-md",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-secondary/80 shadow-sm hover:shadow-md",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
        gradient:
          "bg-gradient-to-r from-primary via-secondary to-primary text-primary-foreground shadow-lg hover:shadow-xl hover:scale-105",
        pancake:
          "pancake-gradient text-white shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-300 font-semibold",
        "pancake-secondary":
          "pancake-gradient-secondary text-white shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-300 font-semibold",
        "pancake-outline":
          "border-2 border-pancake-primary bg-transparent text-pancake-primary hover:bg-pancake-primary hover:text-white shadow-md hover:shadow-lg hover:scale-105 transition-all duration-300 font-semibold",
        "pancake-ghost":
          "bg-transparent text-pancake-primary hover:bg-pancake-primary/10 hover:text-pancake-primary transition-all duration-200 font-medium",
        "pancake-soft":
          "pancake-gradient-soft-highlight text-pancake-primary shadow-sm hover:shadow-md hover:scale-105 transition-all duration-300 font-medium",
        // Trading-specific variants
        bullish:
          "bg-gradient-to-r from-green-500 to-green-600 text-white shadow-lg hover:shadow-xl hover:scale-105 hover:from-green-600 hover:to-green-700 transition-all duration-300 font-semibold",
        bearish:
          "bg-gradient-to-r from-red-500 to-red-600 text-white shadow-lg hover:shadow-xl hover:scale-105 hover:from-red-600 hover:to-red-700 transition-all duration-300 font-semibold",
        neutral:
          "bg-gradient-to-r from-gray-500 to-gray-600 text-white shadow-lg hover:shadow-xl hover:scale-105 hover:from-gray-600 hover:to-gray-700 transition-all duration-300 font-semibold",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3 text-xs",
        lg: "h-11 rounded-lg px-8",
        xl: "h-12 rounded-xl px-10 text-base",
        icon: "h-10 w-10",
        "icon-sm": "h-8 w-8",
        "icon-lg": "h-12 w-12",
      },
      responsive: {
        true: "xs:h-8 xs:px-2 xs:text-xs sm:h-10 sm:px-4 sm:text-sm",
        false: "",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
      responsive: false,
    },
  }
);
