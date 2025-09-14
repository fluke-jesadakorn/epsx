'use client';

import { ReactNode, useState, useEffect } from 'react';
import { DynamicBreadcrumb } from './DynamicBreadcrumb';
import { FloatingActionButtons } from './FloatingActionButtons';
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
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    const checkDarkMode = () => {
      setIsDark(document.documentElement.classList.contains('dark'));
    };
    
    checkDarkMode();
    
    const observer = new MutationObserver(checkDarkMode);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class']
    });
    
    return () => observer.disconnect();
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-slate-800 dark:via-slate-700 dark:to-slate-800">
      {/* Background Decorations */}
      <div className="pointer-events-none fixed inset-0 overflow-hidden">
        {/* Floating Circles */}
        <div className="absolute top-20 left-20 h-32 w-32 animate-pulse rounded-full bg-gradient-to-r from-yellow-400/20 to-orange-500/20 blur-xl"></div>
        <div className="absolute top-40 right-32 h-24 w-24 animate-pulse rounded-full bg-gradient-to-r from-pink-400/20 to-purple-500/20 blur-xl delay-1000"></div>
        <div className="absolute bottom-32 left-40 h-40 w-40 animate-pulse rounded-full bg-gradient-to-r from-blue-400/20 to-teal-500/20 blur-xl delay-2000"></div>
        <div className="absolute right-20 bottom-20 h-28 w-28 animate-pulse rounded-full bg-gradient-to-r from-green-400/20 to-emerald-500/20 blur-xl delay-500"></div>

        {/* Grid Pattern */}
        <div className="bg-grid-pattern absolute inset-0 opacity-5"></div>

        {/* Gradient Overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-transparent via-transparent to-white/5 dark:to-black/10"></div>
      </div>

      <div className="relative z-10 flex">
        {/* Sidebar Navigation */}
        <PancakeAdminNav />

        {/* Main Content Area */}
        <div className="flex min-h-screen flex-1 flex-col">
          {/* Header */}
          <PancakeAdminHeader user={user} />

          {/* Main Content */}
          <main className="relative flex-1">
            {/* Content Container */}
            <div className="relative">
              {/* Breadcrumb Bar */}
              <div 
                className="border-b border-yellow-200/30 bg-white/50 px-6 py-3 backdrop-blur-sm dark:border-slate-700/30 dark:bg-slate-900/90"
                style={{
                  background: isDark 
                    ? 'rgba(15, 23, 42, 0.9)' 
                    : undefined
                }}
              >
                <DynamicBreadcrumb />
              </div>

              {/* Page Content */}
              <div className="relative">
                {/* Content Wrapper with Pancake Style */}
                <div className="relative min-h-[calc(100vh-8rem)]">
                  {/* Glass Background */}
                  <div className="absolute inset-0 bg-gradient-to-r from-white/40 via-white/20 to-transparent backdrop-blur-sm dark:from-slate-800/40 dark:via-slate-700/20 dark:to-transparent"></div>

                  {/* Actual Content */}
                  <div className="relative z-10">{children}</div>
                </div>
              </div>
            </div>
          </main>

          {/* Footer */}
          <footer
            className="border-t border-yellow-200/50 bg-gradient-to-r from-white/80 via-yellow-50/80 to-orange-50/80 px-4 py-4 backdrop-blur-sm sm:px-6 dark:border-slate-600/20 dark:from-slate-800/80 dark:via-slate-700/50 dark:to-slate-800/80"
            suppressHydrationWarning
          >
            <div className="flex flex-col items-center justify-between gap-2 sm:flex-row">
              <div className="flex items-center gap-3 sm:gap-4">
                <span className="text-xl sm:text-2xl">⚡</span>
                <div className="text-center sm:text-left">
                  <div className="max-w-[200px] truncate bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-xs font-bold text-transparent sm:max-w-none sm:text-sm">
                    EPSX Admin Dashboard
                  </div>
                  <div className="max-w-[250px] truncate text-xs text-gray-600 sm:max-w-none dark:text-gray-400">
                    EPSX - Advanced analytics platform
                  </div>
                </div>
              </div>

              <div className="flex flex-wrap items-center justify-center gap-2 text-xs text-gray-500 sm:justify-end sm:gap-4 dark:text-gray-400">
                <span className="whitespace-nowrap">🚀 Version 2.0</span>
                <span className="hidden sm:inline">•</span>
                <span className="whitespace-nowrap">⚡ Fast & Sweet</span>
              </div>
            </div>
          </footer>
        </div>
      </div>

      {/* Floating Action Buttons */}
      <FloatingActionButtons />
    </div>
  );
}

// CSS for grid pattern (add to globals.css)
const gridPatternCSS = `
.bg-grid-pattern {
  background-image: 
    linear-gradient(rgba(255, 193, 7, 0.1) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 193, 7, 0.1) 1px, transparent 1px);
  background-size: 50px 50px;
}
`;
