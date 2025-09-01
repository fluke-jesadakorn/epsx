"use client"

import React from 'react'
import { Bell, AlertTriangle, Info, CheckCircle, Clock, Users, Shield, Settings } from 'lucide-react'

interface NotificationCardProps {
  notification: {
    id: string
    type: string
    title: string
    message: string
    is_read: boolean
    created_at: string
    metadata?: Record<string, any>
  }
  onMarkAsRead: (id: string) => void
  onDelete: (id: string) => void
}

function NotificationCard({ notification, onMarkAsRead, onDelete }: NotificationCardProps) {
  const getTypeInfo = (type: string) => {
    switch (type) {
      case 'security':
        return { icon: Shield, color: 'text-red-600', bgColor: 'bg-red-50 dark:bg-red-900/20', emoji: '🛡️' }
      case 'system':
        return { icon: Settings, color: 'text-blue-600', bgColor: 'bg-blue-50 dark:bg-blue-900/20', emoji: '⚙️' }
      case 'user':
        return { icon: Users, color: 'text-green-600', bgColor: 'bg-green-50 dark:bg-green-900/20', emoji: '👤' }
      case 'performance':
        return { icon: AlertTriangle, color: 'text-orange-600', bgColor: 'bg-orange-50 dark:bg-orange-900/20', emoji: '⚡' }
      default:
        return { icon: Info, color: 'text-gray-600', bgColor: 'bg-gray-50 dark:bg-gray-900/20', emoji: 'ℹ️' }
    }
  }

  const typeInfo = getTypeInfo(notification.type)
  const timeAgo = new Date(Date.now() - new Date(notification.created_at).getTime())
  const minutes = Math.floor(timeAgo.getTime() / (1000 * 60))
  const timeString = minutes < 60 ? `${minutes}min ago` : 
                     minutes < 1440 ? `${Math.floor(minutes / 60)}h ago` :
                     `${Math.floor(minutes / 1440)}d ago`

  return (
    <div className={`p-4 rounded-lg border transition-colors ${
      notification.is_read 
        ? 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700' 
        : 'bg-blue-50 dark:bg-blue-900/10 border-blue-200 dark:border-blue-700'
    } hover:border-blue-300 dark:hover:border-blue-600`}>
      <div className="flex items-start gap-3">
        {/* Type Icon */}
        <div className={`${typeInfo.bgColor} p-2 rounded-lg flex-shrink-0`}>
          <typeInfo.icon size={20} className={typeInfo.color} />
        </div>
        
        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <span className="text-lg">{typeInfo.emoji}</span>
                <h3 className={`font-medium ${notification.is_read ? 'text-gray-700 dark:text-gray-300' : 'text-gray-900 dark:text-white'}`}>
                  {notification.title}
                </h3>
                {!notification.is_read && (
                  <div className="w-2 h-2 bg-blue-500 rounded-full flex-shrink-0"></div>
                )}
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                {notification.message}
              </p>
              <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                <span className="flex items-center gap-1">
                  <Clock size={12} />
                  {timeString}
                </span>
                <span className="capitalize bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">
                  {notification.type}
                </span>
              </div>
            </div>
            
            {/* Actions */}
            <div className="flex items-center gap-2 flex-shrink-0">
              {!notification.is_read && (
                <button 
                  onClick={() => onMarkAsRead(notification.id)}
                  className="px-2 py-1 text-xs bg-blue-100 text-blue-800 hover:bg-blue-200 rounded transition-colors"
                >
                  ✓ Read
                </button>
              )}
              <button 
                onClick={() => onDelete(notification.id)}
                className="px-2 py-1 text-xs bg-red-100 text-red-800 hover:bg-red-200 rounded transition-colors"
              >
                🗑️ Delete
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default function InteractiveNotificationCard({ notification }: { notification: any }) {
  const handleMarkAsRead = (id: string) => {
    console.log('Mark as read:', id)
  }

  const handleDelete = (id: string) => {
    console.log('Delete notification:', id)
  }

  return (
    <NotificationCard
      notification={notification}
      onMarkAsRead={handleMarkAsRead}
      onDelete={handleDelete}
    />
  )
}