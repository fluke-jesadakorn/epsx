import { Kanit } from 'next/font/google';
import { OptimizedLayout } from '@/components/server/OptimizedLayout';
import { ServerNavigation } from '@/components/server/ServerNavigation';
import { OptimizedToaster } from '@/components/ui/optimized-toaster';
import { OptimizedSuspense } from '@/components/common/OptimizedSuspense';
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

interface OptimizedRootLayoutProps {
  children: React.ReactNode;
}

/**
 * Optimized root layout for minimal hydration and better performance
 */
export default function OptimizedRootLayout({ children }: OptimizedRootLayoutProps) {
  return (
    <OptimizedLayout>
      <div className={`${kanit.variable} font-sans antialiased min-h-screen`}>
        {/* Server-rendered navigation */}
        <OptimizedSuspense name="navigation">
          <ServerNavigation />
        </OptimizedSuspense>
        
        {/* Main content with minimal client hydration */}
        <main className="relative z-10">
          <OptimizedSuspense name="main content">
            {children}
          </OptimizedSuspense>
        </main>
        
        {/* Client-side toast system */}
        <OptimizedToaster />
      </div>
    </OptimizedLayout>
  );
}