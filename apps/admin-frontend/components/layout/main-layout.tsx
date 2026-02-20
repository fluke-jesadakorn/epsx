'use client';

import type { ReactNode} from 'react';
import { Suspense } from 'react';

import { Breadcrumb } from './breadcrumb';
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
    <div className="flex h-screen w-full overflow-hidden bg-gray-100 dark:bg-[#0f1117] bg-grid-pattern">
      {/* Sidebar Navigation - Fixed height handled by parent h-screen */}
      <Sidebar />

      {/* Right Side - Content Area */}
      <div className="flex flex-1 flex-col h-full overflow-hidden">
        {/* Header - Sticky within the content area if needed, or just top block */}
        <Header user={user} />

        {/* Content Wrapper - Fixed frame, internal main scrolls */}
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {/* Breadcrumb - Fixed with glass effect */}
          <div className="border-b border-gray-200 dark:border-slate-700 bg-white/80 dark:bg-[#13151e]/80 backdrop-blur-xl px-3 sm:px-4 lg:px-6 py-2 sm:py-3 z-30">
            <Breadcrumb />
          </div>

          {/* Main Content - Scrollable */}
          <main className="flex-1 overflow-y-auto overflow-x-hidden p-0">
            <Suspense fallback={null}>
              {children}
            </Suspense>
          </main>

          {/* Footer - Fixed with glass effect */}
          <footer className="border-t border-gray-200 dark:border-slate-700 bg-white/80 dark:bg-[#13151e]/80 backdrop-blur-xl px-4 py-3">
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
