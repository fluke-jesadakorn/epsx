import { ToastProvider } from '@/components/ui/toast';
import { ThemeTransition } from '@/components/ui/theme-transition';
import { AppAdminAuthProvider } from '@/auth/ctx';
import { ModuleAuthProvider } from '@/auth/module-ctx';
import type { Metadata } from 'next';
import { GlobalThemeProvider } from '@epsx/theme';
import { Geist, Geist_Mono } from 'next/font/google';
import './globals.css';

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin'],
  display: 'swap',
});

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin'],
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'EPSX Admin',
  description: 'EPSX Admin Dashboard',
};

export const viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={`preload ${geistSans.variable} ${geistMono.variable}`}
    >
      <body className="min-h-screen font-sans antialiased bg-background text-foreground">
        <GlobalThemeProvider attribute="class" defaultTheme="system" enableSystem>
          <ThemeTransition />
          <AppAdminAuthProvider>
            <ModuleAuthProvider>
              <ToastProvider>
                {/* <Navigation /> removed as requested */}
                <div className="relative flex min-h-screen flex-col card">
                  {children}
                </div>
              </ToastProvider>
            </ModuleAuthProvider>
          </AppAdminAuthProvider>
        </GlobalThemeProvider>
      </body>
    </html>
  );
}
