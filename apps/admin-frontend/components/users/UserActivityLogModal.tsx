'use client'

import React, { useState, useEffect } from 'react'
import { X, Activity, Clock, Shield, User, AlertCircle, CheckCircle, XCircle, Filter } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { getUserActivityLogs } from '@/lib/actions/users'
import type { ActivityLogEntry, ActivityLogParams } from '@/lib/actions/users'
import Pagination from '@/components/ui/Pagination'

interface UserActivityLogModalProps {
  isOpen: boolean
  onClose: () => void
  userId: string
  userName: string
}

const ActionIconMap: Record<string, typeof Activity> = {
  'login': User,
  'login_failed': XCircle,
  'logout': User,
  'user_created': CheckCircle,
  'user_updated': User,
  'user_deleted': XCircle,
  'permission_granted': Shield,
  'permission_revoked': XCircle,
  'role_assigned': Shield,
  'role_unassigned': XCircle,
  'default': Activity
}

const ResultColorMap: Record<string, string> = {
  'success': 'text-green-500',
  'failure': 'text-red-500',
  'error': 'text-red-500',
  'denied': 'text-orange-500',
  'partial_success': 'text-yellow-500'
}

const ResultBgMap: Record<string, string> = {
  'success': 'bg-green-100 dark:bg-green-900/20',
  'failure': 'bg-red-100 dark:bg-red-900/20',
  'error': 'bg-red-100 dark:bg-red-900/20',
  'denied': 'bg-orange-100 dark:bg-orange-900/20',
  'partial_success': 'bg-yellow-100 dark:bg-yellow-900/20'
}

export default function UserActivityLogModal({ isOpen, onClose, userId, userName }: UserActivityLogModalProps) {
  const [activities, setActivities] = useState<ActivityLogEntry[]>([])
  const [statistics, setStatistics] = useState<any>(null)
  const [pagination, setPagination] = useState({ limit: 20, offset: 0, total: 0 })
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [filter, setFilter] = useState<ActivityLogParams>({})
  const [expandedActivity, setExpandedActivity] = useState<string | null>(null)

  // Load activity logs
  const loadActivityLogs = async (params: ActivityLogParams = {}) => {
    setLoading(true)
    setError(null)

    try {
      const result = await getUserActivityLogs(userId, params)
      
      if (result.success) {
        setActivities(result.data.activities)
        setStatistics(result.data.statistics)
        setPagination(result.data.pagination)
      } else {
        setError(result.error.message)
      }
    } catch (err) {
      setError('Failed to load activity logs')
    } finally {
      setLoading(false)
    }
  }

  // Load initial data when modal opens
  useEffect(() => {
    if (isOpen) {
      loadActivityLogs({ limit: 20, offset: 0, ...filter })
    }
  }, [isOpen, filter])

  // Handle pagination
  const handlePageChange = (page: number) => {
    const newOffset = (page - 1) * pagination.limit
    loadActivityLogs({ 
      limit: pagination.limit, 
      offset: newOffset,
      ...filter 
    })
  }

  // Format action name for display
  const formatAction = (action: string) => {
    return action.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
  }

  // Get action icon
  const getActionIcon = (action: string) => {
    const IconComponent = ActionIconMap[action] || ActionIconMap.default
    return IconComponent
  }

  // Format timestamp
  const formatTimestamp = (timestamp: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: true
    }).format(timestamp)
  }

  const currentPage = Math.floor(pagination.offset / pagination.limit) + 1
  const totalPages = Math.ceil(pagination.total / pagination.limit)

  if (!isOpen) return null

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
        onClick={onClose}
      >
        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className="bg-white dark:bg-gray-900 border-2 border-[#FFC107] shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="bg-gradient-to-r from-[#FFC107] to-[#FF8F00] text-black px-6 py-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Activity className="w-6 h-6 text-black" />
              <div>
                <h2 className="text-lg font-bold">Activity Logs</h2>
                <p className="text-sm opacity-80">User: {userName}</p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-white/20 transition-colors border border-black/20"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Statistics Bar */}
          {statistics && (
            <div className="bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-3">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div className="flex items-center gap-2">
                  <Activity className="w-4 h-4 text-blue-500" />
                  <span className="text-gray-600 dark:text-gray-400">Total:</span>
                  <span className="font-medium">{statistics.total_activities}</span>
                </div>
                <div className="flex items-center gap-2">
                  <User className="w-4 h-4 text-green-500" />
                  <span className="text-gray-600 dark:text-gray-400">Logins:</span>
                  <span className="font-medium">{statistics.login_activities}</span>
                </div>
                <div className="flex items-center gap-2">
                  <XCircle className="w-4 h-4 text-red-500" />
                  <span className="text-gray-600 dark:text-gray-400">Failed:</span>
                  <span className="font-medium">{statistics.failed_activities}</span>
                </div>
                <div className="flex items-center gap-2">
                  <Clock className="w-4 h-4 text-yellow-500" />
                  <span className="text-gray-600 dark:text-gray-400">Recent:</span>
                  <span className="font-medium">{statistics.recent_activities}</span>
                </div>
              </div>
            </div>
          )}

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-6">
            {loading ? (
              <div className="space-y-4">
                {Array.from({ length: 5 }).map((_, i) => (
                  <div key={i} className="animate-pulse">
                    <div className="flex items-center gap-4">
                      <div className="w-10 h-10 bg-gray-300 dark:bg-gray-700"></div>
                      <div className="flex-1">
                        <div className="h-4 bg-gray-300 dark:bg-gray-700 w-1/3 mb-2"></div>
                        <div className="h-3 bg-gray-200 dark:bg-gray-600 w-1/2"></div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : error ? (
              <div className="text-center py-12">
                <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-red-700 dark:text-red-400 mb-2">Error Loading Activity Logs</h3>
                <p className="text-gray-600 dark:text-gray-400 mb-4">{error}</p>
                <button
                  onClick={() => loadActivityLogs({ limit: 20, offset: 0, ...filter })}
                  className="px-4 py-2 bg-[#FFC107] text-black font-medium hover:bg-[#FFB300] transition-colors"
                >
                  Retry
                </button>
              </div>
            ) : activities.length === 0 ? (
              <div className="text-center py-12">
                <Activity className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-700 dark:text-gray-300 mb-2">No Activity Logs</h3>
                <p className="text-gray-600 dark:text-gray-400">This user has no recorded activity yet.</p>
              </div>
            ) : (
              <div className="space-y-3">
                {activities.map((activity) => {
                  const IconComponent = getActionIcon(activity.action)
                  const isExpanded = expandedActivity === activity.id
                  
                  return (
                    <motion.div
                      key={activity.id}
                      className={`border-l-4 border-[#FFC107] bg-white dark:bg-gray-800 shadow-lg transition-all hover:shadow-xl cursor-pointer ${ResultBgMap[activity.result] || 'bg-gray-50 dark:bg-gray-800'}`}
                      onClick={() => setExpandedActivity(isExpanded ? null : activity.id)}
                      whileHover={{ scale: 1.01 }}
                      whileTap={{ scale: 0.99 }}
                    >
                      <div className="p-4">
                        <div className="flex items-start gap-4">
                          <div className={`w-10 h-10 bg-gradient-to-br from-[#FFC107] to-[#FF8F00] text-black flex items-center justify-center`}>
                            <IconComponent className="w-5 h-5" />
                          </div>
                          
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center justify-between mb-1">
                              <h4 className="font-medium text-gray-900 dark:text-white">
                                {formatAction(activity.action)}
                              </h4>
                              <span className={`text-xs font-medium px-2 py-1 rounded ${ResultColorMap[activity.result]} ${ResultBgMap[activity.result]}`}>
                                {activity.result.toUpperCase()}
                              </span>
                            </div>
                            
                            <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400 mb-2">
                              <div className="flex items-center gap-1">
                                <Clock className="w-3 h-3" />
                                {formatTimestamp(activity.timestamp)}
                              </div>
                              <div>
                                Resource: {activity.resource_type}
                              </div>
                              {activity.client_ip && (
                                <div>
                                  IP: {activity.client_ip}
                                </div>
                              )}
                            </div>

                            {/* Expanded Details */}
                            <AnimatePresence>
                              {isExpanded && (
                                <motion.div
                                  initial={{ opacity: 0, height: 0 }}
                                  animate={{ opacity: 1, height: 'auto' }}
                                  exit={{ opacity: 0, height: 0 }}
                                  className="border-t border-gray-200 dark:border-gray-600 pt-3 mt-3"
                                >
                                  <div className="grid md:grid-cols-2 gap-4 text-sm">
                                    <div>
                                      <strong>Resource ID:</strong> {activity.resource_id}
                                    </div>
                                    {activity.session_id && (
                                      <div>
                                        <strong>Session:</strong> {activity.session_id.substring(0, 8)}...
                                      </div>
                                    )}
                                    {activity.user_agent && (
                                      <div className="col-span-2">
                                        <strong>User Agent:</strong> 
                                        <span className="text-xs text-gray-500 dark:text-gray-400 ml-2 break-all">
                                          {activity.user_agent.substring(0, 100)}...
                                        </span>
                                      </div>
                                    )}
                                    {activity.metadata.error_message && (
                                      <div className="col-span-2 p-2 bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
                                        <strong className="text-red-700 dark:text-red-400">Error:</strong>
                                        <div className="text-red-600 dark:text-red-300 mt-1">{activity.metadata.error_message}</div>
                                      </div>
                                    )}
                                    {activity.metadata.duration_ms && (
                                      <div>
                                        <strong>Duration:</strong> {activity.metadata.duration_ms}ms
                                      </div>
                                    )}
                                  </div>
                                </motion.div>
                              )}
                            </AnimatePresence>
                          </div>
                        </div>
                      </div>
                    </motion.div>
                  )
                })}
              </div>
            )}
          </div>

          {/* Pagination Footer */}
          {totalPages > 1 && (
            <div className="border-t border-gray-200 dark:border-gray-700">
              <Pagination
                currentPage={currentPage}
                totalPages={totalPages}
                onPageChange={handlePageChange}
                disabled={loading}
              />
            </div>
          )}
        </motion.div>
      </motion.div>
    </AnimatePresence>
  )
}