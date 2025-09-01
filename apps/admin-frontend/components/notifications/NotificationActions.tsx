"use client"

import React from 'react'
import { Bell, AlertTriangle, CheckCircle } from 'lucide-react'

export default function NotificationActions() {
  const handleMarkAllAsRead = () => {
    console.log('Mark all as read')
  }

  const handleClearAll = () => {
    console.log('Clear all notifications')
  }

  return (
    <div className="flex gap-2">
      <button 
        onClick={handleMarkAllAsRead}
        className="flex items-center gap-2 px-4 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
      >
        <CheckCircle size={20} />
        ✓ Mark All Read
      </button>
      <button className="flex items-center gap-2 px-4 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors">
        <Bell size={20} />
        📧 Send Test
      </button>
      <button 
        onClick={handleClearAll}
        className="flex items-center gap-2 px-4 py-3 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
      >
        <AlertTriangle size={20} />
        🗑️ Clear All
      </button>
    </div>
  )
}