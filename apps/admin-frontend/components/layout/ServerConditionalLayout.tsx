import { ConditionalAdminLayout } from './ConditionalAdminLayout'
import { ReactNode } from 'react'

interface ServerConditionalLayoutProps {
  children: ReactNode
}

export async function ServerConditionalLayout({ children }: ServerConditionalLayoutProps) {
  // With SharedOpenIDWeb3Provider, authentication is handled client-side
  // Server components cannot access localStorage tokens
  // Pass undefined user and let client-side auth populate it
  
  return (
    <ConditionalAdminLayout user={undefined}>
      {children}
    </ConditionalAdminLayout>
  )
}