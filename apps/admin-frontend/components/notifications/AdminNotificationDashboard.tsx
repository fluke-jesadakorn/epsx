'use client'

import { useState, useTransition } from 'react'
import { Send, BarChart3, Users, Clock, Zap, Settings, Filter } from 'lucide-react'
import { useRouter } from 'next/navigation'
import { toast } from 'sonner'
import { sendNotification } from '@/lib/actions/admin-notification-actions'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { AdminNotificationStats, RecentNotification } from '@/lib/api/notifications'

interface AdminNotificationDashboardProps {
  initialStats: AdminNotificationStats
  initialRecentNotifications: RecentNotification[]
}

interface NotificationForm {
  title: string
  body: string
  target: 'all_users' | 'admin_users' | 'premium_users' | 'specific_user'
  targetUserId?: string
  priority: 'low' | 'normal' | 'high' | 'urgent'
  type: 'system' | 'admin' | 'data' | 'feature' | 'security'
  imageUrl?: string
  actionUrl?: string
  customData?: string
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
        <div className="text-2xl font-bold">{value}</div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  )
}

function RecentNotificationCard({ notification }: { notification: RecentNotification }) {
  const statusColors = {
    sent: 'bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-100',
    delivering: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-950 dark:text-yellow-100',
    delivered: 'bg-green-100 text-green-800 dark:bg-green-950 dark:text-green-100',
    failed: 'bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-100'
  }

  const statusIcons = {
    sent: '📤',
    delivering: '⏳', 
    delivered: '✅',
    failed: '🔴'
  }

  return (
    <div className="p-3 border rounded-lg bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm">{statusIcons[notification.deliveryStatus]}</span>
            <h4 className="font-medium text-sm truncate">{notification.title}</h4>
            <Badge className={`text-xs ${statusColors[notification.deliveryStatus]}`}>
              {notification.deliveryStatus}
            </Badge>
          </div>
          <p className="text-xs text-gray-600 dark:text-gray-400 line-clamp-2 mb-2">
            {notification.body}
          </p>
          <div className="flex items-center gap-3 text-xs text-gray-500">
            <span>→ {notification.target}</span>
            <span>({notification.recipientCount} recipients)</span>
            <span>{new Date(notification.sentAt).toLocaleTimeString()}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export function AdminNotificationDashboard({
  initialStats,
  initialRecentNotifications
}: AdminNotificationDashboardProps) {
  // Provide default values for stats to prevent crashes
  const defaultStats = {
    todaysSent: 0,
    todaysDelivered: 0,
    successRate: 0,
    failed: 0,
    totalSent: 0,
    pending: 0,
    peakHour: '12:00',
    avgDeliveryTime: '0'
  }
  
  const [stats] = useState(initialStats || defaultStats)
  const [recentNotifications] = useState(initialRecentNotifications || [])
  const [isPending, startTransition] = useTransition()
  const router = useRouter()
  
  const [form, setForm] = useState<NotificationForm>({
    title: '',
    body: '',
    target: 'all_users',
    priority: 'normal',
    type: 'system'
  })

  const handleQuickTemplate = (template: 'maintenance' | 'security' | 'feature' | 'announcement') => {
    const templates = {
      maintenance: {
        title: 'Scheduled Maintenance',
        body: 'EPSX will undergo scheduled maintenance tonight from 2:00-4:00 AM UTC. Services may be briefly unavailable.',
        priority: 'high' as const,
        type: 'system' as const,
      },
      security: {
        title: 'Security Update',
        body: 'We have enhanced our security systems. Please review your account settings if needed.',
        priority: 'high' as const,
        type: 'security' as const,
      },
      feature: {
        title: 'New Feature Available',
        body: 'Check out our enhanced analytics dashboard with new charting capabilities!',
        priority: 'normal' as const,
        type: 'feature' as const,
      },
      announcement: {
        title: 'Platform Update',
        body: 'We have exciting updates to share about the EPSX platform improvements.',
        priority: 'normal' as const,
        type: 'admin' as const,
      }
    }
    
    setForm(prev => ({ ...prev, ...templates[template] }))
  }

  const handleSendNotification = async () => {
    if (!form.title.trim() || !form.body.trim()) return

    startTransition(async () => {
      try {
        // Parse custom data if provided
        let customData: Record<string, any> | undefined
        if (form.customData?.trim()) {
          try {
            customData = JSON.parse(form.customData)
          } catch (e) {
            toast.error('Invalid JSON in custom data field')
            return
          }
        }

        const result = await sendNotification({
          title: form.title,
          body: form.body,
          target: form.target,
          targetUserId: form.targetUserId,
          priority: form.priority,
          type: form.type,
          imageUrl: form.imageUrl,
          actionUrl: form.actionUrl,
          customData
        })

        if (result.success) {
          toast.success(`Notification sent successfully to ${result.recipientCount || 1} recipients!`)
          
          // Reset form after successful send
          setForm({
            title: '',
            body: '',
            target: 'all_users',
            priority: 'normal',
            type: 'system'
          })
          
          // Refresh the page to update stats and recent notifications
          router.refresh()
        } else {
          toast.error(result.error || 'Failed to send notification')
        }
      } catch (error) {
        console.error('Failed to send notification:', error)
        toast.error('Failed to send notification')
      }
    })
  }

  return (
    <div className="space-y-8">
      {/* Stats Dashboard */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Today's Sent"
          value={stats?.todaysSent || 0}
          description={`${stats?.todaysDelivered || 0} delivered`}
          icon={Send}
          colorClass="text-blue-600"
        />
        <StatCard
          title="Success Rate"
          value={`${(stats?.successRate || 0).toFixed(1)}%`}
          description={`${stats?.failed || 0} failed deliveries`}
          icon={BarChart3}
          colorClass="text-green-600"
        />
        <StatCard
          title="Total Recipients"
          value={stats?.totalSent || 0}
          description={`${stats?.pending || 0} pending`}
          icon={Users}
          colorClass="text-purple-600"
        />
        <StatCard
          title="Peak Hour"
          value={stats?.peakHour || '12:00'}
          description={`Avg delivery: ${stats?.avgDeliveryTime || '0'}ms`}
          icon={Clock}
          colorClass="text-orange-600"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Send Notification Panel */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Send className="h-5 w-5" />
              Send Notification
            </CardTitle>
            <CardDescription>
              Send notifications to users and administrators across the platform
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Quick Templates */}
            <div>
              <label className="text-sm font-medium mb-2 block">⚡ Quick Templates</label>
              <div className="grid grid-cols-2 gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleQuickTemplate('maintenance')}
                >
                  🔧 Maintenance
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleQuickTemplate('security')}
                >
                  🔒 Security
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleQuickTemplate('feature')}
                >
                  ✨ Feature
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleQuickTemplate('announcement')}
                >
                  📢 Announcement
                </Button>
              </div>
            </div>

            {/* Target Audience */}
            <div>
              <label className="text-sm font-medium mb-2 block">🎯 Target Audience</label>
              <Select value={form.target} onValueChange={(value: any) => setForm(prev => ({ ...prev, target: value }))}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all_users">All Users (1,247)</SelectItem>
                  <SelectItem value="admin_users">Admin Staff (12)</SelectItem>
                  <SelectItem value="premium_users">Premium Users (89)</SelectItem>
                  <SelectItem value="specific_user">Specific User</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {form.target === 'specific_user' && (
              <div>
                <label className="text-sm font-medium mb-2 block">User Email/ID</label>
                <Input
                  placeholder="user@example.com"
                  value={form.targetUserId || ''}
                  onChange={(e) => setForm(prev => ({ ...prev, targetUserId: e.target.value }))}
                />
              </div>
            )}

            {/* Message Content */}
            <div>
              <label className="text-sm font-medium mb-2 block">📝 Title</label>
              <Input
                placeholder="Notification title"
                value={form.title}
                onChange={(e) => setForm(prev => ({ ...prev, title: e.target.value }))}
              />
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Message</label>
              <Textarea
                placeholder="Notification message content..."
                rows={3}
                value={form.body}
                onChange={(e) => setForm(prev => ({ ...prev, body: e.target.value }))}
              />
            </div>

            {/* Priority and Type */}
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-sm font-medium mb-2 block">Priority</label>
                <Select value={form.priority} onValueChange={(value: any) => setForm(prev => ({ ...prev, priority: value }))}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">⚪ Low</SelectItem>
                    <SelectItem value="normal">🔵 Normal</SelectItem>
                    <SelectItem value="high">🟡 High</SelectItem>
                    <SelectItem value="urgent">🔴 Urgent</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="text-sm font-medium mb-2 block">Type</label>
                <Select value={form.type} onValueChange={(value: any) => setForm(prev => ({ ...prev, type: value }))}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="system">🔧 System</SelectItem>
                    <SelectItem value="admin">👨‍💼 Admin</SelectItem>
                    <SelectItem value="data">📊 Data</SelectItem>
                    <SelectItem value="feature">✨ Feature</SelectItem>
                    <SelectItem value="security">🔒 Security</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Optional Fields */}
            <details className="space-y-3">
              <summary className="text-sm font-medium cursor-pointer hover:text-blue-600">
                🎛️ Advanced Options
              </summary>
              <div className="space-y-3 pt-3">
                <div>
                  <label className="text-sm font-medium mb-2 block">Image URL</label>
                  <Input
                    placeholder="https://example.com/image.jpg"
                    value={form.imageUrl || ''}
                    onChange={(e) => setForm(prev => ({ ...prev, imageUrl: e.target.value }))}
                  />
                </div>
                <div>
                  <label className="text-sm font-medium mb-2 block">Action URL</label>
                  <Input
                    placeholder="/dashboard or https://epsx.io/page"
                    value={form.actionUrl || ''}
                    onChange={(e) => setForm(prev => ({ ...prev, actionUrl: e.target.value }))}
                  />
                </div>
                <div>
                  <label className="text-sm font-medium mb-2 block">Custom Data (JSON)</label>
                  <Textarea
                    placeholder='{"custom_key": "custom_value"}'
                    rows={2}
                    value={form.customData || ''}
                    onChange={(e) => setForm(prev => ({ ...prev, customData: e.target.value }))}
                  />
                </div>
              </div>
            </details>

            <Button 
              onClick={handleSendNotification}
              disabled={isPending || !form.title.trim() || !form.body.trim()}
              className="w-full"
            >
              {isPending ? (
                <>
                  <div className="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full mr-2"></div>
                  Sending...
                </>
              ) : (
                <>
                  <Send className="h-4 w-4 mr-2" />
                  Send Notification
                </>
              )}
            </Button>
          </CardContent>
        </Card>

        {/* Recent Notifications */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="h-5 w-5" />
              Recent Notifications
            </CardTitle>
            <CardDescription>
              Latest notification activity and delivery status
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {recentNotifications.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  <Send className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">No recent notifications</p>
                </div>
              ) : (
                recentNotifications.map((notification, index) => (
                  <RecentNotificationCard key={`${notification.id}-${index}`} notification={notification} />
                ))
              )}
            </div>

            <div className="border-t pt-4 mt-4 flex justify-between items-center">
              <Button variant="outline" size="sm">
                <BarChart3 className="h-4 w-4 mr-2" />
                View Analytics
              </Button>
              <Button variant="outline" size="sm">
                <Settings className="h-4 w-4 mr-2" />
                Settings
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Live Feed */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5" />
            🔔 Live Activity Feed
          </CardTitle>
          <CardDescription>
            Real-time notification delivery events
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-black text-green-400 p-4 rounded-lg font-mono text-sm">
            <div className="space-y-1">
              <div>16:45:23 → user123@domain.com: Notification delivered</div>
              <div>16:45:22 → admin-users topic: Broadcast sent (12 recipients)</div>
              <div>16:45:19 → Token registered: c2HTa1OCBF...</div>
              <div>16:45:15 → user456@domain.com: Notification clicked</div>
              <div className="text-green-500">■ Ready for new notifications</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}