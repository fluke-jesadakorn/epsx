'use client'

import React, { useState, useEffect } from 'react'
import { 
  Settings, 
  BarChart3, 
  FileText, 
  TrendingUp, 
  Clock, 
  CheckCircle,
  AlertCircle,
  Users,
  MessageSquare,
  Activity,
  Bell,
  Shield
} from 'lucide-react'
// Using client-side API with server-side types imported separately
import { SystemStats, NotificationTemplate } from '@/lib/api/notification-client'

interface SystemSettingsDashboardProps {
  onStatsUpdate?: (stats: any) => void
}

export default function SystemSettingsDashboard({ onStatsUpdate }: SystemSettingsDashboardProps) {
  const [stats, setStats] = useState<SystemStats | null>(null)
  const [templateStats, setTemplateStats] = useState<any>(null)
  const [systemMessages, setSystemMessages] = useState<any[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadSystemData()
  }, [])

  const loadSystemData = async () => {
    setLoading(true)
    try {
      const [systemStats, templates, messages] = await Promise.all([
        ServerNotificationAPI.getSystemStats(),
        ServerNotificationAPI.getTemplateStats(),
        ServerNotificationAPI.getSystemMessages(),
      ])

      setStats(systemStats)
      setTemplateStats(templates)
      setSystemMessages(messages.messages)

      if (onStatsUpdate) {
        onStatsUpdate({ system: systemStats, templates: templates })
      }
    } catch (error) {
      console.error('Failed to load system data:', error)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="animate-pulse">
          <div className="h-6 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {[1, 2, 3].map(i => (
              <div key={i} className="h-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white flex items-center gap-3 mb-2">
          <Settings size={24} className="text-purple-600" />
          ⚙️ System Settings Dashboard
        </h3>
        <p className="text-gray-600 dark:text-gray-400">
          Monitor notification system performance and manage global settings
        </p>
      </div>

      {/* Quick Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 text-white p-6 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-sm font-medium opacity-90">Active Messages</h4>
              <p className="text-3xl font-bold">{stats?.active_messages || 0}</p>
              <p className="text-xs opacity-75">of {stats?.total_messages || 0} total</p>
            </div>
            <MessageSquare size={32} className="opacity-80" />
          </div>
        </div>

        <div className="bg-gradient-to-r from-green-500 to-green-600 text-white p-6 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-sm font-medium opacity-90">Templates</h4>
              <p className="text-3xl font-bold">{templateStats?.active_templates || 0}</p>
              <p className="text-xs opacity-75">of {templateStats?.total_templates || 0} total</p>
            </div>
            <FileText size={32} className="opacity-80" />
          </div>
        </div>

        <div className="bg-gradient-to-r from-orange-500 to-orange-600 text-white p-6 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-sm font-medium opacity-90">Today</h4>
              <p className="text-3xl font-bold">{stats?.messages_sent_today || 0}</p>
              <p className="text-xs opacity-75">messages sent</p>
            </div>
            <Activity size={32} className="opacity-80" />
          </div>
        </div>

        <div className="bg-gradient-to-r from-purple-500 to-purple-600 text-white p-6 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-sm font-medium opacity-90">This Week</h4>
              <p className="text-3xl font-bold">{stats?.messages_sent_this_week || 0}</p>
              <p className="text-xs opacity-75">total volume</p>
            </div>
            <TrendingUp size={32} className="opacity-80" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* System Messages Status */}
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-6 border-b border-gray-200 dark:border-gray-700">
            <h4 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <Bell size={20} />
              System Messages
            </h4>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {systemMessages.slice(0, 5).map((message, index) => (
                <div key={message.id || index} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                  <div className="flex items-center gap-3">
                    <div className={`w-2 h-2 rounded-full ${
                      message.enabled ? 'bg-green-500' : 'bg-gray-400'
                    }`}></div>
                    <div>
                      <p className="font-medium text-gray-900 dark:text-white">{message.name}</p>
                      <p className="text-sm text-gray-500 dark:text-gray-400">{message.category}</p>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`px-2 py-1 rounded-full text-xs font-medium ${
                      message.enabled
                        ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300'
                        : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
                    }`}>
                      {message.enabled ? 'Active' : 'Disabled'}
                    </div>
                  </div>
                </div>
              ))}
              
              {systemMessages.length === 0 && (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  <MessageSquare size={32} className="mx-auto mb-2 opacity-50" />
                  <p>No system messages configured</p>
                </div>
              )}
            </div>
            
            <button className="w-full mt-4 py-2 text-blue-600 hover:text-blue-700 font-medium text-sm">
              Manage System Messages →
            </button>
          </div>
        </div>

        {/* Template Usage Analytics */}
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-6 border-b border-gray-200 dark:border-gray-700">
            <h4 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <BarChart3 size={20} />
              Template Analytics
            </h4>
          </div>
          <div className="p-6">
            {templateStats?.usage_stats && templateStats.usage_stats.length > 0 ? (
              <div className="space-y-4">
                {templateStats.usage_stats.slice(0, 5).map((stat: any) => (
                  <div key={stat.template_id} className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-gray-900 dark:text-white">{stat.template_name}</p>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {stat.success_rate.toFixed(1)}% success rate
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="font-bold text-gray-900 dark:text-white">{stat.usage_count}</p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">uses</p>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                <FileText size={32} className="mx-auto mb-2 opacity-50" />
                <p>No template usage data</p>
              </div>
            )}
            
            <button className="w-full mt-4 py-2 text-purple-600 hover:text-purple-700 font-medium text-sm">
              View Full Analytics →
            </button>
          </div>
        </div>
      </div>

      {/* Category Breakdown */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <h4 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <Shield size={20} />
            Category Breakdown
          </h4>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {templateStats?.templates_by_category ? 
              Object.entries(templateStats.templates_by_category).map(([category, count]) => (
                <div key={category} className="text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">{count as number}</div>
                  <div className="text-sm text-gray-500 dark:text-gray-400 capitalize">{category}</div>
                </div>
              )) :
              <div className="col-span-4 text-center py-8 text-gray-500 dark:text-gray-400">
                <BarChart3 size={32} className="mx-auto mb-2 opacity-50" />
                <p>No category data available</p>
              </div>
            }
          </div>
        </div>
      </div>

      {/* Most Used Template */}
      {stats?.most_used_template && (
        <div className="bg-gradient-to-r from-indigo-500 to-purple-600 text-white p-6 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-lg font-semibold mb-2">🏆 Most Popular Template</h4>
              <p className="text-xl font-bold">{stats.most_used_template}</p>
              <p className="text-sm opacity-90">
                Most frequently used in {stats.most_used_category} category
              </p>
            </div>
            <div className="text-right">
              <div className="text-3xl font-bold">#{1}</div>
              <div className="text-sm opacity-90">Rank</div>
            </div>
          </div>
        </div>
      )}

      {/* Quick Actions */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Actions</h4>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {[
            { label: 'Create Template', icon: '📝', color: 'blue' },
            { label: 'System Message', icon: '⚙️', color: 'green' },
            { label: 'View Analytics', icon: '📊', color: 'purple' },
            { label: 'Export Data', icon: '📤', color: 'orange' },
          ].map(({ label, icon, color }) => (
            <button
              key={label}
              className={`p-4 rounded-lg border-2 border-${color}-200 hover:border-${color}-400 hover:bg-${color}-50 dark:hover:bg-${color}-900/20 transition-colors text-center group`}
            >
              <div className="text-2xl mb-2">{icon}</div>
              <div className="text-sm font-medium text-gray-700 dark:text-gray-300 group-hover:text-gray-900 dark:group-hover:text-white">
                {label}
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}