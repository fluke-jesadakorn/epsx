'use client';

import '@/lib/polyfills';

import type { ReactNode } from 'react';

import { AuthLayout } from './auth-layout';

interface LayoutWrapperProps {
  children: ReactNode;
  initialUser?: any;
}

export function LayoutWrapper({ children, initialUser }: LayoutWrapperProps) {
  return (
    <AuthLayout user={initialUser}>
      {children}
    </AuthLayout>
  );
}