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
    <div className="container mx-auto py-8 px-4">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
          📤 Notification Control Center
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Send real-time notifications to users and administrators across the EPSX platform
        </p>
      </div>
      
      <AdminNotificationDashboard 
        initialStats={stats}
        initialRecentNotifications={recentNotifications}
      />
    </div>
  )
}