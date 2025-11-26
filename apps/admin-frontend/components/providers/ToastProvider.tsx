'use client'

import { Toaster } from 'react-hot-toast'

export function ToastProvider() {
  return (
    <Toaster
      position="top-right"
      reverseOrder={false}
      gutter={12}
      containerStyle={{
        top: 24,
        right: 24,
        zIndex: 99999,
      }}
      toastOptions={{
        duration: 5000,
        style: {
          borderRadius: '12px',
          background: 'hsl(220 26% 8%)',
          color: 'hsl(45 100% 95%)',
          border: '1px solid hsl(45 15% 20%)',
          padding: '16px 20px',
          fontSize: '14px',
          fontWeight: '500',
          boxShadow: '0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
          backdropFilter: 'blur(8px)',
          position: 'relative',
          overflow: 'hidden',
        },
        success: {
          style: {
            background: 'linear-gradient(135deg, hsl(142 71% 45% / 0.95) 0%, hsl(142 71% 50% / 0.95) 100%)',
            color: 'white',
            border: '1px solid hsl(142 71% 45%)',
          },
          iconTheme: {
            primary: 'white',
            secondary: 'hsl(142 71% 45%)',
          },
        },
        error: {
          style: {
            background: 'linear-gradient(135deg, hsl(0 85% 60% / 0.95) 0%, hsl(0 85% 65% / 0.95) 100%)',
            color: 'white',
            border: '1px solid hsl(0 85% 60%)',
          },
          iconTheme: {
            primary: 'white',
            secondary: 'hsl(0 85% 60%)',
          },
        },
        loading: {
          style: {
            background: 'linear-gradient(135deg, hsl(217 91% 65% / 0.95) 0%, hsl(213 94% 73% / 0.95) 100%)',
            color: 'white',
            border: '1px solid hsl(217 91% 65%)',
          },
          iconTheme: {
            primary: 'white',
            secondary: 'hsl(217 91% 65%)',
          },
        },
        blank: {
          style: {
            background: 'linear-gradient(135deg, hsl(47 100% 63% / 0.95) 0%, hsl(45 100% 58% / 0.95) 100%)',
            color: 'hsl(220 26% 8%)',
            border: '1px solid hsl(47 100% 63%)',
          },
        },
      }}
    />
  )
}
