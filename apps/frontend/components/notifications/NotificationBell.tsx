import { NotificationBellClient } from './NotificationBellClient'
import { createNotificationsClient } from '@/shared/api/notifications'
import { createFrontendApiClient } from '@/shared/utils/api-client'

// Server Component that fetches notification data
export default async function NotificationBell() {
  try {
    const client = createNotificationsClient(createFrontendApiClient())

    const data = await client.getNotifications({
      page: 1,
      limit: 5,
      status: 'unread'
    })

    const count = data.data.unread_count
    const notifications = data.data.notifications.map(n => ({
      id: n.id,
      title: n.title,
      body: n.message,
      type: n.notification_type as 'system' | 'admin' | 'data' | 'feature' | 'security',
      priority: n.priority as 'urgent' | 'high' | 'normal' | 'low',
      createdAt: n.timestamp,
      actionUrl: n.action_url
    }))

    return (
      <NotificationBellClient
        count={count}
        recentNotifications={notifications}
      />
    )
  } catch (error) {
    // If there's an error (e.g., user not logged in), show empty state
    return (
      <NotificationBellClient
        count={0}
        recentNotifications={[]}
      />
    )
  }
}