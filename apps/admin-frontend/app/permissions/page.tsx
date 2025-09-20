import { redirect } from 'next/navigation'

export const dynamic = 'force-dynamic'

interface PermissionsPageProps {
  searchParams?: Promise<{
    mode?: string
    userId?: string
    email?: string
  }>
}

// Redirect to Web3 permissions page
export default async function AdminPermissionsPage(props: PermissionsPageProps) {
  // Preserve search params when redirecting
  const params = props.searchParams ? await props.searchParams : {}
  const searchParamsString = new URLSearchParams(params as Record<string, string>).toString()
  const redirectUrl = `/permissions/web3${searchParamsString ? `?${searchParamsString}` : ''}`
  
  console.log('🔄 Permissions: Redirecting to Web3 permissions page:', redirectUrl)
  redirect(redirectUrl)
}