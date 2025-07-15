'use client';

import { DashboardView } from "@/components/dashboard/DashboardView"
import { ClientAuthGuard } from "@/components/auth/ClientAuthGuard"
import { useAuth } from "@/context/auth-context"

export default function DashboardPage() {
  const { user } = useAuth();

  // Convert Firebase user to app User type
  const appUser = user ? {
    id: user.uid,
    email: user.email || '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    emailVerified: user.emailVerified,
    role: 'USER' as const,
    displayName: user.displayName || undefined,
    photoURL: user.photoURL || undefined,
  } : null;

  return (
    <ClientAuthGuard>
      <main>
        {appUser && <DashboardView user={appUser} />}
      </main>
    </ClientAuthGuard>
  )
}
