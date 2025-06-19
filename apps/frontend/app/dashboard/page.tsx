export const dynamic = 'force-dynamic'

import { getCurrentUser } from "@/app/actions/auth"
import { DashboardView } from "@/components/dashboard/DashboardView"

export default async function DashboardPage() {
  const user = await getCurrentUser()

  return (
    <main>
      <DashboardView user={user} />
    </main>
  )
}
