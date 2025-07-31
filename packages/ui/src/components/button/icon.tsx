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
  active?: boolean;
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
    active = false,
    "aria-label": ariaLabel,
    ...props 
  }, ref) => {
    const buttonElement = (
      <button
        className={cn(
          buttonVariants({ variant, size }), 
          "rounded-full focus-visible:ring-4 focus-visible:ring-accent/10 focus-visible:outline-1",
          "[&_svg:not([class*='text-'])]:text-muted-foreground",
          "[&_svg:not([class*='size-'])]:size-4",
          active && "bg-accent/50 text-accent-foreground",
          className
        )}
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

    return buttonElement;
  }
);

ButtonIcon.displayName = "ButtonIcon";

export { ButtonIcon };
