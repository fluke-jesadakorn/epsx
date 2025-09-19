/**
 * EPSX Admin - Enhanced Root Layout
 * Features: Windows Phone + PancakeSwap design system integration
 * Enhanced dark mode, spacing consistency, and accessibility
 */

import './globals.css';
import { AuthProvider } from '@/components/providers/AuthProvider';
import { ErrorBoundary } from '@/components/providers/ErrorBoundary';
import { ServiceWorkerInitializer } from '@/components/ServiceWorkerInitializer';
import { Toaster } from 'react-hot-toast';
import { Metadata, Viewport } from 'next';

export const metadata: Metadata = {
  title: 'EPSX Admin',
  description: 'Administrative interface for EPSX trading platform - User management, analytics, and system monitoring',
  keywords: 'EPSX, admin, analytics, trading, user management, dashboard',
  authors: [{ name: 'EPSX Team' }],
  creator: 'EPSX',
  publisher: 'EPSX',
  formatDetection: {
    email: false,
    address: false,
    telephone: false,
  },
  robots: {
    index: false,
    follow: false,
    noarchive: true,
    nosnippet: true,
  },
};

export const viewport: Viewport = {
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: 'hsl(47 100% 63%)' },
    { media: '(prefers-color-scheme: dark)', color: 'hsl(220 26% 3%)' }
  ],
  colorScheme: 'light dark',
  width: 'device-width',
  initialScale: 1,
  maximumScale: 5,
  userScalable: true,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html 
      lang="en" 
      suppressHydrationWarning
      className="font-sans antialiased scroll-smooth"
    >
      <body className="min-h-screen bg-background text-foreground">
        {/* Prevent flash of unstyled content */}
        <script
          dangerouslySetInnerHTML={{
            __html: `
              try {
                if (localStorage.theme === 'dark' || (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
                  document.documentElement.classList.add('dark')
                } else {
                  document.documentElement.classList.remove('dark')
                }
              } catch (_) {}
            `,
          }}
        />
        
        {/* Main application wrapper with consistent spacing */}
        <div className="flex min-h-screen flex-col">
          <ErrorBoundary>
            <AuthProvider>
              <main className="flex-1 relative">
                {children}
              </main>
            </AuthProvider>
          </ErrorBoundary>
        </div>

        {/* Service Worker for FCM notifications */}
        <ServiceWorkerInitializer />

        {/* Enhanced Windows Phone + PancakeSwap Toast Notifications */}
        <Toaster
          position="top-right"
          reverseOrder={false}
          gutter={12}
          containerStyle={{
            top: 24,
            right: 24,
            zIndex: 9998,
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
      </body>
    </html>
  );
}