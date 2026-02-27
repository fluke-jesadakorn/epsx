'use client';

import '@/lib/polyfills';

import type { ReactNode } from 'react';

import { AuthLayout } from './auth-layout';

interface InitialUser {
  id: string;
  email: string;
  name?: string;
  role: string;
}

interface LayoutWrapperProps {
  children: ReactNode;
  initialUser?: InitialUser;
}

export function LayoutWrapper({ children, initialUser }: LayoutWrapperProps) {
  return (
    <AuthLayout user={initialUser}>
      {children}
    </AuthLayout>
  );
}