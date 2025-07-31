import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { BackgroundDecorationsClient } from '@/components/layout/BackgroundDecorations.client';
import { Navigation } from '@/components/nav';
import { ToastProvider } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { AuthProvider } from '@/context/auth-context';
import { PermissionProvider } from '@epsx/server-providers/client';
import { UIProvider } from '@/context/ui-context';
import { GlobalThemeProvider } from '@epsx/theme';
import { Kanit } from 'next/font/google';
import './globals.css';

const kanit = Kanit({
  subsets: ['latin'],
  weight: ['200', '300', '400', '500', '600', '700', '800'],
  display: 'swap',
  variable: '--font-kanit',
});

export const metadata = {
  title: 'EPSX - Stock Trading Platform',
  description: 'Advanced stock trading and analytics platform',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {

  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${kanit.variable} font-sans antialiased`}>
        <GlobalThemeProvider defaultTheme="system" enableSystem>
          <PerformanceProvider>
            <AppStateProvider>
              <UIProvider>
                <AuthProvider>
                  <PermissionProvider serverData={{
                    paymentStatus: null,
                    permissions: [],
                    featureAccess: {},
                    rankingAccess: {},
                    error: null
                  }}>
                    <ToastProvider>
                      <BackgroundDecorationsClient />

                      <OptimizedSuspenseBoundary identifier="navigation">
                        <Navigation />
                      </OptimizedSuspenseBoundary>

                      <OptimizedSuspenseBoundary identifier="main content">
                        {children}
                      </OptimizedSuspenseBoundary>
                    </ToastProvider>
                  </PermissionProvider>
                </AuthProvider>
              </UIProvider>
            </AppStateProvider>
          </PerformanceProvider>
        </GlobalThemeProvider>
      </body>
    </html>
  );
}
