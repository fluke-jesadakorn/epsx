"use client";

import * as React from "react";

import { cn } from "../../lib/utils";

import { buttonVariants } from "./variants";

import type { VariantProps } from "class-variance-authority";

export interface ButtonIconProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  children?: React.ReactNode;
  tooltip?: string;
  srLabel?: string; // Screen reader label for accessibility
}

const ButtonIcon = React.forwardRef<HTMLButtonElement, ButtonIconProps>(
  ({ 
    className, 
    variant = "ghost", 
    size = "icon", 
    asChild: _asChild = false, 
    children, 
    tooltip,
    srLabel,
    "aria-label": ariaLabel,
    ...props 
  }, ref) => {
    const buttonElement = (
      <button
        className={cn(
          buttonVariants({ variant, size }), 
          "rounded-full focus:ring-ring/10 dark:focus:ring-ring/20 dark:outline-ring/40 outline-ring/50",
          "[&_svg:not([class*='text-'])]:text-muted-foreground",
          "[&_svg:not([class*='size-'])]:size-4",
          "transition-colors hover:bg-primary/10 text-muted-foreground hover:text-primary",
          className
        )}
        ref={ref}
        aria-label={ariaLabel || srLabel}
        title={tooltip}
        {...props}
      >
        {children}
        {srLabel && !ariaLabel && (
          <span className="sr-only">{srLabel}</span>
        )}
      </button>
    );

    return buttonElement;
  }
);

ButtonIcon.displayName = "ButtonIcon";

export { ButtonIcon };
