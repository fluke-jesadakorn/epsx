'use client'

import { useState, useTransition } from 'react'
import { Send, BarChart3, Users, Clock, Zap, Settings, Filter, Sparkles, Bell } from 'lucide-react'
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

function StatCard({ title, value, description, icon: Icon, gradient }: {
  title: string
  value: string | number
  description: string
  icon: any
  gradient: string
}) {
  // Map gradient to icon color
  const getIconColor = () => {
    if (gradient.includes('blue') || gradient.includes('cyan')) return 'text-blue-500'
    if (gradient.includes('green') || gradient.includes('emerald')) return 'text-green-500'
    if (gradient.includes('purple') || gradient.includes('pink')) return 'text-purple-500'
    if (gradient.includes('orange') || gradient.includes('yellow')) return 'text-orange-500'
    return 'text-blue-500'
  }

  const getStatusLabel = () => {
    if (gradient.includes('blue') || gradient.includes('cyan')) return 'Today'
    if (gradient.includes('green') || gradient.includes('emerald')) return 'Success'
    if (gradient.includes('purple') || gradient.includes('pink')) return 'Total'
    if (gradient.includes('orange') || gradient.includes('yellow')) return 'Peak'
    return 'Status'
  }

  const getBorderColor = () => {
    if (gradient.includes('blue') || gradient.includes('cyan')) return 'border-blue-300 dark:border-blue-700'
    if (gradient.includes('green') || gradient.includes('emerald')) return 'border-green-300 dark:border-green-700'
    if (gradient.includes('purple') || gradient.includes('pink')) return 'border-purple-300 dark:border-purple-700'
    if (gradient.includes('orange') || gradient.includes('yellow')) return 'border-orange-300 dark:border-orange-700'
    return 'border-blue-300 dark:border-blue-700'
  }

  return (
    <div className={`bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 shadow-xl border-2 hover:shadow-2xl transition-shadow ${getBorderColor()}`}>
      <div className="flex items-center justify-between mb-4">
        <Icon className={`h-8 w-8 ${getIconColor()}`} />
        <span className="text-sm font-medium text-gray-500 dark:text-gray-400">{getStatusLabel()}</span>
      </div>
      <div className="space-y-1">
        <div className="text-3xl font-bold text-gray-900 dark:text-white">{value}</div>
        <div className="text-sm text-gray-600 dark:text-gray-300">{title}</div>
        <div className="text-xs text-gray-500 dark:text-gray-400">{description}</div>
      </div>
    </div>
  )
}

function RecentNotificationCard({ notification }: { notification: RecentNotification }) {
  const statusColors = {
    sent: 'bg-gradient-to-r from-blue-500 to-cyan-500',
    delivering: 'bg-gradient-to-r from-yellow-500 to-orange-500',
    delivered: 'bg-gradient-to-r from-green-500 to-emerald-500',
    failed: 'bg-gradient-to-r from-red-500 to-pink-500'
  }

  const statusIcons = {
    sent: '📤',
    delivering: '⏳', 
    delivered: '✅',
    failed: '🔴'
  }

  return (
    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-gray-100/50 to-gray-200/50 dark:from-gray-800/50 dark:to-gray-700/50 p-0.5 group hover:scale-[1.02] transition-all duration-200">
      <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-sm rounded-2xl p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-3 mb-2">
              <span className="text-lg">{statusIcons[notification.deliveryStatus]}</span>
              <h4 className="font-semibold text-sm truncate text-gray-800 dark:text-gray-200">{notification.title}</h4>
              <div className={`px-3 py-1 rounded-full text-xs font-medium text-white ${statusColors[notification.deliveryStatus]}`}>
                {notification.deliveryStatus}
              </div>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2 mb-3">
              {notification.body}
            </p>
            <div className="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400">
              <span className="flex items-center gap-1">
                <Users className="h-3 w-3" />
                {notification.target}
              </span>
              <span className="flex items-center gap-1">
                <Bell className="h-3 w-3" />
                {notification.recipientCount} recipients
              </span>
              <span className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                {new Date(notification.sentAt).toLocaleTimeString()}
              </span>
            </div>
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
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>

      <div className="relative space-y-8">
        {/* Page Header */}
        <div className="text-center mb-12">
          <div className="relative inline-block">
            <h1 className="text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              📤 Notification Control Center
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full animate-pulse"></div>
          </div>
          <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Send real-time notifications to users and administrators across the EPSX platform
          </p>
        </div>

        {/* Stats Dashboard */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <StatCard
            title="Today's Sent"
            value={stats?.todaysSent || 0}
            description={`${stats?.todaysDelivered || 0} delivered`}
            icon={Send}
            gradient="bg-gradient-to-r from-blue-500 to-cyan-500"
          />
          <StatCard
            title="Success Rate"
            value={`${(stats?.successRate || 0).toFixed(1)}%`}
            description={`${stats?.failed || 0} failed deliveries`}
            icon={BarChart3}
            gradient="bg-gradient-to-r from-green-500 to-emerald-500"
          />
          <StatCard
            title="Total Recipients"
            value={stats?.totalSent || 0}
            description={`${stats?.pending || 0} pending`}
            icon={Users}
            gradient="bg-gradient-to-r from-purple-500 to-pink-500"
          />
          <StatCard
            title="Peak Hour"
            value={stats?.peakHour || '12:00'}
            description={`Avg delivery: ${stats?.avgDeliveryTime || '0'}ms`}
            icon={Clock}
            gradient="bg-gradient-to-r from-orange-500 to-yellow-500"
          />
        </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Send Notification Panel */}
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 via-orange-400/20 to-pink-400/20 p-0.5 group">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl">
            {/* Floating decoration */}
            <div className="absolute top-4 right-4 w-6 h-6 bg-gradient-to-r from-yellow-400/40 to-orange-500/40 rounded-full blur-sm animate-pulse"></div>
            
            <div className="p-8">
              <div className="mb-6">
                <h2 className="flex items-center gap-3 text-2xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 to-pink-600 bg-clip-text text-transparent">
                  <div className="p-2 rounded-2xl bg-gradient-to-r from-yellow-500 to-orange-500">
                    <Send className="h-6 w-6 text-white" />
                  </div>
                  Send Notification
                </h2>
                <p className="text-gray-600 dark:text-gray-400 mt-2 ml-14">
                  Send notifications to users and administrators across the platform
                </p>
              </div>
              <div className="space-y-6">
            {/* Quick Templates */}
            <div>
              <label className="text-sm font-semibold mb-3 block flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-yellow-500" />
                Quick Templates
              </label>
              <div className="grid grid-cols-2 gap-3">
                <button
                  className="flex items-center gap-2 px-4 py-3 bg-gradient-to-r from-orange-400/10 to-red-400/10 hover:from-orange-400/20 hover:to-red-400/20 border border-orange-200 dark:border-orange-700 rounded-2xl text-sm font-medium transition-all duration-200 hover:scale-105"
                  onClick={() => handleQuickTemplate('maintenance')}
                >
                  🔧 Maintenance
                </button>
                <button
                  className="flex items-center gap-2 px-4 py-3 bg-gradient-to-r from-red-400/10 to-pink-400/10 hover:from-red-400/20 hover:to-pink-400/20 border border-red-200 dark:border-red-700 rounded-2xl text-sm font-medium transition-all duration-200 hover:scale-105"
                  onClick={() => handleQuickTemplate('security')}
                >
                  🔒 Security
                </button>
                <button
                  className="flex items-center gap-2 px-4 py-3 bg-gradient-to-r from-blue-400/10 to-purple-400/10 hover:from-blue-400/20 hover:to-purple-400/20 border border-blue-200 dark:border-blue-700 rounded-2xl text-sm font-medium transition-all duration-200 hover:scale-105"
                  onClick={() => handleQuickTemplate('feature')}
                >
                  ✨ Feature
                </button>
                <button
                  className="flex items-center gap-2 px-4 py-3 bg-gradient-to-r from-green-400/10 to-teal-400/10 hover:from-green-400/20 hover:to-teal-400/20 border border-green-200 dark:border-green-700 rounded-2xl text-sm font-medium transition-all duration-200 hover:scale-105"
                  onClick={() => handleQuickTemplate('announcement')}
                >
                  📢 Announcement
                </button>
              </div>
            </div>

            {/* Target Audience */}
            <div>
              <label className="text-sm font-semibold mb-3 block flex items-center gap-2">
                <Users className="h-4 w-4 text-purple-500" />
                Target Audience
              </label>
              <Select value={form.target} onValueChange={(value: any) => setForm(prev => ({ ...prev, target: value }))}>
                <SelectTrigger className="rounded-2xl border-2 border-purple-200 dark:border-purple-700 bg-gradient-to-r from-purple-50/50 to-pink-50/50 dark:from-purple-900/20 dark:to-pink-900/20 hover:border-purple-300 dark:hover:border-purple-600 transition-all duration-200">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="rounded-2xl border-2 border-purple-200 dark:border-purple-700">
                  <SelectItem value="all_users" className="rounded-xl">All Users (1,247)</SelectItem>
                  <SelectItem value="admin_users" className="rounded-xl">Admin Staff (12)</SelectItem>
                  <SelectItem value="premium_users" className="rounded-xl">Premium Users (89)</SelectItem>
                  <SelectItem value="specific_user" className="rounded-xl">Specific User</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {form.target === 'specific_user' && (
              <div>
                <label className="text-sm font-semibold mb-3 block">User Email/ID</label>
                <Input
                  placeholder="user@example.com"
                  value={form.targetUserId || ''}
                  onChange={(e) => setForm(prev => ({ ...prev, targetUserId: e.target.value }))}
                  className="rounded-2xl border-2 border-blue-200 dark:border-blue-700 bg-gradient-to-r from-blue-50/50 to-cyan-50/50 dark:from-blue-900/20 dark:to-cyan-900/20 hover:border-blue-300 dark:hover:border-blue-600 transition-all duration-200"
                />
              </div>
            )}

            {/* Message Content */}
            <div>
              <label className="text-sm font-semibold mb-3 block flex items-center gap-2">
                <Bell className="h-4 w-4 text-yellow-500" />
                Title
              </label>
              <Input
                placeholder="Notification title"
                value={form.title}
                onChange={(e) => setForm(prev => ({ ...prev, title: e.target.value }))}
                className="rounded-2xl border-2 border-yellow-200 dark:border-yellow-700 bg-gradient-to-r from-yellow-50/50 to-orange-50/50 dark:from-yellow-900/20 dark:to-orange-900/20 hover:border-yellow-300 dark:hover:border-yellow-600 transition-all duration-200"
              />
            </div>

            <div>
              <label className="text-sm font-semibold mb-3 block flex items-center gap-2">
                <Settings className="h-4 w-4 text-orange-500" />
                Message
              </label>
              <Textarea
                placeholder="Notification message content..."
                rows={3}
                value={form.body}
                onChange={(e) => setForm(prev => ({ ...prev, body: e.target.value }))}
                className="rounded-2xl border-2 border-orange-200 dark:border-orange-700 bg-gradient-to-r from-orange-50/50 to-pink-50/50 dark:from-orange-900/20 dark:to-pink-900/20 hover:border-orange-300 dark:hover:border-orange-600 transition-all duration-200 resize-none"
              />
            </div>

            {/* Priority and Type */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-semibold mb-3 block flex items-center gap-2">
                  <Filter className="h-4 w-4 text-red-500" />
                  Priority
                </label>
                <Select value={form.priority} onValueChange={(value: any) => setForm(prev => ({ ...prev, priority: value }))}>
                  <SelectTrigger className="rounded-2xl border-2 border-red-200 dark:border-red-700 bg-gradient-to-r from-red-50/50 to-pink-50/50 dark:from-red-900/20 dark:to-pink-900/20 hover:border-red-300 dark:hover:border-red-600 transition-all duration-200">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="rounded-2xl border-2 border-red-200 dark:border-red-700">
                    <SelectItem value="low" className="rounded-xl">⚪ Low</SelectItem>
                    <SelectItem value="normal" className="rounded-xl">🔵 Normal</SelectItem>
                    <SelectItem value="high" className="rounded-xl">🟡 High</SelectItem>
                    <SelectItem value="urgent" className="rounded-xl">🔴 Urgent</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="text-sm font-semibold mb-3 block flex items-center gap-2">
                  <Settings className="h-4 w-4 text-green-500" />
                  Type
                </label>
                <Select value={form.type} onValueChange={(value: any) => setForm(prev => ({ ...prev, type: value }))}>
                  <SelectTrigger className="rounded-2xl border-2 border-green-200 dark:border-green-700 bg-gradient-to-r from-green-50/50 to-teal-50/50 dark:from-green-900/20 dark:to-teal-900/20 hover:border-green-300 dark:hover:border-green-600 transition-all duration-200">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="rounded-2xl border-2 border-green-200 dark:border-green-700">
                    <SelectItem value="system" className="rounded-xl">🔧 System</SelectItem>
                    <SelectItem value="admin" className="rounded-xl">👨‍💼 Admin</SelectItem>
                    <SelectItem value="data" className="rounded-xl">📊 Data</SelectItem>
                    <SelectItem value="feature" className="rounded-xl">✨ Feature</SelectItem>
                    <SelectItem value="security" className="rounded-xl">🔒 Security</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Optional Fields */}
            <details className="group space-y-4">
              <summary className="flex items-center gap-2 text-sm font-semibold cursor-pointer hover:text-blue-600 dark:hover:text-blue-400 transition-colors duration-200 p-3 rounded-2xl bg-gradient-to-r from-blue-50/30 to-purple-50/30 dark:from-blue-900/20 dark:to-purple-900/20 border border-blue-200/50 dark:border-blue-700/50">
                <Sparkles className="h-4 w-4 text-blue-500" />
                Advanced Options
                <div className="ml-auto transform transition-transform group-open:rotate-180">
                  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </div>
              </summary>
              <div className="space-y-4 pt-4">
                <div>
                  <label className="text-sm font-semibold mb-3 block">Image URL</label>
                  <Input
                    placeholder="https://example.com/image.jpg"
                    value={form.imageUrl || ''}
                    onChange={(e) => setForm(prev => ({ ...prev, imageUrl: e.target.value }))}
                    className="rounded-2xl border-2 border-indigo-200 dark:border-indigo-700 bg-gradient-to-r from-indigo-50/50 to-purple-50/50 dark:from-indigo-900/20 dark:to-purple-900/20 hover:border-indigo-300 dark:hover:border-indigo-600 transition-all duration-200"
                  />
                </div>
                <div>
                  <label className="text-sm font-semibold mb-3 block">Action URL</label>
                  <Input
                    placeholder="/dashboard or https://epsx.io/page"
                    value={form.actionUrl || ''}
                    onChange={(e) => setForm(prev => ({ ...prev, actionUrl: e.target.value }))}
                    className="rounded-2xl border-2 border-teal-200 dark:border-teal-700 bg-gradient-to-r from-teal-50/50 to-cyan-50/50 dark:from-teal-900/20 dark:to-cyan-900/20 hover:border-teal-300 dark:hover:border-teal-600 transition-all duration-200"
                  />
                </div>
                <div>
                  <label className="text-sm font-semibold mb-3 block">Custom Data (JSON)</label>
                  <Textarea
                    placeholder='{"custom_key": "custom_value"}'
                    rows={3}
                    value={form.customData || ''}
                    onChange={(e) => setForm(prev => ({ ...prev, customData: e.target.value }))}
                    className="rounded-2xl border-2 border-violet-200 dark:border-violet-700 bg-gradient-to-r from-violet-50/50 to-purple-50/50 dark:from-violet-900/20 dark:to-purple-900/20 hover:border-violet-300 dark:hover:border-violet-600 transition-all duration-200 resize-none font-mono text-sm"
                  />
                </div>
              </div>
            </details>

            <button 
              onClick={handleSendNotification}
              disabled={isPending || !form.title.trim() || !form.body.trim()}
              className="w-full px-6 py-4 bg-gradient-to-r from-yellow-500 via-orange-500 to-pink-500 hover:from-yellow-600 hover:via-orange-600 hover:to-pink-600 text-white font-semibold rounded-2xl transition-all duration-300 hover:scale-105 hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 flex items-center justify-center gap-2"
            >
              {isPending ? (
                <>
                  <div className="animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full"></div>
                  Sending...
                </>
              ) : (
                <>
                  <Send className="h-5 w-5" />
                  Send Notification
                </>
              )}
            </button>
              </div>
            </div>
          </div>
        </div>

        {/* Recent Notifications */}
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-pink-400/20 via-purple-400/20 to-blue-400/20 p-0.5 group">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl">
            {/* Floating decoration */}
            <div className="absolute top-4 right-4 w-6 h-6 bg-gradient-to-r from-pink-400/40 to-purple-500/40 rounded-full blur-sm animate-pulse"></div>
            
            <div className="p-8">
              <div className="mb-6">
                <h2 className="flex items-center gap-3 text-2xl font-bold bg-gradient-to-r from-pink-600 via-purple-600 to-blue-600 bg-clip-text text-transparent">
                  <div className="p-2 rounded-2xl bg-gradient-to-r from-pink-500 to-purple-500">
                    <BarChart3 className="h-6 w-6 text-white" />
                  </div>
                  Recent Notifications
                </h2>
                <p className="text-gray-600 dark:text-gray-400 mt-2 ml-14">
                  Latest notification activity and delivery status
                </p>
              </div>
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

            <div className="border-t border-gray-200/50 dark:border-gray-700/50 pt-6 mt-6 flex justify-between items-center">
              <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-purple-400/10 to-pink-400/10 hover:from-purple-400/20 hover:to-pink-400/20 border border-purple-200 dark:border-purple-700 rounded-2xl text-sm font-medium transition-all duration-200 hover:scale-105">
                <BarChart3 className="h-4 w-4" />
                View Analytics
              </button>
              <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-gray-400/10 to-gray-500/10 hover:from-gray-400/20 hover:to-gray-500/20 border border-gray-200 dark:border-gray-700 rounded-2xl text-sm font-medium transition-all duration-200 hover:scale-105">
                <Settings className="h-4 w-4" />
                Settings
              </button>
            </div>
            </div>
          </div>
        </div>
      </div>

      {/* Live Feed */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 via-blue-400/20 to-purple-400/20 p-0.5 group">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl">
          {/* Floating decoration */}
          <div className="absolute top-4 right-4 w-6 h-6 bg-gradient-to-r from-green-400/40 to-blue-500/40 rounded-full blur-sm animate-pulse"></div>
          
          <div className="p-8">
            <div className="mb-6">
              <h2 className="flex items-center gap-3 text-2xl font-bold bg-gradient-to-r from-green-600 via-blue-600 to-purple-600 bg-clip-text text-transparent">
                <div className="p-2 rounded-2xl bg-gradient-to-r from-green-500 to-blue-500">
                  <Zap className="h-6 w-6 text-white" />
                </div>
                🔔 Live Activity Feed
              </h2>
              <p className="text-gray-600 dark:text-gray-400 mt-2 ml-14">
                Real-time notification delivery events
              </p>
            </div>
          <div className="relative overflow-hidden bg-gradient-to-br from-gray-900 via-black to-gray-900 text-green-400 p-6 rounded-2xl font-mono text-sm border border-green-500/20">
            {/* Terminal-style decorations */}
            <div className="absolute top-2 left-2 flex gap-1">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <div className="w-2 h-2 bg-yellow-500 rounded-full"></div>
              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
            </div>
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400/30 to-emerald-500/30 rounded-full blur-sm animate-pulse"></div>
            
            <div className="space-y-2 mt-4">
              <div className="flex items-center gap-2">
                <span className="text-green-500">●</span>
                <span>16:45:23 → user123@domain.com: Notification delivered</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-blue-400">●</span>
                <span>16:45:22 → admin-users topic: Broadcast sent (12 recipients)</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-yellow-400">●</span>
                <span>16:45:19 → Token registered: c2HTa1OCBF...</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-purple-400">●</span>
                <span>16:45:15 → user456@domain.com: Notification clicked</span>
              </div>
              <div className="flex items-center gap-2 text-green-500 font-semibold pt-2">
                <span className="animate-pulse">■</span>
                <span>Ready for new notifications</span>
              </div>
            </div>
          </div>
          </div>
        </div>
      </div>
      </div>
    </div>
  )
}