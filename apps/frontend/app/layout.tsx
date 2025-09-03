import { NavigationClient } from '@/components/nav/NavigationClient';
import { getAuthUser } from '@/lib/server/auth';
import { Kanit } from 'next/font/google';
import { type EPSXJWTPayload } from '@/lib/auth-utils';
// Notifications re-enabled with working API client
import { NotificationProvider } from '@/context/notification-context';
import { Toaster } from 'sonner';
import { ServiceWorkerInitializer } from '@/components/ServiceWorkerInitializer';
import { ClientProviders } from '@/components/providers/ClientProviders';
import './globals.css';

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
  
  try {
    jwtPayload = await getAuthUser();
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
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        
        {/* Critical resource hints */}
        <link rel="dns-prefetch" href="//fonts.googleapis.com" />
        <link rel="dns-prefetch" href="//fonts.gstatic.com" />
        
        
        {/* Performance and mobile optimizations */}
        <meta httpEquiv="x-ua-compatible" content="ie=edge" />
        <meta name="HandheldFriendly" content="true" />
        <meta name="MobileOptimized" content="width" />
      </head>
      <body className={`${kanit.variable} font-sans antialiased bg-background text-foreground overflow-x-hidden`}>
        <ClientProviders>
          <NotificationProvider>
            {/* Service Worker Registration */}
            <ServiceWorkerInitializer />
            
            {/* Mobile navigation optimized for touch */}
            <NavigationClient user={user} />
            
            {/* Main content with mobile scroll optimization */}
            <main className="relative min-h-screen">
              {children}
            </main>
            
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
          </NotificationProvider>
        </ClientProviders>
      </body>
    </html>
  );
}
