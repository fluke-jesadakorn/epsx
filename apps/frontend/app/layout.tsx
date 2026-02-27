import { GlobalErrorBoundary } from '@/components/error-boundaries/global-error-boundary';
import { SafeThemeScript } from '@/components/ui/safe-theme-script';
import '@/lib/polyfills';

import { NavigationClient } from '@/components/nav/navigation-client';
import { ClientProviders } from '@/components/providers/client-providers';
import { SharedOpenIDWeb3Provider } from '@/shared/components/auth';
import type { UserInfoResponse } from '@/shared/auth/client';
import { COOKIES } from '@/shared/auth/cookies';
import { getServerConfig } from '@/shared/config/wagmi';
import { initializeRuntimeEnvironment } from '@/shared/utils/runtime-env-validator';
import { Kanit } from 'next/font/google';
import { cookies, headers } from 'next/headers';
import { Toaster } from 'sonner';
import { cookieToInitialState } from 'wagmi';
import { ChatWidget } from '@/components/chat';
import './globals.css';

// Initialize runtime environment validation
initializeRuntimeEnvironment();

const kanit = Kanit({
  subsets: ['latin'],
  weight: ['400', '600', '700'],
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
      // Include HttpOnly access_token for client-side hydration
      // (JS can't read HttpOnly cookies, so pass it via initialUser)
      if (initialUser !== null && (initialUser.access === undefined || initialUser.access === '')) {
        const accessToken = cookieStore.get(COOKIES.access_token)?.value;
        if (accessToken !== undefined && accessToken !== '') {
          initialUser = { ...initialUser, access: accessToken };
        }
      }
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
              backendUrl={process.env['NEXT_PUBLIC_BACKEND_URL'] || process.env['BACKEND_URL'] || undefined}
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
