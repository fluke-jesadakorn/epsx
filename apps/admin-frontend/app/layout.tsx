import '@/lib/polyfills';
/**
 * EPSX Admin - Root Layout
 * Modern, clean admin interface with system fonts
 */

import { Metadata, Viewport } from 'next';

import './globals.css';

import { LayoutWrapper } from '@/components/layout/LayoutWrapper';
import { ClientProviders } from '@/components/providers/ClientProviders';
import { ErrorBoundary } from '@/components/providers/ErrorBoundary';
import { ToastProvider } from '@/components/providers/ToastProvider';
import { SafeThemeScript } from '@/shared/components/ui/SafeThemeScript';
import { getServerConfig } from '@/shared/config/wagmi';
import { Kanit } from 'next/font/google';
import { headers } from 'next/headers';
import { cookieToInitialState } from 'wagmi';

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

import { getAuthUser } from '@/lib/server/auth';

/**
 *
 * @param root0
 * @param root0.children
 */
export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Server-side auth check to seed client state
  const user = await getAuthUser();

  // Unified Web3 Cookie Hydration
  const headersList = await headers();
  const rawCookie = headersList.get('cookie');
  const cookie = rawCookie ? decodeURIComponent(rawCookie) : rawCookie;
  const initialState = cookieToInitialState(getServerConfig(), cookie);

  return (
    <html
      lang="en"
      suppressHydrationWarning
      className="antialiased scroll-smooth"
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
        className={`${kanit.variable} h-screen bg-[#08060b] text-foreground overflow-hidden font-sans`}
        suppressHydrationWarning
      >
        {/* Modern, deep background system */}
        <div className="fixed inset-0 z-[-1] overflow-hidden pointer-events-none">
          {/* Base gradient */}
          <div className="absolute inset-0 bg-gradient-to-br from-[#08060b] via-[#120c1d] to-[#08060b]" />

          {/* Subdued Grid */}
          <div className="absolute inset-0 bg-grid-pattern opacity-40" />

          {/* Decorative blurry blobs */}
          <div className="absolute -top-[10%] -left-[10%] w-[40%] h-[40%] bg-[#7645d9]/10 rounded-full blur-[120px] animate-pulse" />
          <div className="absolute top-[20%] -right-[5%] w-[30%] h-[30%] bg-[#1fc7d4]/5 rounded-full blur-[100px]" />
          <div className="absolute -bottom-[10%] left-[20%] w-[50%] h-[50%] bg-[#ed4b9e]/5 rounded-full blur-[150px] animate-pulse" style={{ animationDuration: '8s' }} />

          {/* Subtle noise/texture overlay if needed, currently just using the clean gradient */}
        </div>

        {/* Main application wrapper */}
        <div className="flex h-screen flex-col overflow-hidden relative z-0">
          <ErrorBoundary>
            <ClientProviders initialUser={user} initialState={initialState}>
              <div className="flex-1 relative overflow-hidden">
                <LayoutWrapper initialUser={user}>
                  {children}
                </LayoutWrapper>
              </div>
              <ToastProvider />
            </ClientProviders>
          </ErrorBoundary>
        </div>
      </body>
    </html>
  );
}