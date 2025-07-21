import { Navigation } from '@/components/layout/nav';
import { ToastProvider } from '@/components/ui/toast';
import { AdminAuthProvider } from '@/context/admin-auth';
import type { Metadata } from 'next';
import { ThemeProvider } from 'next-themes';
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
  title: 'ESPx Admin',
  description: 'ESPx - Admin Dashboard',
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
      className={`${geistSans.variable} ${geistMono.variable}`}
    >
      <body
        className="min-h-screen font-sans antialiased"
        style={{
          background: 'linear-gradient(135deg, #f3b9ff 0%, #53dee9 100%)',
          color: 'var(--color-text)',
        }}
      >
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
          <AdminAuthProvider>
            <ToastProvider>
              <Navigation />
              <div className="relative flex min-h-screen flex-col card">
                {children}
              </div>
            </ToastProvider>
          </AdminAuthProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
