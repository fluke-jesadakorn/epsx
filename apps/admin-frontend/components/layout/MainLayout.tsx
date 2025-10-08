'use client';

import { usePathname } from 'next/navigation';
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

const NO_SIDEBAR_PATHS = ['/permissions/policies'];

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.user
 */
export function MainLayout({
  children,
  user,
}: MainLayoutProps) {
  const pathname = usePathname();
  const hideLayout = NO_SIDEBAR_PATHS.some(path => pathname === path || pathname.startsWith(path));

  if (hideLayout) {
    return <>{children}</>;
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex">
      {/* Sidebar Navigation */}
      <Sidebar />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <Header user={user} />

        {/* Breadcrumb */}
        <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-3">
          <Breadcrumb />
        </div>

        {/* Main Content */}
        <main className="flex-1 p-6">
          {children}
        </main>

        {/* Footer */}
        <footer className="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-6 py-4">
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
  );
}

