import { getUnreadNotifications } from '@/lib/api/notifications'
import { NotificationBellClient } from './NotificationBellClient'

// Server Component that fetches notification data
export default async function NotificationBell() {
  const notifications = await getUnreadNotifications()
  const count = notifications.length
  
  return (
    <NotificationBellClient 
      count={count}
      recentNotifications={notifications.slice(0, 5)}
    />
  )
}