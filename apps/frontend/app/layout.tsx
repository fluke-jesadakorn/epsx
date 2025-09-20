import { NavigationClient } from '@/components/nav/NavigationClient';
import { ClientProviders } from '@/components/providers/ClientProviders';
import { Web3Provider } from '@/providers/Web3Provider';
import { Web3AuthProvider } from '@/providers/Web3AuthProvider';
import { type EPSXJWTPayload } from '../../../shared/auth/jwt';
import { getAuthUser } from '@/lib/server/auth';
import { getUserNotifications, type NotificationData } from '@/lib/actions/notifications';
import { initializeRuntimeEnvironment } from '../../../shared/utils/runtime-env-validator';
import { Kanit } from 'next/font/google';
import { Toaster } from 'sonner';
import './globals.css';

// Initialize runtime environment validation
initializeRuntimeEnvironment();

export const dynamic = 'force-dynamic';

// Convert EPSXJWTPayload to AuthUser format
function mapToAuthUser(payload: EPSXJWTPayload | null) {
  if (!payload) return null;

  return {
    user_id: payload.sub,
    email: payload.email,
    role: String(payload.role || 'user'),
    permissions: payload.permissions || [],
    package_tier: String(payload.package_tier || 'basic'),
  };
}

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

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  let jwtPayload: EPSXJWTPayload | null = null;
  let notificationData: NotificationData | null = null;

  try {
    jwtPayload = await getAuthUser();
    
    // Fetch notifications if user is authenticated
    if (jwtPayload) {
      notificationData = await getUserNotifications();
    }
  } catch (error) {
    // Gracefully handle auth failures for static generation
    console.warn('Failed to get auth user in layout:', error);
  }

  const user = mapToAuthUser(jwtPayload);

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
        <ClientProviders>
          <Web3Provider>
            <Web3AuthProvider>
              {/* Mobile navigation optimized for touch */}
              <NavigationClient user={user} initialNotificationData={notificationData} />

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
            </Web3AuthProvider>
          </Web3Provider>
        </ClientProviders>
      </body>
    </html>
  );
}
