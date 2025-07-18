import { AuthProvider } from '@/lib/auth';
import { ThemeProvider } from 'next-themes';
import { ToastProvider } from '@/components/ui/toaster';
import { Navigation } from '@/components/nav';
import { AuthDebugger } from '@/components/debug/AuthDebugger';
import './globals.css';

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
      <body>
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
          <AuthProvider>
            <ToastProvider>
              <Navigation />
              {children}
              <AuthDebugger />
            </ToastProvider>
          </AuthProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
