import '@/lib/polyfills';
// Import browser polyfills first to handle SSR issues
import { GlobalErrorBoundary } from '@/components/error-boundaries/GlobalErrorBoundary';

import { NavigationClient } from '@/components/nav/NavigationClient';
import { ClientProviders } from '@/components/providers/ClientProviders';
import { SharedOpenIDWeb3Provider } from '@/shared/components/auth/Provider';
import { getServerConfig } from '@/shared/config/wagmi';
import { initializeRuntimeEnvironment } from '@/shared/utils/runtime-env-validator';
import { Kanit } from 'next/font/google';
import { headers } from 'next/headers';
import { Toaster } from 'sonner';
import { cookieToInitialState } from 'wagmi';
import './globals.css';

import { SafeThemeScript } from '@/components/ui/SafeThemeScript';

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
  // Unified Web3 Cookie Hydration
  const headersList = await headers();
  const cookie = headersList.get('cookie');
  const initialState = cookieToInitialState(getServerConfig(), cookie);

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
              backendUrl={process.env.NEXT_PUBLIC_BACKEND_URL}
            >
              {/* Mobile navigation optimized for touch */}
              <NavigationClient />

              {/* Main content with mobile scroll optimization */}
              <main className="relative min-h-screen">{children}</main>


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
