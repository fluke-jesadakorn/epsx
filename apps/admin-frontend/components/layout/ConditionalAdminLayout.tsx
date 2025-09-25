'use client'

import { usePathname } from 'next/navigation'
import { PancakeAdminLayout } from './PancakeAdminLayout'
import { ReactNode } from 'react'

interface ConditionalAdminLayoutProps {
  children: ReactNode
  user?: {
    id: string
    email: string
    name?: string
    role: string
  }
}

// Pages that should NOT have the admin layout
const NO_LAYOUT_PATHS = [
  '/login',
  '/unauthorized', 
  '/access-denied',
  '/request-access'
]

export function ConditionalAdminLayout({ children, user }: ConditionalAdminLayoutProps) {
  const pathname = usePathname()
  
  // Check if current path should have admin layout
  const shouldHaveLayout = !NO_LAYOUT_PATHS.some(path => pathname === path)
  
  if (!shouldHaveLayout) {
    return <>{children}</>
  }
  
  return (
    <PancakeAdminLayout user={user}>
      {children}
    </PancakeAdminLayout>
  )
}