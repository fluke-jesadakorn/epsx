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
    id: session.user.sub,
    email: session.user.email,
    name: session.user.name,
    role: session.hasAdminAccess ? 'admin' : 'user'
  } : undefined

  return (
    <ConditionalAdminLayout user={layoutUser}>
      {children}
    </ConditionalAdminLayout>
  )
}