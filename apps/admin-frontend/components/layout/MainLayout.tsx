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
    <div className="flex h-screen w-full overflow-hidden bg-gray-50 dark:bg-gray-900">
      {/* Sidebar Navigation - Fixed height handled by parent h-screen */}
      <Sidebar />

      {/* Right Side - Content Area */}
      <div className="flex flex-1 flex-col h-full overflow-hidden">
        {/* Header - Sticky within the content area if needed, or just top block */}
        <Header user={user} />

        {/* Scrollable Content Wrapper */}
        <div className="flex-1 overflow-y-auto overflow-x-hidden">
          {/* Breadcrumb */}
          <div className="border-b border-gray-200 bg-white px-6 py-3 dark:border-gray-700 dark:bg-gray-800 sticky top-0 z-30">
            <Breadcrumb />
          </div>

          {/* Main Content */}
          <main className="p-0">{children}</main>

          {/* Footer */}
          <footer className="border-t border-gray-200 bg-white px-6 py-4 dark:border-gray-700 dark:bg-gray-800">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span>⚡</span>
                <span className="text-sm font-semibold text-gray-900 dark:text-white">
                  EPSX Admin Dashboard
                </span>
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">
                Version 2.0
              </div>
            </div>
          </footer>
        </div>
      </div>
    </div>
  );
}
