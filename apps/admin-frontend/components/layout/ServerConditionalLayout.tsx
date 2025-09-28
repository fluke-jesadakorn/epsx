import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { ConditionalAdminLayout } from './ConditionalAdminLayout'
import { ReactNode } from 'react'

interface ServerConditionalLayoutProps {
  children: ReactNode
}

export async function ServerConditionalLayout({ children }: ServerConditionalLayoutProps) {
  // Get user session for layout
  const session = await UnifiedAuth.getSession()
  const layoutUser = session?.user ? {
    id: session.user.walletAddress,
    email: '',
    name: session.user.displayName || session.user.walletAddress,
    role: session.hasAdminAccess ? 'admin' : 'user'
  } : undefined

  return (
    <ConditionalAdminLayout user={layoutUser}>
      {children}
    </ConditionalAdminLayout>
  )
}