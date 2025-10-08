import { ReactNode } from 'react'

import { AuthLayout } from './AuthLayout'

interface LayoutWrapperProps {
  children: ReactNode
}

/**
 *
 * @param root0
 * @param root0.children
 */
export async function LayoutWrapper({ children }: LayoutWrapperProps) {
  // With SharedOpenIDWeb3Provider, authentication is handled client-side
  // Server components cannot access localStorage tokens
  // Pass undefined user and let client-side auth populate it

  return (
    <AuthLayout user={undefined}>
      {children}
    </AuthLayout>
  )
}