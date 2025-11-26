import { redirect } from 'next/navigation'

export const dynamic = 'force-dynamic'

interface PermissionsPageProps {
  searchParams?: Promise<{
    mode?: string
    userId?: string
    email?: string
  }>
}

/**
 * Redirect to permissions page
 * @deprecated Use /web3-permissions directly
 */
export default async function AdminPermissionsPage(props: PermissionsPageProps) {
  // Preserve search params when redirecting
  const params = props.searchParams ? await props.searchParams : {}
  const searchParamsString = new URLSearchParams(params as Record<string, string>).toString()
  const redirectUrl = `/web3-permissions${searchParamsString ? `?${searchParamsString}` : ''}`

  redirect(redirectUrl)
}