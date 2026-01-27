'use client';

import { ReactNode, Suspense } from 'react';

import { Breadcrumb } from './Breadcrumb';
import { Header } from './Header';
import { Sidebar } from './Sidebar';

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
    <div className="flex h-screen w-full overflow-hidden bg-gradient-to-br from-slate-900 via-purple-900/20 to-slate-900 bg-grid-pattern">
      {/* Sidebar Navigation - Fixed height handled by parent h-screen */}
      <Sidebar />

      {/* Right Side - Content Area */}
      <div className="flex flex-1 flex-col h-full overflow-hidden">
        {/* Header - Sticky within the content area if needed, or just top block */}
        <Header user={user} />

        {/* Content Wrapper - Fixed frame, internal main scrolls */}
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {/* Breadcrumb - Fixed with glass effect */}
          <div className="border-b border-white/10 bg-white/5 backdrop-blur-xl px-3 sm:px-4 lg:px-6 py-2 sm:py-3 z-30">
            <Breadcrumb />
          </div>

          {/* Main Content - Scrollable */}
          <main className="flex-1 overflow-y-auto overflow-x-hidden p-0">
            <Suspense fallback={null}>
              {children}
            </Suspense>
          </main>

          {/* Footer - Fixed with glass effect */}
          <footer className="border-t border-white/10 bg-white/5 backdrop-blur-xl px-4 py-3">
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
