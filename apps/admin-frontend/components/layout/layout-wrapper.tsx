'use client';

import '@/lib/polyfills';

import type { ReactNode } from 'react';

import { AuthLayout } from './auth-layout';

interface LayoutWrapperProps {
  children: ReactNode;
  initialUser?: any;
  hasAuthCookie?: boolean;
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.initialUser
 * @param root0.hasAuthCookie
 */
export function LayoutWrapper({ children, initialUser, hasAuthCookie }: LayoutWrapperProps) {
  // Pure client-side wrapper for AuthLayout
  // Web3 authentication handled entirely client-side via SharedOpenIDWeb3Provider
  // initialUser is passed from server-side for immediate auth state awareness
  return (
    <AuthLayout user={initialUser} hasAuthCookie={hasAuthCookie}>
      {children}
    </AuthLayout>
  );
}