import { NotificationHistoryClient } from '@/components/notifications/NotificationHistoryClient'
import { createNotificationsClient } from '@/shared/api/notifications'
import { createFrontendApiClient } from '@/shared/utils/api-client'
import { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Notifications - EPSX Platform',
  description: 'View and manage your notifications from the EPSX platform',
}

export default async function NotificationsPage() {
  try {
    const client = createNotificationsClient(createFrontendApiClient())
    const notificationData = await client.getNotifications({ limit: 50, page: 1 })

    // Map backend format to frontend format
    const mappedNotifications = notificationData.data.notifications.map(n => ({
      id: n.id,
      title: n.title,
      body: n.message,
      type: n.notification_type as 'system' | 'admin' | 'data' | 'feature' | 'security',
      priority: n.priority as 'urgent' | 'high' | 'normal' | 'low',
      createdAt: n.timestamp,
      read: !!n.read_at,
      actionUrl: n.action_url
    }))

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
          initialNotifications={mappedNotifications}
          totalCount={notificationData.data.total_count}
          unreadCount={notificationData.data.unread_count}
        />
      </div>
    )
  } catch (error) {
    // If there's an error (e.g., user not logged in), show empty state
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
          initialNotifications={[]}
          totalCount={0}
          unreadCount={0}
        />
      </div>
    )
  }
}