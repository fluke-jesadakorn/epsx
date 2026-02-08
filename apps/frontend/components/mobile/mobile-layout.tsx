'use client';

import type { ReactNode } from 'react';
import { MobileHeader } from './Mobileheader';
import { MobileBottomNav } from './mobile-bottom-nav';

interface MobileLayoutProps {
  children: ReactNode;
  title?: string;
  user?: {
    name?: string;
    email?: string;
    avatar?: string;
  };
  showSearch?: boolean;
  showNotifications?: boolean;
  showBottomNav?: boolean;
  notificationCount?: number;
  onLogout?: () => void;
}

export function MobileLayout({ 
  children,
  title,
  user,
  showSearch = true,
  showNotifications = true,
  showBottomNav = true,
  notificationCount = 0,
  onLogout
}: MobileLayoutProps) {
  return (
    <div className="md:hidden min-h-screen bg-background">
      <MobileHeader 
        title={title}
        user={user}
        showSearch={showSearch}
        showNotifications={showNotifications}
        notificationCount={notificationCount}
        onLogout={onLogout}
      />
      
      <main className={`${showBottomNav ? 'pb-16' : ''}`}>
        {children}
      </main>
      
      {showBottomNav && <MobileBottomNav />}
    </div>
  );
}