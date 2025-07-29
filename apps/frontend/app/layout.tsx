import { OptimizedSuspenseBoundary } from '@/components/common/OptimizedSuspenseBoundary';
import { PerformanceProvider } from '@/components/common/PerformanceProvider';
import { BackgroundDecorationsClient } from '@/components/layout/BackgroundDecorations.client';
import { Navigation } from '@/components/nav';
import { ToastProvider } from '@/components/ui/toaster';
import { AppStateProvider } from '@/context/app-state';
import { ServerAuthProvider } from '@/components/auth/ServerAuthProvider';
import { UIProvider } from '@/context/ui-context';
import { GlobalThemeProvider } from '@epsx/theme';
import { getPermissionData } from '@epsx/server-providers/server';
import { PermissionProviderWrapper } from '@/components/providers/PermissionProviderWrapper';
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

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Fetch permission data server-side
  const permissionData = await getPermissionData();

  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${kanit.variable} font-sans antialiased`}>
        <GlobalThemeProvider defaultTheme="system" enableSystem>
          <PerformanceProvider>
            <AppStateProvider>
              <UIProvider>
                <ServerAuthProvider>
                  <PermissionProviderWrapper serverData={permissionData}>
                    <ToastProvider>
                      <BackgroundDecorationsClient />

                      <OptimizedSuspenseBoundary identifier="navigation">
                        <Navigation />
                      </OptimizedSuspenseBoundary>

                      <OptimizedSuspenseBoundary identifier="main content">
                        {children}
                      </OptimizedSuspenseBoundary>
                    </ToastProvider>
                  </PermissionProviderWrapper>
                </ServerAuthProvider>
              </UIProvider>
            </AppStateProvider>
          </PerformanceProvider>
        </GlobalThemeProvider>
      </body>
    </html>
  );
}
