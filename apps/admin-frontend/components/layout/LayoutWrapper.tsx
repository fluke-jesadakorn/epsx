'use client';

import '@/lib/polyfills';

import { ReactNode } from 'react';

import { AuthLayout } from './AuthLayout';

interface LayoutWrapperProps {
  children: ReactNode;
  initialUser?: any;
}

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.initialUser
 */
export function LayoutWrapper({ children, initialUser }: LayoutWrapperProps) {
  // Pure client-side wrapper for AuthLayout
  // Web3 authentication handled entirely client-side via SharedOpenIDWeb3Provider
  // initialUser is passed from server-side for immediate auth state awareness
  return (
    <AuthLayout user={initialUser}>
      {children}
    </AuthLayout>
  );
}