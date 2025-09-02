"use client"

import React, { useState } from 'react'
import { Bell, AlertTriangle, CheckCircle, Shield, Smartphone, Settings } from 'lucide-react'
import { adminFCMClient } from '@/lib/admin-fcm-client'
import { Button } from '@/components/ui/button'
import { useToast } from '@/components/ui/use-toast'

export default function NotificationActions() {
  const [loading, setLoading] = useState(false)
  const { toast } = useToast()

  const handleMarkAllAsRead = async () => {
    setLoading(true)
    try {
      // Call API to mark all notifications as read
      const response = await fetch('/api/v1/admin/notifications/mark-all-read', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        }
      })

      if (response.ok) {
        toast({
          title: "✅ Success",
          description: "All notifications marked as read"
        })
        // Refresh the page to update the notification list
        window.location.reload()
      } else {
        throw new Error('Failed to mark notifications as read')
      }
    } catch (error) {
      toast({
        title: "❌ Error",
        description: "Failed to mark all notifications as read",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleSendTestNotification = async () => {
    setLoading(true)
    try {
      const testNotification = {
        title: 'Admin Test Notification',
        body: 'This is a test notification from EPSX admin interface',
        adminType: 'system_alert' as const,
        priority: 'normal' as const,
        url: '/notifications'
      }

      await adminFCMClient.sendTestNotification(testNotification)
      
      toast({
        title: "🔔 Test Sent",
        description: "Admin test notification sent successfully"
      })
    } catch (error) {
      console.error('Error sending test notification:', error)
      toast({
        title: "❌ Error", 
        description: "Failed to send test notification",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleClearAll = async () => {
    if (!confirm('Are you sure you want to clear all notifications? This action cannot be undone.')) {
      return
    }

    setLoading(true)
    try {
      const response = await fetch('/api/v1/admin/notifications/clear-all', {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        }
      })

      if (response.ok) {
        toast({
          title: "🗑️ Cleared",
          description: "All notifications cleared successfully"
        })
        // Refresh the page to update the notification list
        window.location.reload()
      } else {
        throw new Error('Failed to clear notifications')
      }
    } catch (error) {
      toast({
        title: "❌ Error",
        description: "Failed to clear all notifications",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleToggleFCMSubscription = async () => {
    setLoading(true)
    try {
      const status = await adminFCMClient.getSubscriptionStatus()
      
      if (status.isSubscribed) {
        await adminFCMClient.unsubscribe()
        toast({
          title: "🔕 Unsubscribed",
          description: "Unsubscribed from admin push notifications"
        })
      } else {
        await adminFCMClient.subscribe()
        toast({
          title: "🔔 Subscribed", 
          description: "Subscribed to admin push notifications"
        })
      }
    } catch (error) {
      console.error('Error toggling FCM subscription:', error)
      toast({
        title: "❌ Error",
        description: "Failed to toggle FCM subscription",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex flex-wrap gap-2">
      <Button 
        onClick={handleMarkAllAsRead}
        disabled={loading}
        className="bg-blue-600 hover:bg-blue-700 text-white"
      >
        <CheckCircle size={16} className="mr-2" />
        Mark All Read
      </Button>

      <Button 
        onClick={handleSendTestNotification}
        disabled={loading}
        className="bg-green-600 hover:bg-green-700 text-white"
      >
        <Bell size={16} className="mr-2" />
        Send Test FCM
      </Button>

      <Button 
        onClick={handleToggleFCMSubscription}
        disabled={loading}
        variant="outline"
        className="border-yellow-500 hover:bg-yellow-50 dark:hover:bg-yellow-950"
      >
        <Smartphone size={16} className="mr-2" />
        Toggle FCM
      </Button>

      <Button 
        onClick={handleClearAll}
        disabled={loading}
        variant="destructive"
      >
        <AlertTriangle size={16} className="mr-2" />
        Clear All
      </Button>

      <Button 
        onClick={() => window.location.href = '/fcm-test'}
        variant="outline"
        className="border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-950"
      >
        <Shield size={16} className="mr-2" />
        FCM Settings
      </Button>
    </div>
  )
}