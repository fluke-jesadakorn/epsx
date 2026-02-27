'use client';

import type { ReactNode} from 'react';
import { Suspense } from 'react';

import { Header } from './header';
import { Sidebar } from './sidebar';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface MainLayoutProps {
  children: ReactNode;
  user?: User;
}

/**
 * Main Layout with Sidebar + Header
 * Using "App Shell" fixed layout:
 * - Outer container: h-screen, overflow-hidden (prevents body scroll)
 * - Sidebar: h-full (fits to screen)
 * - Main Content: overflow-y-auto (scrolls independently)
 *
 * @param root0
 * @param root0.children
 * @param root0.user
 */
export function MainLayout({ children, user }: MainLayoutProps) {
  return (
    <div className="flex h-screen w-full overflow-hidden bg-background">
      {/* Sidebar Navigation - Fixed height handled by parent h-screen */}
      <Sidebar />

      {/* Right Side - Content Area */}
      <div className="flex flex-1 flex-col h-full overflow-hidden">
        {/* Header - Sticky within the content area if needed, or just top block */}
        <Header user={user} />

        {/* Content Wrapper - Fixed frame, internal main scrolls */}
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {/* Main Content - Scrollable */}
          <main className="flex-1 overflow-y-auto overflow-x-hidden p-0">
            <Suspense fallback={null}>
              {children}
            </Suspense>
          </main>

          {/* Footer - Fixed with glass effect */}
          <footer className="border-t border-border/40 bg-card px-4 py-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-foreground">
                EPSX Admin Dashboard
              </span>
              <span className="text-sm text-muted-foreground">
                Version 2.0
              </span>
            </div>
          </footer>
        </div>
      </div>
    </div>
  );
}
