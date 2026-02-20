import { GlobalErrorBoundary } from '@/components/error-boundaries/global-error-boundary';
import '@/lib/polyfills';
import Script from 'next/script';

import { NavigationClient } from '@/components/nav/navigation-client';
import { ClientProviders } from '@/components/providers/client-providers';
import { SharedOpenIDWeb3Provider } from '@/shared/components/auth';
import type { UserInfoResponse } from '@/shared/auth/client';
import { COOKIES } from '@/shared/auth/cookies';
import { getServerConfig } from '@/shared/config/wagmi';
import { initializeRuntimeEnvironment } from '@/shared/utils/runtime-env-validator';
import { Kanit } from 'next/font/google';
import { cookies, headers } from 'next/headers';
import { ChatWidget } from '@/components/chat';
import { Toaster } from 'sonner';
import { cookieToInitialState } from 'wagmi';
import './globals.css';

import { SafeThemeScript } from '@/components/ui/safe-theme-script';
// Verify polyfills are active for debugging (both for server and client module load)
if (typeof window !== 'undefined') {
  const mathPowAny = Math.pow as unknown as Record<string, unknown>;
  if (mathPowAny.__isPolyfilled === true) {
    // Polyfills verified active
  }
}

// Initialize runtime environment validation
initializeRuntimeEnvironment();

// Pure Web3 layout - no server-side session checking required
export const dynamic = 'force-dynamic';

const kanit = Kanit({
  subsets: ['latin'],
  weight: ['200', '300', '400', '500', '600', '700', '800'],
  display: 'swap',
  variable: '--font-kanit',
});

export const metadata = {
  title: 'EPSX - Stock Analytics Platform',
  description: 'Advanced stock data analytics platform',
  keywords: ['stock analytics', 'financial data', 'EPSX', 'market insights'],
  authors: [{ name: 'EPSX Team' }],
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  formatDetection: {
    telephone: false,
    date: false,
    address: false,
    email: false,
  },
};

export const viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 5,
  userScalable: true,
  viewportFit: 'cover',
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: '#ffffff' },
    { media: '(prefers-color-scheme: dark)', color: '#000000' },
  ],
};

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Hydrate user from server-side cookie for auth persistence across page refreshes
  let initialUser: UserInfoResponse | null = null;
  try {
    const cookieStore = await cookies();
    const userCookie = cookieStore.get(COOKIES.user)?.value;
    if (userCookie !== undefined && userCookie !== '') {
      initialUser = JSON.parse(decodeURIComponent(userCookie)) as UserInfoResponse;
    }
  } catch {
    // Invalid cookie - start fresh
  }

  // Unified Web3 Cookie Hydration
  const headersList = await headers();
  const cookie = headersList.get('cookie');

  // Safely parse wagmi cookie state - handle URL-encoded or malformed cookies
  let initialState;
  try {
    initialState = cookieToInitialState(getServerConfig(), cookie);
  } catch (_error) {
    // Cookie value might be URL-encoded (e.g., '%7B%22stat...' instead of raw JSON)
    // Try decoding the cookie string first
    try {
      const decodedCookie = cookie !== null && cookie !== undefined ? decodeURIComponent(cookie) : null;
      initialState = cookieToInitialState(getServerConfig(), decodedCookie);
    } catch {
      // If all parsing fails, use undefined (fresh state)
      initialState = undefined;
    }
  }

  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        {/* CRITICAL: BigInt-safe Math polyfill must run before ANY other JS */}
        <Script id="bigint-polyfill" strategy="beforeInteractive">
          {`
            (function() {
              if (!Math.pow || !Math.pow.__isPolyfilled) {
                var originalPow = Math.pow;
                Math.pow = function(b, e) {
                  if (typeof b === 'bigint' || typeof e === 'bigint') {
                    try {
                      return new Function('b', 'e', 'return b ** e')(b, e);
                    } catch (err) {
                      if (typeof b === 'bigint' && typeof e === 'bigint') {
                         var res = 1n; 
                         var exp = BigInt(e);
                         if (exp < 0n) return 0n;
                         for (var i = 0n; i < exp; i++) res *= b;
                         return res;
                      }
                      return NaN;
                    }
                  }
                  return originalPow.apply(Math, arguments);
                };
                Math.pow.__isPolyfilled = true;
                
                ['floor', 'ceil', 'round', 'trunc', 'abs'].forEach(function(f) {
                  var orig = Math[f];
                  if (orig) {
                    Math[f] = function(v) {
                      if (typeof v === 'bigint') return v;
                      return orig.call(Math, v);
                    };
                  }
                });
                console.log('[Polyfill] Inline BigInt-safe Math polyfill active');
              }
            })();
          `}
        </Script>
        {/* Mobile performance optimizations */}
        <meta name="msapplication-tap-highlight" content="no" />

        {/* Preconnect to external resources */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin="anonymous"
        />

        {/* Critical resource hints */}
        <link rel="dns-prefetch" href="//fonts.googleapis.com" />
        <link rel="dns-prefetch" href="//fonts.gstatic.com" />

        {/* Performance and mobile optimizations */}
        <meta httpEquiv="x-ua-compatible" content="ie=edge" />
        <meta name="HandheldFriendly" content="true" />
        <meta name="MobileOptimized" content="width" />
      </head>
      <body
        className={`${kanit.variable} bg-background text-foreground overflow-x-hidden font-sans antialiased`}
        suppressHydrationWarning
      >
        <SafeThemeScript />
        <GlobalErrorBoundary level="global">
          <ClientProviders initialState={initialState}>
            <SharedOpenIDWeb3Provider
              clientId="epsx-frontend"
              backendUrl={process.env.NEXT_PUBLIC_BACKEND_URL}
              initialUser={initialUser}
            >
              {/* Mobile navigation optimized for touch */}
              <NavigationClient />

              {/* Main content with mobile scroll optimization */}
              <main className="relative min-h-screen">{children}</main>

              {/* Floating chat widget */}
              <ChatWidget />

              {/* Toast notifications */}
              <Toaster
                position="top-right"
                toastOptions={{
                  style: {
                    background: 'hsl(var(--background))',
                    color: 'hsl(var(--foreground))',
                    border: '1px solid hsl(var(--border))',
                  },
                }}
              />

            </SharedOpenIDWeb3Provider>
          </ClientProviders>
        </GlobalErrorBoundary>
      </body>
    </html>
  );
}
