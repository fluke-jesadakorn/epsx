/**
 * EPSX Admin - Enhanced Root Layout
 * Features: Windows Phone + PancakeSwap design system integration
 * Enhanced dark mode, spacing consistency, and accessibility
 */

import { Metadata, Viewport } from 'next';

import './globals.css';

// Force dynamic rendering to avoid context issues during static prerendering
export const dynamic = 'force-dynamic';

import { LayoutWrapper } from '@/components/layout/LayoutWrapper';
import { ClientProviders } from '@/components/providers/ClientProviders';
import { ErrorBoundary } from '@/components/providers/ErrorBoundary';
import { ThemeProvider } from '@/components/providers/ThemeProvider';
import { ToastProvider } from '@/components/providers/ToastProvider';

export const metadata: Metadata = {
  title: 'EPSX Admin',
  description: 'Administrative interface for EPSX data analytics platform - User management and system monitoring',
  keywords: 'EPSX, admin, analytics, user management, dashboard',
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

/**
 *
 * @param root0
 * @param root0.children
 */
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
      <body
        className="min-h-screen bg-background text-foreground"
        suppressHydrationWarning
      >
        {/* Theme handled by ThemeProvider to prevent FOUC securely */}

        {/* Main application wrapper with consistent spacing */}
        <div className="flex min-h-screen flex-col">
          <ErrorBoundary>
            <ThemeProvider>
              <ClientProviders>
                <main className="flex-1 relative">
                  <LayoutWrapper>
                    {children}
                  </LayoutWrapper>
                </main>
                <ToastProvider />
              </ClientProviders>
            </ThemeProvider>
          </ErrorBoundary>
        </div>


      </body>
    </html>
  );
}