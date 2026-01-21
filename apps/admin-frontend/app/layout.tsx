import '@/lib/polyfills';
/**
 * EPSX Admin - Enhanced Root Layout
 * Features: Windows Phone + PancakeSwap design system integration
 * Enhanced dark mode, spacing consistency, and accessibility
 */

import { Metadata, Viewport } from 'next';

import './globals.css';

import { LayoutWrapper } from '@/components/layout/LayoutWrapper';
import { ClientProviders } from '@/components/providers/ClientProviders';
import { ErrorBoundary } from '@/components/providers/ErrorBoundary';
import { ToastProvider } from '@/components/providers/ToastProvider';
import { SafeThemeScript } from '@/shared/components/ui/SafeThemeScript';
import { Kanit } from 'next/font/google';

const kanit = Kanit({
  subsets: ['latin'],
  weight: ['200', '300', '400', '500', '600', '700', '800'],
  display: 'swap',
  variable: '--font-kanit',
});

// Force dynamic rendering to avoid context issues during static prerendering
export const dynamic = 'force-dynamic';

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
    { media: '(prefers-color-scheme: light)', color: '#ffffff' },
    { media: '(prefers-color-scheme: dark)', color: '#000000' }
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
      className={`${kanit.variable} font-sans antialiased scroll-smooth`}
    >
      <head>
        {/* CRITICAL: Synchronous BigInt polyfill to prevent Math.pow crash in dependencies like viem */}
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                if (typeof Math.pow !== 'function' || Math.pow.__isPolyfilled) return;
                var originalPow = Math.pow;
                Math.pow = function(base, exponent) {
                  if (typeof base === 'bigint' || typeof exponent === 'bigint') {
                    try {
                      return new Function('b', 'e', 'return b ** e')(base, exponent);
                    } catch (e) {
                      if (typeof base === 'bigint' && typeof exponent === 'number' && exponent >= 0 && Math.floor(exponent) === exponent) {
                        var res = 1n;
                        for (var i = 0; i < exponent; i++) res *= base;
                        return res;
                      }
                      return NaN;
                    }
                  }
                  return originalPow.call(Math, base, exponent);
                };
                Math.pow.__isPolyfilled = true;
              })();
            `,
          }}
        />
        {/* Theme initialization script - prevents FOUC */}
        <SafeThemeScript />
      </head>
      <body
        className="h-screen bg-background text-foreground overflow-hidden"
        suppressHydrationWarning
      >
        {/* Theme handled by CommonProviders (inside ClientProviders) */}

        {/* Enhanced Background Mesh - Matches Frontend */}
        <div className="fixed inset-0 -z-10 bg-background">
          <div className="absolute inset-0 bg-grid-pattern opacity-[0.03] dark:opacity-[0.05]" />
          <div className="absolute top-0 -left-1/4 w-1/2 h-1/2 bg-blue-500/10 blur-[120px] rounded-full mix-blend-screen dark:mix-blend-screen" />
          <div className="absolute bottom-0 -right-1/4 w-1/2 h-1/2 bg-indigo-500/10 blur-[120px] rounded-full mix-blend-screen dark:mix-blend-screen" />
        </div>

        {/* Main application wrapper with consistent spacing */}
        <div className="flex h-screen flex-col overflow-hidden">
          <ErrorBoundary>
            <ClientProviders>
              <main className="flex-1 relative overflow-hidden">
                <LayoutWrapper>
                  {children}
                </LayoutWrapper>
              </main>
              <ToastProvider />
            </ClientProviders>
          </ErrorBoundary>
        </div>

      </body>
    </html>
  );
}