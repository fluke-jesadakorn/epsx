import { NotificationBellClient } from './NotificationBellClient'

// Mock function for notifications (to be implemented)
async function getUnreadNotifications() {
  // TODO: Implement actual notification fetching
  return [];
}

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