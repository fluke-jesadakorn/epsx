'use client'

import React, { useState, useEffect } from 'react'
import { 
  Send, 
  Users, 
  MessageSquare, 
  Settings, 
  Eye, 
  TestTube,
  Zap,
  Bell,
  AlertTriangle,
  CheckCircle,
  Target,
  PlayCircle
} from 'lucide-react'
// Using client-side API with server-side types imported separately
import { SystemMessage, NotificationTemplate, TestMessageRequest } from '@/lib/api/notification-client'

interface PushMessageManagerProps {
  onMessageSent?: (result: any) => void
}

export default function PushMessageManager({ onMessageSent }: PushMessageManagerProps) {
  const [activeTab, setActiveTab] = useState<'compose' | 'templates' | 'test' | 'bulk'>('compose')
  const [templates, setTemplates] = useState<NotificationTemplate[]>([])
  const [selectedTemplate, setSelectedTemplate] = useState<NotificationTemplate | null>(null)
  const [loading, setLoading] = useState(false)
  const [previewHtml, setPreviewHtml] = useState('')

  // Form states
  const [composeForm, setComposeForm] = useState({
    title: '',
    body: '',
    priority: 'normal' as 'normal' | 'high',
    data: {} as Record<string, string>,
    image_url: '',
    target: 'broadcast' as 'broadcast' | 'user' | 'test',
    user_id: '',
    test_user_ids: [''],
  })

  const [bulkForm, setBulkForm] = useState({
    template_id: '',
    variables: {} as Record<string, string>,
    target: 'all' as 'all' | 'active_users',
    priority: 'normal' as 'normal' | 'high',
  })

  // Load templates on mount
  useEffect(() => {
    loadTemplates()
  }, [])

  const loadTemplates = async () => {
    try {
      const response = await ServerNotificationAPI.getTemplates()
      setTemplates(response.templates)
    } catch (error) {
      console.error('Failed to load templates:', error)
    }
  }

  const handlePreviewTemplate = async (template: NotificationTemplate, variables: Record<string, string>) => {
    try {
      const preview = await ServerNotificationAPI.renderTemplate(template.id, variables)
      setPreviewHtml(preview.preview_html)
    } catch (error) {
      console.error('Failed to preview template:', error)
    }
  }

  const handleSendMessage = async () => {
    setLoading(true)
    try {
      let result
      if (composeForm.target === 'broadcast') {
        result = await ServerNotificationAPI.sendBroadcast({
          title: composeForm.title,
          body: composeForm.body,
          priority: composeForm.priority,
          data: Object.keys(composeForm.data).length > 0 ? composeForm.data : undefined,
          image_url: composeForm.image_url || undefined,
        })
      } else if (composeForm.target === 'user') {
        result = await ServerNotificationAPI.sendToUser(composeForm.user_id, {
          title: composeForm.title,
          body: composeForm.body,
          priority: composeForm.priority,
          data: Object.keys(composeForm.data).length > 0 ? composeForm.data : undefined,
          image_url: composeForm.image_url || undefined,
        })
      }

      if (result && onMessageSent) {
        onMessageSent(result)
      }
    } catch (error) {
      console.error('Failed to send message:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleTestMessage = async () => {
    setLoading(true)
    try {
      const testRequest: TestMessageRequest = {
        title: composeForm.title,
        body: composeForm.body,
        test_user_ids: composeForm.test_user_ids.filter(id => id.trim()),
        priority: composeForm.priority,
        data: Object.keys(composeForm.data).length > 0 ? composeForm.data : undefined,
      }

      const result = await ServerNotificationAPI.testMessage(testRequest)
      if (onMessageSent) {
        onMessageSent(result)
      }
    } catch (error) {
      console.error('Failed to test message:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleBulkSend = async () => {
    setLoading(true)
    try {
      const result = await ServerNotificationAPI.sendBulkMessage({
        template_id: bulkForm.template_id,
        variables: bulkForm.variables,
        target: bulkForm.target,
        priority: bulkForm.priority,
      })

      if (onMessageSent) {
        onMessageSent(result)
      }
    } catch (error) {
      console.error('Failed to send bulk message:', error)
    } finally {
      setLoading(false)
    }
  }

  const addDataField = () => {
    const key = `field_${Date.now()}`
    setComposeForm(prev => ({
      ...prev,
      data: { ...prev.data, [key]: '' }
    }))
  }

  const updateDataField = (oldKey: string, newKey: string, value: string) => {
    setComposeForm(prev => {
      const newData = { ...prev.data }
      delete newData[oldKey]
      newData[newKey] = value
      return { ...prev, data: newData }
    })
  }

  const removeDataField = (key: string) => {
    setComposeForm(prev => {
      const newData = { ...prev.data }
      delete newData[key]
      return { ...prev, data: newData }
    })
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="p-6 border-b border-gray-200 dark:border-gray-700">
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white flex items-center gap-3">
          <Send size={24} className="text-blue-600" />
          🚀 Push Message Manager
        </h3>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Create, test, and send push notifications to users
        </p>
      </div>

      {/* Tab Navigation */}
      <div className="border-b border-gray-200 dark:border-gray-700">
        <div className="flex overflow-x-auto">
          {[
            { key: 'compose', label: 'Compose', icon: MessageSquare },
            { key: 'templates', label: 'Templates', icon: Settings },
            { key: 'test', label: 'Test', icon: TestTube },
            { key: 'bulk', label: 'Bulk Send', icon: Users },
          ].map(({ key, label, icon: Icon }) => (
            <button
              key={key}
              onClick={() => setActiveTab(key as any)}
              className={`px-6 py-4 font-medium text-sm whitespace-nowrap border-b-2 transition-colors flex items-center gap-2 ${
                activeTab === key
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              <Icon size={16} />
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="p-6">
        {/* Compose Tab */}
        {activeTab === 'compose' && (
          <div className="space-y-6">
            {/* Target Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Target Audience
              </label>
              <div className="grid grid-cols-3 gap-3">
                {[
                  { key: 'broadcast', label: 'Broadcast All', icon: '📢', desc: 'Send to all users' },
                  { key: 'user', label: 'Specific User', icon: '👤', desc: 'Send to one user' },
                  { key: 'test', label: 'Test Group', icon: '🧪', desc: 'Test with multiple users' },
                ].map(({ key, label, icon, desc }) => (
                  <label key={key} className="relative">
                    <input
                      type="radio"
                      name="target"
                      value={key}
                      checked={composeForm.target === key}
                      onChange={(e) => setComposeForm(prev => ({ ...prev, target: e.target.value as any }))}
                      className="sr-only"
                    />
                    <div className={`p-4 border-2 rounded-lg cursor-pointer transition-all ${
                      composeForm.target === key
                        ? 'border-blue-600 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                    }`}>
                      <div className="text-center">
                        <div className="text-2xl mb-2">{icon}</div>
                        <div className="font-medium text-gray-900 dark:text-white">{label}</div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">{desc}</div>
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            </div>

            {/* User ID Input (for specific user) */}
            {composeForm.target === 'user' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  User ID
                </label>
                <input
                  type="text"
                  value={composeForm.user_id}
                  onChange={(e) => setComposeForm(prev => ({ ...prev, user_id: e.target.value }))}
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="Enter user ID"
                />
              </div>
            )}

            {/* Test User IDs (for test group) */}
            {composeForm.target === 'test' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Test User IDs
                </label>
                <div className="space-y-2">
                  {composeForm.test_user_ids.map((userId, index) => (
                    <div key={index} className="flex gap-2">
                      <input
                        type="text"
                        value={userId}
                        onChange={(e) => {
                          const newIds = [...composeForm.test_user_ids]
                          newIds[index] = e.target.value
                          setComposeForm(prev => ({ ...prev, test_user_ids: newIds }))
                        }}
                        className="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        placeholder="Enter user ID"
                      />
                      <button
                        onClick={() => {
                          const newIds = composeForm.test_user_ids.filter((_, i) => i !== index)
                          setComposeForm(prev => ({ ...prev, test_user_ids: newIds }))
                        }}
                        className="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                  <button
                    onClick={() => setComposeForm(prev => ({ ...prev, test_user_ids: [...prev.test_user_ids, ''] }))}
                    className="text-blue-600 hover:text-blue-700 text-sm font-medium"
                  >
                    + Add User ID
                  </button>
                </div>
              </div>
            )}

            {/* Message Content */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Title *
                </label>
                <input
                  type="text"
                  value={composeForm.title}
                  onChange={(e) => setComposeForm(prev => ({ ...prev, title: e.target.value }))}
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="Notification title"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Priority
                </label>
                <select
                  value={composeForm.priority}
                  onChange={(e) => setComposeForm(prev => ({ ...prev, priority: e.target.value as 'normal' | 'high' }))}
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                >
                  <option value="normal">🔔 Normal</option>
                  <option value="high">⚡ High</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Message Body *
              </label>
              <textarea
                value={composeForm.body}
                onChange={(e) => setComposeForm(prev => ({ ...prev, body: e.target.value }))}
                rows={4}
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white resize-none"
                placeholder="Enter your notification message..."
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Image URL (Optional)
              </label>
              <input
                type="url"
                value={composeForm.image_url}
                onChange={(e) => setComposeForm(prev => ({ ...prev, image_url: e.target.value }))}
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="https://example.com/image.jpg"
              />
            </div>

            {/* Custom Data Fields */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Custom Data (Key-Value Pairs)
              </label>
              <div className="space-y-2">
                {Object.entries(composeForm.data).map(([key, value]) => (
                  <div key={key} className="flex gap-2">
                    <input
                      type="text"
                      value={key}
                      onChange={(e) => updateDataField(key, e.target.value, value)}
                      className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                      placeholder="Key"
                    />
                    <input
                      type="text"
                      value={value}
                      onChange={(e) => updateDataField(key, key, e.target.value)}
                      className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                      placeholder="Value"
                    />
                    <button
                      onClick={() => removeDataField(key)}
                      className="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"
                    >
                      ✕
                    </button>
                  </div>
                ))}
                <button
                  onClick={addDataField}
                  className="text-blue-600 hover:text-blue-700 text-sm font-medium"
                >
                  + Add Data Field
                </button>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex gap-3 pt-4">
              {composeForm.target === 'test' ? (
                <button
                  onClick={handleTestMessage}
                  disabled={loading || !composeForm.title || !composeForm.body}
                  className="flex-1 bg-orange-600 hover:bg-orange-700 disabled:opacity-50 text-white px-6 py-3 rounded-lg font-medium flex items-center justify-center gap-2 transition-colors"
                >
                  <TestTube size={20} />
                  {loading ? 'Testing...' : 'Send Test Message'}
                </button>
              ) : (
                <button
                  onClick={handleSendMessage}
                  disabled={loading || !composeForm.title || !composeForm.body}
                  className="flex-1 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white px-6 py-3 rounded-lg font-medium flex items-center justify-center gap-2 transition-colors"
                >
                  <Send size={20} />
                  {loading ? 'Sending...' : 'Send Message'}
                </button>
              )}
              
              <button
                onClick={() => {
                  // Preview functionality would go here
                  alert('Preview functionality coming soon!')
                }}
                className="px-6 py-3 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg font-medium flex items-center gap-2 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
              >
                <Eye size={20} />
                Preview
              </button>
            </div>
          </div>
        )}

        {/* Templates Tab */}
        {activeTab === 'templates' && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <h4 className="text-lg font-medium text-gray-900 dark:text-white">Available Templates</h4>
              <button className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium flex items-center gap-2">
                <Settings size={16} />
                Manage Templates
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {templates.map((template) => (
                <div key={template.id} className="border border-gray-200 dark:border-gray-600 rounded-lg p-4">
                  <div className="flex items-start justify-between mb-3">
                    <div>
                      <h5 className="font-medium text-gray-900 dark:text-white">{template.name}</h5>
                      <p className="text-sm text-gray-500 dark:text-gray-400">{template.category}</p>
                    </div>
                    <div className={`px-2 py-1 rounded-full text-xs font-medium ${
                      template.priority === 'high' 
                        ? 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300'
                        : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
                    }`}>
                      {template.priority === 'high' ? '⚡ High' : '🔔 Normal'}
                    </div>
                  </div>
                  
                  <div className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                    <p className="font-medium mb-1">Title: {template.title_template}</p>
                    <p className="line-clamp-2">{template.body_template}</p>
                  </div>

                  <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 mb-3">
                    <span>Used {template.usage_count} times</span>
                    <span className={`px-2 py-1 rounded ${
                      template.enabled
                        ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300'
                        : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
                    }`}>
                      {template.enabled ? '✅ Active' : '⏸️ Disabled'}
                    </span>
                  </div>

                  <div className="flex gap-2">
                    <button
                      onClick={() => setSelectedTemplate(template)}
                      className="flex-1 bg-blue-600 hover:bg-blue-700 text-white px-3 py-2 rounded text-sm font-medium flex items-center justify-center gap-1"
                    >
                      <PlayCircle size={14} />
                      Use Template
                    </button>
                    <button className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700">
                      <Eye size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Other tabs would be implemented similarly */}
        {activeTab === 'test' && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            <TestTube size={48} className="mx-auto mb-4" />
            <h4 className="text-lg font-medium mb-2">Message Testing</h4>
            <p>Use the compose tab with "Test Group" selected to test messages with specific users.</p>
          </div>
        )}

        {activeTab === 'bulk' && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            <Users size={48} className="mx-auto mb-4" />
            <h4 className="text-lg font-medium mb-2">Bulk Messaging</h4>
            <p>Template-based bulk messaging functionality coming soon.</p>
          </div>
        )}
      </div>
    </div>
  )
}