import { Metadata } from 'next'
import { getAdminNotificationStats, getRecentNotifications } from '@/lib/api/notifications'
import { AdminNotificationDashboard } from '@/components/notifications/AdminNotificationDashboard'

export const metadata: Metadata = {
  title: 'Notifications - EPSX Admin',
  description: 'Send and manage notifications in the EPSX platform',
}

export default async function AdminNotificationsPage() {
  const [stats, recentNotifications] = await Promise.all([
    getAdminNotificationStats(),
    getRecentNotifications(15)
  ])

  return (
    <AdminNotificationDashboard 
      initialStats={stats}
      initialRecentNotifications={recentNotifications}
    />
  )
}