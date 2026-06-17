'use client';

import '@/lib/polyfills';

import type { ReactNode } from 'react';

import { AuthLayout } from './auth-layout';
import type { Notification as ApiNotification } from '@/shared/api/notifications';

interface InitialUser {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface LayoutWrapperProps {
  children: ReactNode;
  initialUser?: InitialUser;
  initialNotifications?: ApiNotification[];
  initialUnreadCount?: number;
}

export function LayoutWrapper({ children, initialUser, initialNotifications, initialUnreadCount }: LayoutWrapperProps) {
  return (
    <AuthLayout user={initialUser} initialNotifications={initialNotifications} initialUnreadCount={initialUnreadCount}>
      {children}
    </AuthLayout>
  );
}