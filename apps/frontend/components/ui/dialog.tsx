"use client"

import * as DialogPrimitive from "@radix-ui/react-dialog"
import * as React from "react"

import { cn } from "@/lib/utils"

const Dialog = (props: DialogPrimitive.DialogProps) => {
  const Root = DialogPrimitive.Root as React.ComponentType<DialogPrimitive.DialogProps>
  return <Root {...props} />
}

const DialogTrigger = (props: DialogPrimitive.DialogTriggerProps) => {
  const Trigger = DialogPrimitive.Trigger as React.ComponentType<DialogPrimitive.DialogTriggerProps>
  return <Trigger {...props} />
}

const DialogContent = ({
  className,
  children,
  ...props
}: DialogPrimitive.DialogContentProps & { children: React.ReactNode }) => {
  const Portal = DialogPrimitive.Portal as React.ComponentType<DialogPrimitive.DialogPortalProps>
  const Overlay = DialogPrimitive.Overlay as React.ComponentType<DialogPrimitive.DialogOverlayProps>
  const Content = DialogPrimitive.Content as React.ComponentType<DialogPrimitive.DialogContentProps>
  const Close = DialogPrimitive.Close as React.ComponentType<DialogPrimitive.DialogCloseProps>

  return (
    <Portal>
      <Overlay 
        className={cn(
          "fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
        )}
      />
      <Content
        className={cn(
          "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",
          className
        )}
        {...props}
      >
        {children}
        <Close className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground">
          <svg
            className="h-4 w-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth={1.5}
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
          <span className="sr-only">Close</span>
        </Close>
      </Content>
    </Portal>
  )
}

const DialogHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => {
  return (
    <div
      className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)}
      {...props}
    />
  )
}

const DialogFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => {
  return (
    <div
      className={cn(
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
        className
      )}
      {...props}
    />
  )
}

const DialogTitle = ({
  className,
  ...props
}: DialogPrimitive.DialogTitleProps) => {
  const Title = DialogPrimitive.Title as React.ComponentType<DialogPrimitive.DialogTitleProps>
  return (
    <Title
      className={cn("text-lg font-semibold leading-none tracking-tight", className)}
      {...props}
    />
  )
}

const DialogDescription = ({
  className,
  ...props
}: DialogPrimitive.DialogDescriptionProps) => {
  const Description = DialogPrimitive.Description as React.ComponentType<DialogPrimitive.DialogDescriptionProps>
  return (
    <Description
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  )
}

export {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
}
