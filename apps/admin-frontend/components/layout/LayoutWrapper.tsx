'use client';

import '@/lib/polyfills';

import { ReactNode } from 'react';

import { AuthLayout } from './AuthLayout';

interface LayoutWrapperProps {
  children: ReactNode;
}

/**
 *
 * @param root0
 * @param root0.children
 */
export function LayoutWrapper({ children }: LayoutWrapperProps) {
  // Pure client-side wrapper for AuthLayout
  // Web3 authentication handled entirely client-side via SharedOpenIDWeb3Provider
  return (
    <AuthLayout user={undefined}>
      {children}
    </AuthLayout>
  );
}