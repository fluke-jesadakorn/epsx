// Import browser polyfills first to handle SSR issues
import { NavigationClient } from '@/components/nav/NavigationClient';
import { ClientProviders } from '@/components/providers/ClientProviders';
import { MinimalWeb3Provider } from '@/components/providers/MinimalWeb3Provider';
import { GlobalErrorBoundary } from '@/components/error-boundaries/GlobalErrorBoundary';
import '@/lib/browser-polyfills';
import { PureWeb3AuthProvider } from '@/providers/PureWeb3AuthProvider';
import { Kanit } from 'next/font/google';
import { Toaster } from 'sonner';
import { initializeRuntimeEnvironment } from '../../../shared/utils/runtime-env-validator';
import './globals.css';

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
  title: 'EPSX - Stock Trading Platform',
  description: 'Advanced stock trading and analytics platform',
  keywords: ['stock trading', 'analytics', 'EPSX', 'financial data'],
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

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Pure Web3 - authentication handled entirely client-side

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
      >
        <GlobalErrorBoundary level="global">
          <ClientProviders>
            <MinimalWeb3Provider>
              <PureWeb3AuthProvider>
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
              </PureWeb3AuthProvider>
            </MinimalWeb3Provider>
          </ClientProviders>
        </GlobalErrorBoundary>
      </body>
    </html>
  );
}
