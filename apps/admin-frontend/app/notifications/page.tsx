import { Metadata } from 'next'

import { AdminNotificationDashboard, AdminNotificationStats, RecentNotification } from '@/components/notifications/AdminNotificationDashboard'
import { createNotificationsClient } from '@/shared/api/notifications'
import { createAdminApiClient } from '@/shared/utils/api-client'

export const metadata: Metadata = {
  title: 'Notifications - EPSX Admin',
  description: 'Send and manage notifications in the EPSX platform',
}

/**
 *
 */
export default async function AdminNotificationsPage() {
  const client = createNotificationsClient(createAdminApiClient())

  // Default empty stats (backend endpoints currently commented out)
  const defaultStats: AdminNotificationStats = {
    totalSent: 0,
    delivered: 0,
    failed: 0,
    pending: 0,
    successRate: 0,
    todaysSent: 0,
    todaysDelivered: 0,
    avgDeliveryTime: 0,
    peakHour: '12:00'
  }

  const defaultNotifications: RecentNotification[] = []

  // Try to fetch data, but gracefully handle errors since endpoints are commented out
  let stats = defaultStats
  let recentNotifications = defaultNotifications

  try {
    const statsResponse = await client.getNotificationStats()
    // Map shared client response to component format if needed
    stats = {
      totalSent: statsResponse.data.total_notifications,
      delivered: statsResponse.data.sent_today,
      failed: 0,
      pending: 0,
      successRate: statsResponse.data.delivery_rate * 100,
      todaysSent: statsResponse.data.sent_today,
      todaysDelivered: Math.floor(statsResponse.data.sent_today * statsResponse.data.delivery_rate),
      avgDeliveryTime: 0,
      peakHour: '12:00'
    }
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.warn('Failed to fetch notification stats (backend endpoint may be commented out):', _error)
  }

  try {
    const notifResponse = await client.getAllNotifications({ page: 1, limit: 15 })
    // Map to component format
    recentNotifications = notifResponse.data.notifications.map(n => ({
      id: n.id,
      title: n.title,
      body: n.message,
      target: 'all_users',
      sentAt: n.timestamp,
      recipientCount: 1,
      deliveryStatus: n.delivered_at ? 'delivered' : 'sent' as const,
      priority: n.priority,
      type: n.notification_type
    }))
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.warn('Failed to fetch recent notifications (backend endpoint may be commented out):', _error)
  }

  return (
    <AdminNotificationDashboard
      initialStats={stats}
      initialRecentNotifications={recentNotifications}
    />
  )
}