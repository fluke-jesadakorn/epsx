'use client';

import type { ReactNode} from 'react';
import { Suspense } from 'react';

import { Header } from './header';
import { Sidebar } from './sidebar';
import type { Notification as ApiNotification } from '@/shared/api/notifications';

interface User {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface MainLayoutProps {
  children: ReactNode;
  user?: User;
  initialNotifications?: ApiNotification[];
  initialUnreadCount?: number;
}

/**
 * Main Layout with Sidebar + Header.
 * Using "App Shell" fixed layout:
 * - Outer container: h-screen, overflow-hidden (prevents body scroll)
 * - Sidebar: h-full (fits to screen)
 * - Main Content: overflow-y-auto (scrolls independently)
 *
 * @param root0 - Component props.
 * @param root0.children - Page content.
 * @param root0.user - Optional authenticated user.
 * @param root0.initialNotifications - Pre-fetched notifications.
 * @param root0.initialUnreadCount - Pre-fetched unread count.
 */
export function MainLayout({ children, user, initialNotifications, initialUnreadCount }: MainLayoutProps) {
  return (
    <div className="flex h-screen w-full overflow-hidden bg-background">
      {/* Sidebar Navigation - Fixed height handled by parent h-screen */}
      <Sidebar />

      {/* Right Side - Content Area */}
      <div className="flex flex-1 flex-col h-full overflow-hidden">
        {/* Header - Sticky within the content area if needed, or just top block */}
        <Header user={user} initialNotifications={initialNotifications} initialUnreadCount={initialUnreadCount} />

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
