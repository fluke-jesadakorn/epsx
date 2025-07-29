import { getSessionInfo } from '@/lib/auth-server';
import { OptimizedThemeProvider } from './OptimizedThemeProvider';
import { OptimizedAuthProvider } from './OptimizedAuthProvider';

interface OptimizedLayoutProps {
  children: React.ReactNode;
}

/**
 * Server-optimized layout that minimizes client hydration
 */
export async function OptimizedLayout({ children }: OptimizedLayoutProps) {
  // Get server-side session for initial state
  const sessionInfo = await getSessionInfo();
  
  return (
    <OptimizedThemeProvider>
      <OptimizedAuthProvider initialSession={sessionInfo}>
        {/* Minimize decorative elements that cause hydration issues */}
        <div className="min-h-screen bg-background">
          {children}
        </div>
      </OptimizedAuthProvider>
    </OptimizedThemeProvider>
  );
}