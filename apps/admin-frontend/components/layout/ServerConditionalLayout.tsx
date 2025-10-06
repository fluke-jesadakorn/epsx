import { ReactNode } from 'react'

import { ConditionalAdminLayout } from './ConditionalAdminLayout'

interface ServerConditionalLayoutProps {
  children: ReactNode
}

/**
 *
 * @param root0
 * @param root0.children
 */
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