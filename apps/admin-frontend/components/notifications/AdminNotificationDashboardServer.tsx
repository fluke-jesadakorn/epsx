/**
 * Server-rendered Notification Dashboard
 * Uses server actions for notifications and reduces client state
 */

import { Send, BarChart3, Users, Clock, Zap, Settings, Filter } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { AdminNotificationStats, RecentNotification } from '@/lib/api/notifications'
import { NotificationSendForm } from './NotificationSendForm'

interface AdminNotificationDashboardServerProps {
  stats: AdminNotificationStats
  recentNotifications: RecentNotification[]
}

function StatCard({ title, value, description, icon: Icon, colorClass }: {
  title: string
  value: string | number
  description: string
  icon: any
  colorClass: string
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className={`h-4 w-4 ${colorClass}`} />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">
          {typeof value === 'number' ? value.toLocaleString() : value}
        </div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  )
}

function NotificationsList({ notifications }: { notifications: RecentNotification[] }) {
  const getPriorityBadgeColor = (priority: string) => {
    switch (priority?.toLowerCase()) {
      case 'urgent': return 'bg-red-100 text-red-800'
      case 'high': return 'bg-orange-100 text-orange-800'
      case 'normal': return 'bg-blue-100 text-blue-800'
      case 'low': return 'bg-gray-100 text-gray-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  const getTypeBadgeColor = (type: string) => {
    switch (type?.toLowerCase()) {
      case 'security': return 'bg-red-100 text-red-800'
      case 'system': return 'bg-yellow-100 text-yellow-800'
      case 'feature': return 'bg-green-100 text-green-800'
      case 'admin': return 'bg-purple-100 text-purple-800'
      case 'data': return 'bg-blue-100 text-blue-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Recent Notifications
        </CardTitle>
        <CardDescription>
          Last {notifications.length} notifications sent
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {notifications.length === 0 ? (
            <p className="text-center text-muted-foreground py-8">
              No notifications sent yet
            </p>
          ) : (
            notifications.map((notification) => (
              <div key={notification.id} className="flex items-start space-x-4 p-4 border rounded-lg">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                      {notification.title}
                    </h4>
                    <div className="flex items-center gap-2 ml-2">
                      <Badge className={getPriorityBadgeColor(notification.priority)}>
                        {notification.priority}
                      </Badge>
                      <Badge className={getTypeBadgeColor(notification.type)}>
                        {notification.type}
                      </Badge>
                    </div>
                  </div>
                  <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                    {notification.body}
                  </p>
                  <div className="flex items-center justify-between mt-3">
                    <div className="flex items-center gap-4 text-xs text-gray-500">
                      <span>Target: {notification.target_group || 'Unknown'}</span>
                      <span>•</span>
                      <span>Sent: {new Date(notification.created_at).toLocaleDateString()}</span>
                    </div>
                    <div className="flex items-center gap-1 text-xs">
                      <Users className="h-3 w-3" />
                      <span>{notification.recipient_count || 0} recipients</span>
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  )
}

export function AdminNotificationDashboardServer({ 
  stats, 
  recentNotifications 
}: AdminNotificationDashboardServerProps) {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Notifications</h2>
          <p className="text-muted-foreground">
            Manage and send notifications to users
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline">
            <Filter className="mr-2 h-4 w-4" />
            Filter
          </Button>
          <Button variant="outline">
            <Settings className="mr-2 h-4 w-4" />
            Settings
          </Button>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="Total Sent"
          value={stats.totalSent}
          description="All time notifications"
          icon={Send}
          colorClass="text-blue-600"
        />
        <StatCard
          title="This Month"
          value={stats.thisMonth}
          description="Notifications this month"
          icon={BarChart3}
          colorClass="text-green-600"
        />
        <StatCard
          title="Success Rate"
          value={`${Math.round((stats.delivered / stats.totalSent) * 100) || 0}%`}
          description="Delivery success rate"
          icon={Zap}
          colorClass="text-yellow-600"
        />
        <StatCard
          title="Active Users"
          value={stats.activeRecipients}
          description="Users receiving notifications"
          icon={Users}
          colorClass="text-purple-600"
        />
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* Send Notification Form */}
        <NotificationSendForm />

        {/* Recent Notifications List */}
        <div>
          <NotificationsList notifications={recentNotifications} />
        </div>
      </div>

      {/* Analytics Section */}
      <Card>
        <CardHeader>
          <CardTitle>Notification Analytics</CardTitle>
          <CardDescription>
            Delivery metrics and performance insights
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid gap-4 md:grid-cols-3">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">{stats.delivered}</div>
                <div className="text-sm text-muted-foreground">Delivered</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-yellow-600">{stats.pending}</div>
                <div className="text-sm text-muted-foreground">Pending</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600">{stats.failed}</div>
                <div className="text-sm text-muted-foreground">Failed</div>
              </div>
            </div>
            
            <div className="mt-6">
              <div className="text-sm font-medium mb-2">Delivery Rate Over Time</div>
              <div className="h-32 bg-muted rounded-lg flex items-center justify-center">
                <p className="text-muted-foreground">Chart visualization would go here</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}