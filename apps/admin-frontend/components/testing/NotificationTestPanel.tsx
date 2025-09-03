'use client'

import React, { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Send, TestTube, Users, Mail } from 'lucide-react'
import { sendNotificationToUser, sendBroadcastNotification } from '@/lib/actions/notification-actions'

export function NotificationTestPanel() {
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<any>(null)
  const [error, setError] = useState<string | null>(null)
  
  const [testForm, setTestForm] = useState({
    type: 'user' as 'user' | 'broadcast',
    userEmail: 'info@epsx.io',
    title: 'Test Notification from Admin',
    body: 'This is a test notification sent from the admin interface to verify FCM integration is working properly.',
    priority: 'normal' as 'normal' | 'high'
  })

  const handleSendTest = async () => {
    setLoading(true)
    setError(null)
    setResult(null)

    try {
      let response
      if (testForm.type === 'user') {
        response = await sendNotificationToUser(
          testForm.userEmail,
          testForm.title,
          testForm.body,
          testForm.priority
        )
      } else {
        response = await sendBroadcastNotification(
          testForm.title,
          testForm.body,
          testForm.priority
        )
      }

      setResult(response)
      if (response.success) {
        console.log('✅ Notification sent successfully:', response)
      } else {
        setError(response.error || 'Unknown error occurred')
      }
    } catch (err) {
      console.error('❌ Failed to send notification:', err)
      setError(err instanceof Error ? err.message : 'Failed to send notification')
    } finally {
      setLoading(false)
    }
  }

  return (
    <Card className="max-w-2xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <TestTube size={20} className="text-blue-600" />
          Notification Test Panel
        </CardTitle>
        <CardDescription>
          Test sending notifications from admin frontend to main frontend (info@epsx.io)
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Notification Type */}
        <div className="space-y-2">
          <Label>Notification Type</Label>
          <Select 
            value={testForm.type} 
            onValueChange={(value: 'user' | 'broadcast') => setTestForm(prev => ({ ...prev, type: value }))}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="user">
                <div className="flex items-center gap-2">
                  <Mail size={16} />
                  Send to Specific User
                </div>
              </SelectItem>
              <SelectItem value="broadcast">
                <div className="flex items-center gap-2">
                  <Users size={16} />
                  Broadcast to All Users
                </div>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* User Email (only for user type) */}
        {testForm.type === 'user' && (
          <div className="space-y-2">
            <Label htmlFor="userEmail">User Email</Label>
            <Input
              id="userEmail"
              type="email"
              value={testForm.userEmail}
              onChange={(e) => setTestForm(prev => ({ ...prev, userEmail: e.target.value }))}
              placeholder="info@epsx.io"
            />
          </div>
        )}

        {/* Title */}
        <div className="space-y-2">
          <Label htmlFor="title">Title</Label>
          <Input
            id="title"
            value={testForm.title}
            onChange={(e) => setTestForm(prev => ({ ...prev, title: e.target.value }))}
            placeholder="Notification title"
          />
        </div>

        {/* Body */}
        <div className="space-y-2">
          <Label htmlFor="body">Message</Label>
          <Textarea
            id="body"
            value={testForm.body}
            onChange={(e) => setTestForm(prev => ({ ...prev, body: e.target.value }))}
            placeholder="Notification message"
            rows={4}
          />
        </div>

        {/* Priority */}
        <div className="space-y-2">
          <Label>Priority</Label>
          <Select 
            value={testForm.priority} 
            onValueChange={(value: 'normal' | 'high') => setTestForm(prev => ({ ...prev, priority: value }))}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="normal">🔔 Normal</SelectItem>
              <SelectItem value="high">⚡ High Priority</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Send Button */}
        <Button
          onClick={handleSendTest}
          disabled={loading || !testForm.title || !testForm.body || (testForm.type === 'user' && !testForm.userEmail)}
          className="w-full"
          size="lg"
        >
          <Send size={16} className="mr-2" />
          {loading ? 'Sending...' : `Send ${testForm.type === 'user' ? 'to User' : 'Broadcast'}`}
        </Button>

        {/* Results */}
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {result && result.success && (
          <Alert>
            <AlertDescription>
              ✅ Notification sent successfully! 
              {result.data && (
                <div className="mt-2 text-sm">
                  <pre className="bg-gray-100 dark:bg-gray-800 p-2 rounded text-xs overflow-auto">
                    {JSON.stringify(result.data, null, 2)}
                  </pre>
                </div>
              )}
            </AlertDescription>
          </Alert>
        )}

        {result && !result.success && (
          <Alert variant="destructive">
            <AlertDescription>
              ❌ Failed to send notification: {result.error}
            </AlertDescription>
          </Alert>
        )}

        {/* Instructions */}
        <div className="p-4 bg-blue-50 dark:bg-blue-950/20 rounded-lg">
          <h4 className="font-medium text-blue-900 dark:text-blue-100 mb-2">Testing Instructions:</h4>
          <ol className="text-sm text-blue-700 dark:text-blue-300 space-y-1">
            <li>1. Make sure both admin frontend (port 3001) and main frontend (port 3000) are running</li>
            <li>2. Ensure the user info@epsx.io exists in the system</li>
            <li>3. Have the main frontend open in another tab/window</li>
            <li>4. Click "Send to User" to test notification delivery</li>
            <li>5. Check the main frontend for push notifications or in-app notifications</li>
          </ol>
        </div>
      </CardContent>
    </Card>
  )
}

export default NotificationTestPanel