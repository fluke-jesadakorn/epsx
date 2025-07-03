export const dynamic = 'force-dynamic'

import { getCurrentUser } from "@/app/actions/auth"
import { DashboardView } from "@/components/dashboard/DashboardView"
import { redirect } from 'next/navigation'

export default async function DashboardPage() {
  const user = await getCurrentUser()

  // If no user is found, this should not redirect here as middleware should handle it
  // But if we somehow get here without a user, redirect to login
  if (!user) {
    redirect('/login?returnUrl=/dashboard')
  }

  return (
    <main>
      <DashboardView user={user} />
    </main>
  )
}
