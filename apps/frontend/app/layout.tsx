import { AuthDebugger } from '@/components/debug/AuthDebugger';
import { Navigation } from '@/components/nav';
import { ToastProvider } from '@/components/ui/toaster';
import { AuthProvider } from '@/context/auth-context';
import { AppStateProvider } from '@/context/app-state';
import { UIProvider } from '@/context/ui-context';
import { ThemeProvider } from '@epsx/ui';
import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { Kanit } from 'next/font/google';
import { Suspense } from 'react';
import { BackgroundDecorationsClient } from '@/components/layout/BackgroundDecorations.client';
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
        <ThemeProvider defaultTheme="pancake" enableSystem>
          <PerformanceProvider>
            <AppStateProvider>
              <UIProvider>
                <AuthProvider>
                  <ToastProvider>
                <BackgroundDecorationsClient />

                <OptimizedSuspenseBoundary identifier="navigation">
                  <Navigation />
                </OptimizedSuspenseBoundary>
                
                <OptimizedSuspenseBoundary identifier="main content">
                  {children}
                </OptimizedSuspenseBoundary>
                
                <Suspense>
                  <AuthDebugger />
                </Suspense>
                  </ToastProvider>
                </AuthProvider>
              </UIProvider>
            </AppStateProvider>
          </PerformanceProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
