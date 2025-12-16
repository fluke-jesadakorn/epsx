'use client'

import { useTheme } from 'next-themes'
import { Toaster } from 'sonner'

export function ToastProvider() {
  const { theme = 'system' } = useTheme()

  return (
    <Toaster
      theme={theme as 'light' | 'dark' | 'system'}
      className="toaster group"
      toastOptions={{
        classNames: {
          toast: 'group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg',
          description: 'group-[.toast]:text-muted-foreground',
          actionButton: 'group-[.toast]:bg-primary group-[.toast]:text-primary-foreground',
          cancelButton: 'group-[.toast]:bg-muted group-[.toast]:text-muted-foreground',
          error: 'group-[.toaster]:bg-red-500/10 group-[.toaster]:text-red-500 group-[.toaster]:border-red-500/20',
          success: 'group-[.toaster]:bg-green-500/10 group-[.toaster]:text-green-500 group-[.toaster]:border-green-500/20',
          warning: 'group-[.toaster]:bg-yellow-500/10 group-[.toaster]:text-yellow-500 group-[.toaster]:border-yellow-500/20',
          info: 'group-[.toaster]:bg-blue-500/10 group-[.toaster]:text-blue-500 group-[.toaster]:border-blue-500/20',
        },
        duration: 5000,
      }}
      position="top-right"
    />
  )
}
