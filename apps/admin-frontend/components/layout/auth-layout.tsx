'use client'

import { usePathname } from 'next/navigation'
import type { ReactNode } from 'react'

import { MainLayout } from './main-layout'

import { useSharedAuth } from '@/shared/components/auth'

interface AuthLayoutProps {
  children: ReactNode
  user?: {
    id: string
    email: string
    name?: string
    role: string
  }
}

// Pages that should NEVER have the admin layout
const NO_LAYOUT_PATHS = [
  '/auth',
  '/login',
  '/unauthorized',
  '/access-denied',
  '/request-access',
  '/permissions/policies',
  '/manual'
]

/**
 * AuthLayout
 * Middleware handles auth protection server-side.
 * This wraps authenticated pages with MainLayout + user context.
 */
export function AuthLayout({ children, user: serverUser }: AuthLayoutProps) {
  const pathname = usePathname()
  const { user: authUser } = useSharedAuth()

  // Special pages that never get layout
  const isNoLayoutPage = NO_LAYOUT_PATHS.some(path => pathname === path ?? pathname.startsWith(path))
  if (isNoLayoutPage) {
    return <>{children}</>
  }

  // Use client-side auth user if available, otherwise use server user
  const layoutUser = authUser ? {
    id: authUser.wallet_address ?? authUser.sub,
    email: authUser.email ?? `${authUser.wallet_address}@web3.epsx.io`,
    name: authUser.wallet_address ? `Admin (${authUser.wallet_address.slice(0, 6)}...${authUser.wallet_address.slice(-4)})` : 'Admin',
    role: 'admin'
  } : serverUser

  return (
    <MainLayout user={layoutUser}>
      {children}
    </MainLayout>
  )
}