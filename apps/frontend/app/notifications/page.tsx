import { getUserNotifications } from '@/lib/actions/notifications'
import { NotificationHistoryClient } from '@/components/notifications/NotificationHistoryClient'
import { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Notifications - EPSX Platform',
  description: 'View and manage your notifications from the EPSX platform',
}

export default async function NotificationsPage() {
  const notificationData = await getUserNotifications({ per_page: 50, page: 1 })
  
  console.log('NotificationData:', JSON.stringify(notificationData, null, 2))
  
  return (
    <div className="container mx-auto py-8 px-4">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
          📊 Notification Center
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Stay updated with system alerts, data updates, and platform messages
        </p>
      </div>

      <NotificationHistoryClient 
        initialNotifications={notificationData.notifications}
        totalCount={notificationData.totalCount}
        unreadCount={notificationData.unreadCount}
      />
    </div>
  )
}