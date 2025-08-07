/**
 * User Profile Default Page
 * Redirects to the overview tab
 */

import { redirect } from 'next/navigation'

interface UserProfilePageProps {
  params: Promise<{ userId: string }>
}

export default async function UserProfilePage({ params }: UserProfilePageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Redirect to the overview tab
  redirect(`/users/${userId}/overview`)
}