'use client';

import { ReactNode } from 'react';
import { DynamicBreadcrumb } from './DynamicBreadcrumb';
import { PancakeAdminHeader } from './PancakeAdminHeader';
import { PancakeAdminNav } from './PancakeAdminNav';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface PancakeAdminLayoutProps {
  children: ReactNode;
  user?: User;
}

export function PancakeAdminLayout({
  children,
  user,
}: PancakeAdminLayoutProps) {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex">
      {/* Sidebar Navigation */}
      <PancakeAdminNav />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <PancakeAdminHeader user={user} />

        {/* Breadcrumb */}
        <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-3">
          <DynamicBreadcrumb />
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

