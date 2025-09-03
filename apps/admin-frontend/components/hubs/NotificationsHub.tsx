import React from 'react'
import { Bell, AlertTriangle, Info, CheckCircle, Clock, Users, Shield, Settings } from 'lucide-react'
import { ServerNotificationAPI } from '@/lib/api/server-admin-api'
import NotificationActions from '@/components/notifications/NotificationActions'
import InteractiveNotificationCard from '@/components/notifications/InteractiveNotificationCard'
import PushMessageManager from '@/components/notifications/PushMessageManager'
import SystemSettingsDashboard from '@/components/notifications/SystemSettingsDashboard'

/**
 * Windows Phone-style Notifications Hub
 * Real-time notification management with filtering and actions
 */


function NotificationStatsCards({ notifications, unreadCount }: { 
  notifications: any[]
  unreadCount: number 
}) {
  const stats = {
    total: notifications.length,
    unread: unreadCount,
    security: notifications.filter(n => n.type === 'security').length,
    system: notifications.filter(n => n.type === 'system').length,
    recent: notifications.filter(n => new Date(n.created_at) > new Date(Date.now() - 24 * 60 * 60 * 1000)).length
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
      <div className="bg-blue-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">📨 Total</h3>
        <p className="text-2xl font-bold">{stats.total}</p>
        <p className="text-xs opacity-75">All notifications</p>
      </div>
      
      <div className="bg-orange-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">🔔 Unread</h3>
        <p className="text-2xl font-bold">{stats.unread}</p>
        <p className="text-xs opacity-75">Need attention</p>
      </div>
      
      <div className="bg-red-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">🛡️ Security</h3>
        <p className="text-2xl font-bold">{stats.security}</p>
        <p className="text-xs opacity-75">Security alerts</p>
      </div>
      
      <div className="bg-green-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">⚙️ System</h3>
        <p className="text-2xl font-bold">{stats.system}</p>
        <p className="text-xs opacity-75">System events</p>
      </div>
      
      <div className="bg-purple-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">🕒 Recent</h3>
        <p className="text-2xl font-bold">{stats.recent}</p>
        <p className="text-xs opacity-75">Last 24 hours</p>
      </div>
    </div>
  )
}


export default async function NotificationsHub() {
  // Fetch data server-side
  const [notifications, unreadCountData] = await Promise.allSettled([
    ServerNotificationAPI.getNotifications(1, 50),
    ServerNotificationAPI.getUnreadCount()
  ])

  const notificationList = notifications.status === 'fulfilled' ? notifications.value : []
  const unreadCount = unreadCountData.status === 'fulfilled' ? unreadCountData.value.count : 0

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-4xl font-light text-gray-900 dark:text-white mb-2">
          🔔 NOTIFICATIONS HUB
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Real-time notification management with filtering and actions
        </p>
      </div>

      {/* Statistics Cards */}
      <NotificationStatsCards notifications={notificationList} unreadCount={unreadCount} />

      {/* Pivot Navigation */}
      <div className="mb-6">
        <div className="flex overflow-x-auto gap-1 border-b border-gray-200 dark:border-gray-700">
          <button className="px-4 py-3 font-medium text-blue-600 border-b-2 border-blue-600 whitespace-nowrap">
            ◄ ALL ►
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            UNREAD
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            SECURITY
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            SYSTEM
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            USERS
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex flex-col sm:flex-row gap-4 mb-6">
        <NotificationActions />
        
        <div className="flex-1 max-w-sm">
          <input 
            type="text"
            placeholder="Search notifications..."
            className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
      </div>

      {/* Real-time Status */}
      <div className="mb-6 p-4 bg-gradient-to-r from-green-500 to-blue-500 text-white rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <div className="w-3 h-3 bg-green-300 rounded-full animate-pulse"></div>
              📡 Real-time Status
            </h3>
            <p className="text-sm opacity-90">
              Connected • Last update: {new Date().toLocaleTimeString()} • {unreadCount} unread notifications
            </p>
          </div>
          <div className="text-2xl font-bold">
            LIVE
          </div>
        </div>
      </div>

      {/* Notifications List */}
      <div className="space-y-4">
        {notificationList.length > 0 ? (
          notificationList.map((notification: any) => (
            <InteractiveNotificationCard
              key={notification.id}
              notification={notification}
            />
          ))
        ) : (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            <Bell size={48} className="mx-auto mb-4 opacity-50" />
            <h3 className="text-lg font-medium mb-2">No notifications</h3>
            <p>You're all caught up! New notifications will appear here.</p>
          </div>
        )}
      </div>

      {/* Load More */}
      {notificationList.length > 0 && (
        <div className="text-center mt-8">
          <button className="px-6 py-3 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors">
            Load More Notifications
          </button>
        </div>
      )}

      {/* Push Message Manager */}
      <div className="mt-12">
        <PushMessageManager />
      </div>

      {/* System Settings Dashboard */}
      <div className="mt-12">
        <SystemSettingsDashboard />
      </div>

      {/* Admin FCM Quick Status */}
      <div className="mt-8 bg-gradient-to-r from-yellow-500 to-orange-500 text-white p-6 rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold flex items-center gap-2 mb-2">
              <Shield size={20} />
              🔔 Admin Push Notifications
            </h3>
            <p className="text-sm opacity-90">
              Real-time admin alerts delivered directly to your devices
            </p>
          </div>
          <div className="text-right">
            <div className="text-2xl font-bold mb-1">FCM</div>
            <div className="text-xs opacity-90">Enhanced Admin</div>
          </div>
        </div>
      </div>

      {/* Notification Settings */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            🔧 Notification Preferences
          </h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Security Alerts:</span>
              <span className="text-green-600 font-medium">🟢 Enabled</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">System Updates:</span>
              <span className="text-green-600 font-medium">🟢 Enabled</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">User Activities:</span>
              <span className="text-green-600 font-medium">🟢 Enabled</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Performance Alerts:</span>
              <span className="text-orange-600 font-medium">🟡 Limited</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">FCM Push Notifications:</span>
              <span className="text-green-600 font-medium">🟢 Available</span>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            📊 Delivery Channels
          </h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">In-App Notifications:</span>
              <span className="text-green-600 font-medium">🟢 Active</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Admin FCM Push:</span>
              <span className="text-green-600 font-medium">🟢 Enhanced</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Email Notifications:</span>
              <span className="text-green-600 font-medium">🟢 Active</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Real-time Events:</span>
              <span className="text-blue-600 font-medium">🔵 SSE + FCM</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Admin Webhooks:</span>
              <span className="text-blue-600 font-medium">🔵 Configured</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}