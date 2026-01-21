'use client';

import { ReactNode } from 'react';

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
    <div className="flex h-screen w-full overflow-hidden bg-background">
      {/* Sidebar Navigation - Fixed height handled by parent h-screen */}
      <Sidebar />

      {/* Right Side - Content Area */}
      <div className="flex flex-1 flex-col h-full overflow-hidden">
        {/* Header - Sticky within the content area if needed, or just top block */}
        <Header user={user} />

        {/* Content Wrapper - Fixed frame, internal main scrolls */}
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {/* Breadcrumb - Fixed */}
          <div className="border-b border-border bg-card px-3 sm:px-4 lg:px-6 py-2 sm:py-3 z-30">
            <Breadcrumb />
          </div>

          {/* Main Content - Scrollable */}
          <main className="flex-1 overflow-y-auto overflow-x-hidden p-0">
            {children}
          </main>

          {/* Footer - Fixed */}
          <footer className="border-t border-border bg-card px-3 sm:px-4 lg:px-6 py-3 sm:py-4">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 sm:gap-4">
              <div className="flex items-center gap-1.5 sm:gap-2">
                <span className="text-sm sm:text-base">⚡</span>
                <span className="text-xs sm:text-sm font-semibold text-gray-900 dark:text-white">
                  EPSX Admin Dashboard
                </span>
              </div>
              <div className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">
                Version 2.0
              </div>
            </div>
          </footer>
        </div>
      </div>
    </div>
  );
}
