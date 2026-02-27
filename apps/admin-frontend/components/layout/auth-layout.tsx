'use client'

import { usePathname } from 'next/navigation'
import type { ReactNode } from 'react'

import { AdminAuthGate } from '@/components/auth/admin-auth-gate'
import { MainLayout } from './main-layout'

import { useSharedAuth } from '@/shared/components/auth'

interface AuthLayoutProps {
  children: ReactNode
  user?: {
    id: string
    email: string
    name?: string
    role: string
  } | null
}

// Pages that should NEVER have the admin layout
const NO_LAYOUT_PATHS = [
  '/login',
  '/unauthorized',
  '/access-denied',
  '/permissions/policies',
]

export function AuthLayout({ children, user: serverUser }: AuthLayoutProps) {
  const pathname = usePathname()
  const { user: authUser, isAuthenticated } = useSharedAuth()

  // Special pages that never get layout
  const isNoLayoutPage = NO_LAYOUT_PATHS.some(path => pathname === path || pathname.startsWith(path))
  if (isNoLayoutPage) {
    return <>{children}</>
  }

  // Use client-side auth user if available, otherwise use server user
  const layoutUser = authUser !== null ? {
    id: authUser.wallet_address,
    email: authUser.email ?? `${authUser.wallet_address}@web3.epsx.io`,
    name: authUser.wallet_address !== '' ? `Admin (${authUser.wallet_address.slice(0, 6)}...${authUser.wallet_address.slice(-4)})` : 'Admin',
    role: 'admin'
  } : serverUser

  // Gate: server has no user and client confirms unauthenticated
  // Overlay the gate on top of MainLayout so the page tree stays mounted
  // (avoids hooks count mismatch when RSC reconciles after router.refresh())
  const isGated = (serverUser === null || serverUser === undefined) && !isAuthenticated

  return (
    <>
      <MainLayout user={layoutUser}>
        {children}
      </MainLayout>
      {isGated && <AdminAuthGate />}
    </>
  )
}
