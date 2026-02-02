'use client'

import { usePathname } from 'next/navigation'
import { ReactNode } from 'react'

import { MainLayout } from './MainLayout'

import { useSharedAuth } from '@/shared/components/auth/Provider'

interface AuthLayoutProps {
  children: ReactNode
  user?: {
    id: string
    email: string
    name?: string
    role: string
  }
  hasAuthCookie?: boolean
}

// Pages that should NEVER have the admin layout
const NO_LAYOUT_PATHS = [
  '/auth',
  '/login',
  '/unauthorized',
  '/access-denied',
  '/request-access',
  '/permissions/policies'
]

import { AdminAuthModal } from '@/components/auth/AdminAuthModal'

/**
 * AuthLayout
 * Simplified: Relies on Proxy for protection.
 * Primarily serves as a wrapper to inject User context into MainLayout.
 */
export function AuthLayout({ children, user: serverUser, hasAuthCookie }: AuthLayoutProps) {
  const pathname = usePathname()
  const { user: authUser } = useSharedAuth()

  // Special pages that never get layout
  const isNoLayoutPage = NO_LAYOUT_PATHS.some(path => pathname === path || pathname.startsWith(path))
  if (isNoLayoutPage) {
    return <>{children}</>
  }

  // Use client-side auth user if available, otherwise use server user
  const layoutUser = authUser ? {
    id: authUser.wallet_address || authUser.sub,
    email: authUser.email || `${authUser.wallet_address}@web3.epsx.io`,
    name: authUser.wallet_address ? `Admin (${authUser.wallet_address.slice(0, 6)}...${authUser.wallet_address.slice(-4)})` : 'Admin',
    role: 'admin'
  } : serverUser

  // Always show layout for all pages except the excluded ones
  return (
    <AdminAuthModal initialHasAuthCookie={hasAuthCookie}>
      <MainLayout user={layoutUser}>
        {children}
      </MainLayout>
    </AdminAuthModal>
  )
}