"use client"

import * as DialogPrimitive from "@radix-ui/react-dialog";
import { clsx } from 'clsx';
import { X } from "lucide-react";
import * as React from "react";
import { twMerge } from 'tailwind-merge';

// Local cn function to avoid circular dependencies
function cn(...inputs: (string | undefined | null | false)[]) {
  return twMerge(clsx(inputs));
}

const Dialog = DialogPrimitive.Root

const DialogTrigger = DialogPrimitive.Trigger

const DialogPortal = DialogPrimitive.Portal

const DialogClose = DialogPrimitive.Close

export interface DialogOverlayProps
  extends React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay> {
  /** Enable fade animations on open/close */
  animated?: boolean;
}

const DialogOverlay = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  DialogOverlayProps
>(({ className, animated = true, ...props }, ref) => (
  <DialogPrimitive.Overlay
    ref={ref}
    className={cn(
      "fixed inset-0 backdrop-blur-sm",
      animated && "state-open:animate-fade-in state-closed:animate-fade-out",
      className
    )}
    style={{ zIndex: 1090, backgroundColor: 'rgba(0,0,0,0.5)' }}
    {...props}
  />
))
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName

export interface DialogContentProps
  extends React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content> {
  /** Enable slide/zoom animations on open/close */
  animated?: boolean;
  /** Show close button */
  showClose?: boolean;
}

const DialogContent = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  DialogContentProps
>(({ className, children, animated = true, showClose = true, style, ...props }, ref) => (
  <DialogPortal>
    <DialogOverlay animated={animated} />
    <div className="fixed inset-0 flex items-center justify-center p-4" style={{ zIndex: 1100 }}>
      <DialogPrimitive.Content
        ref={ref}
        className={cn(
          "grid w-full max-w-lg gap-6 border border-gray-200 dark:border-slate-700 bg-white dark:bg-[#0B0F1A] text-foreground p-8 shadow-[0_0_50px_-12px_rgba(0,0,0,0.5)] rounded-2xl sm:rounded-3xl outline-none",
          animated && "state-open:animate-zoom-in state-closed:animate-zoom-out duration-300",
          className
        )}
        style={style}
        {...props}
      >
        <DialogPrimitive.Title className="sr-only">Dialog</DialogPrimitive.Title>
        {children}
        {showClose && (
          <DialogPrimitive.Close className="absolute right-4 top-4 rounded-lg bg-white dark:bg-white/5 backdrop-blur-sm border border-gray-200 dark:border-slate-700 opacity-70 ring-offset-background transition-all hover:opacity-100 hover:bg-black/[0.05] dark:hover:bg-white/10 focus:outline-none focus:ring-2 focus:ring-purple-500/50 focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground">
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Content>
    </div>
  </DialogPortal>
))
DialogContent.displayName = DialogPrimitive.Content.displayName

const DialogHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col space-y-1.5 text-center sm:text-left",
      className
    )}
    {...props}
  />
)
DialogHeader.displayName = "Dialogheader"

const DialogFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
      className
    )}
    {...props}
  />
)
DialogFooter.displayName = "DialogFooter"

const DialogTitle = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Title>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Title
    ref={ref}
    className={cn(
      "text-lg font-semibold leading-none tracking-tight",
      className
    )}
    {...props}
  />
))
DialogTitle.displayName = DialogPrimitive.Title.displayName

const DialogDescription = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Description>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Description
    ref={ref}
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
DialogDescription.displayName = DialogPrimitive.Description.displayName

export {
  Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogOverlay, DialogPortal, DialogTitle, DialogTrigger
};

