'use client'

import { useState, useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import { 
  X, Shield, Clock, Search, CheckSquare, Square, 
  User, Calendar, Plus, AlertTriangle, CheckCircle,
  Zap, Database, Settings, Eye, Edit, Trash2
} from 'lucide-react'
import { User as UserType } from '@/types/core'

const grantPermissionSchema = z.object({
  userId: z.string().min(1, 'User ID is required'),
  permissions: z.array(z.string()).min(1, 'At least one permission is required'),
  expiryDate: z.string().optional(),
  expiryTime: z.string().optional(),
  reason: z.string().min(10, 'Please provide a reason (minimum 10 characters)')
})

type GrantPermissionFormData = z.infer<typeof grantPermissionSchema>

interface GrantPermissionFormProps {
  user: UserType
  currentUser: any
  onClose: () => void
}

interface PermissionCategory {
  id: string
  name: string
  icon: any
  color: string
  permissions: PermissionOption[]
}

interface PermissionOption {
  id: string
  label: string
  description: string
  risk: 'low' | 'medium' | 'high'
  category: string
}

export function GrantPermissionForm({ user, currentUser, onClose }: GrantPermissionFormProps) {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([])
  const [isSubmitting, setIsSubmitting] = useState(false)

  const permissionCategories: PermissionCategory[] = [
    {
      id: 'admin',
      name: 'Administration',
      icon: Shield,
      color: 'from-red-500 to-pink-500',
      permissions: [
        {
          id: 'admin:*:*',
          label: 'Full Admin Access',
          description: 'Complete administrative control over the platform',
          risk: 'high',
          category: 'admin'
        },
        {
          id: 'admin:users:view',
          label: 'View Users',
          description: 'View user information and profiles',
          risk: 'low',
          category: 'admin'
        },
        {
          id: 'admin:users:manage',
          label: 'Manage Users',
          description: 'Create, update, and delete user accounts',
          risk: 'high',
          category: 'admin'
        },
        {
          id: 'admin:permissions:view',
          label: 'View Permissions',
          description: 'View permission assignments and roles',
          risk: 'low',
          category: 'admin'
        },
        {
          id: 'admin:permissions:grant',
          label: 'Grant Permissions',
          description: 'Assign permissions to other users',
          risk: 'high',
          category: 'admin'
        }
      ]
    },
    {
      id: 'analytics',
      name: 'Analytics',
      icon: Eye,
      color: 'from-blue-500 to-cyan-500',
      permissions: [
        {
          id: 'epsx:analytics:view',
          label: 'View Analytics',
          description: 'Access analytics dashboards and reports',
          risk: 'low',
          category: 'analytics'
        },
        {
          id: 'epsx:analytics:premium',
          label: 'Premium Analytics',
          description: 'Access advanced analytics features',
          risk: 'medium',
          category: 'analytics'
        },
        {
          id: 'epsx:analytics:export',
          label: 'Export Analytics',
          description: 'Export analytics data and reports',
          risk: 'medium',
          category: 'analytics'
        }
      ]
    },
    {
      id: 'rankings',
      name: 'Rankings',
      icon: Zap,
      color: 'from-yellow-500 to-orange-500',
      permissions: [
        {
          id: 'epsx:rankings:view:1',
          label: 'Basic Rankings (1 stock)',
          description: 'View top 1 stock ranking',
          risk: 'low',
          category: 'rankings'
        },
        {
          id: 'epsx:rankings:view:5',
          label: 'Premium Rankings (5 stocks)',
          description: 'View top 5 stock rankings',
          risk: 'low',
          category: 'rankings'
        },
        {
          id: 'epsx:rankings:view:10',
          label: 'Pro Rankings (10 stocks)',
          description: 'View top 10 stock rankings',
          risk: 'medium',
          category: 'rankings'
        }
      ]
    },
    {
      id: 'system',
      name: 'System',
      icon: Settings,
      color: 'from-purple-500 to-indigo-500',
      permissions: [
        {
          id: 'epsx:system:health',
          label: 'System Health',
          description: 'View system health and status',
          risk: 'low',
          category: 'system'
        },
        {
          id: 'epsx:system:config',
          label: 'System Configuration',
          description: 'Modify system configuration',
          risk: 'high',
          category: 'system'
        }
      ]
    }
  ]

  const allPermissions = permissionCategories.flatMap(cat => cat.permissions)

  const filteredPermissions = useMemo(() => {
    let permissions = allPermissions

    if (selectedCategory !== 'all') {
      permissions = permissions.filter(p => p.category === selectedCategory)
    }

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      permissions = permissions.filter(p => 
        p.label.toLowerCase().includes(query) ||
        p.description.toLowerCase().includes(query) ||
        p.id.toLowerCase().includes(query)
      )
    }

    return permissions
  }, [allPermissions, selectedCategory, searchQuery])

  const { register, handleSubmit, formState: { errors } } = useForm<GrantPermissionFormData>({
    resolver: zodResolver(grantPermissionSchema),
    defaultValues: {
      userId: user.id,
      permissions: [],
      reason: ''
    }
  })

  const handlePermissionToggle = (permissionId: string) => {
    setSelectedPermissions(prev => 
      prev.includes(permissionId) 
        ? prev.filter(id => id !== permissionId)
        : [...prev, permissionId]
    )
  }

  const handleSelectAll = () => {
    const allFilteredIds = filteredPermissions.map(p => p.id)
    const allSelected = allFilteredIds.every(id => selectedPermissions.includes(id))
    
    if (allSelected) {
      setSelectedPermissions(prev => prev.filter(id => !allFilteredIds.includes(id)))
    } else {
      setSelectedPermissions(prev => [...new Set([...prev, ...allFilteredIds])])
    }
  }

  const onSubmit = async (data: GrantPermissionFormData) => {
    setIsSubmitting(true)
    
    try {
      // Combine date and time if both provided
      let expiresAt = undefined
      if (data.expiryDate && data.expiryTime) {
        expiresAt = new Date(`${data.expiryDate}T${data.expiryTime}`).toISOString()
      }

      const payload = {
        userId: user.id,
        permissions: selectedPermissions,
        reason: data.reason,
        expiresAt
      }

      console.log('Granting permissions:', payload)
      
      // TODO: Implement API call
      // await grantPermissions(payload)
      
      // Show success and close
      alert(`Successfully granted ${selectedPermissions.length} permissions to ${user.email}`)
      onClose()
    } catch (error) {
      console.error('Failed to grant permissions:', error)
      alert('Failed to grant permissions. Please try again.')
    } finally {
      setIsSubmitting(false)
    }
  }

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-100'
      case 'medium': return 'text-yellow-600 bg-yellow-100'
      case 'high': return 'text-red-600 bg-red-100'
      default: return 'text-gray-600 bg-gray-100'
    }
  }

  const getRiskIcon = (risk: string) => {
    switch (risk) {
      case 'low': return CheckCircle
      case 'medium': return AlertTriangle
      case 'high': return AlertTriangle
      default: return CheckCircle
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-6xl max-h-[90vh] overflow-hidden bg-white dark:bg-gray-900 rounded-3xl shadow-2xl">
        {/* Header */}
        <div className="sticky top-0 bg-gradient-to-r from-purple-600 via-pink-600 to-red-600 p-6 text-white">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Shield className="h-8 w-8" />
              <div>
                <h2 className="text-2xl font-bold">Grant Permissions</h2>
                <p className="text-purple-100">Assign permissions to {user.email}</p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-white/20 rounded-2xl transition-colors"
            >
              <X className="h-6 w-6" />
            </button>
          </div>
        </div>

        <div className="p-6 overflow-y-auto max-h-[calc(90vh-100px)]">
          {/* User Info Card */}
          <div className="mb-8 p-6 bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 rounded-2xl">
            <div className="flex items-center gap-4">
              <div className="w-16 h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center">
                <User className="h-8 w-8 text-white" />
              </div>
              <div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white">
                  {user.displayName || user.name || 'Unknown User'}
                </h3>
                <p className="text-gray-600 dark:text-gray-300">{user.email}</p>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Current role: {user.role} • {user.permissions?.length || 0} existing permissions
                </p>
              </div>
            </div>
          </div>

          <form onSubmit={handleSubmit(onSubmit)} className="space-y-8">
            {/* Permission Selection */}
            <div>
              <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-6 flex items-center">
                <Shield className="h-6 w-6 mr-3 text-purple-600" />
                Select Permissions to Grant
              </h3>

              {/* Search and Filter */}
              <div className="flex flex-col lg:flex-row gap-4 mb-6">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                  <input
                    type="text"
                    placeholder="Search permissions..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full pl-10 pr-4 py-3 border border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 focus:ring-2 focus:ring-purple-500"
                  />
                </div>
                
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={() => setSelectedCategory('all')}
                    className={`px-4 py-3 rounded-2xl font-medium transition-colors ${
                      selectedCategory === 'all'
                        ? 'bg-purple-600 text-white'
                        : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-purple-100'
                    }`}
                  >
                    All Categories
                  </button>
                  {permissionCategories.map(category => (
                    <button
                      key={category.id}
                      type="button"
                      onClick={() => setSelectedCategory(category.id)}
                      className={`px-4 py-3 rounded-2xl font-medium transition-colors flex items-center gap-2 ${
                        selectedCategory === category.id
                          ? 'bg-purple-600 text-white'
                          : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-purple-100'
                      }`}
                    >
                      <category.icon className="h-4 w-4" />
                      {category.name}
                    </button>
                  ))}
                </div>
              </div>

              {/* Select All Button */}
              <div className="flex justify-between items-center mb-4">
                <button
                  type="button"
                  onClick={handleSelectAll}
                  className="flex items-center gap-2 px-4 py-2 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded-xl hover:bg-purple-200 transition-colors"
                >
                  <CheckSquare className="h-4 w-4" />
                  Select All Visible ({filteredPermissions.length})
                </button>
                <div className="text-sm text-gray-600 dark:text-gray-400">
                  {selectedPermissions.length} permissions selected
                </div>
              </div>

              {/* Permission Grid */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-h-96 overflow-y-auto p-1">
                {filteredPermissions.map(permission => {
                  const isSelected = selectedPermissions.includes(permission.id)
                  const RiskIcon = getRiskIcon(permission.risk)
                  
                  return (
                    <div
                      key={permission.id}
                      onClick={() => handlePermissionToggle(permission.id)}
                      className={`p-4 rounded-2xl border-2 cursor-pointer transition-all hover:scale-102 ${
                        isSelected
                          ? 'border-purple-500 bg-purple-50 dark:bg-purple-900/30'
                          : 'border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 hover:border-purple-300'
                      }`}
                    >
                      <div className="flex items-start gap-3">
                        <div className="mt-1">
                          {isSelected ? (
                            <CheckSquare className="h-5 w-5 text-purple-600" />
                          ) : (
                            <Square className="h-5 w-5 text-gray-400" />
                          )}
                        </div>
                        <div className="flex-1">
                          <div className="flex items-center justify-between mb-2">
                            <h4 className="font-semibold text-gray-900 dark:text-white">
                              {permission.label}
                            </h4>
                            <div className={`flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium ${getRiskColor(permission.risk)}`}>
                              <RiskIcon className="h-3 w-3" />
                              {permission.risk.toUpperCase()}
                            </div>
                          </div>
                          <p className="text-sm text-gray-600 dark:text-gray-300 mb-2">
                            {permission.description}
                          </p>
                          <p className="text-xs font-mono text-gray-500 dark:text-gray-400">
                            {permission.id}
                          </p>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>

              {filteredPermissions.length === 0 && (
                <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                  <AlertTriangle className="h-16 w-16 mx-auto mb-4" />
                  <p>No permissions found matching your criteria</p>
                </div>
              )}
            </div>

            {/* Expiry Settings */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Expiry Date (Optional)
                </label>
                <div className="relative">
                  <Calendar className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                  <input
                    type="date"
                    {...register('expiryDate')}
                    className="w-full pl-10 pr-4 py-3 border border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 focus:ring-2 focus:ring-purple-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Expiry Time (Optional)
                </label>
                <div className="relative">
                  <Clock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                  <input
                    type="time"
                    {...register('expiryTime')}
                    className="w-full pl-10 pr-4 py-3 border border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 focus:ring-2 focus:ring-purple-500"
                  />
                </div>
              </div>
            </div>

            {/* Reason */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Reason for Granting Permissions *
              </label>
              <textarea
                {...register('reason')}
                rows={4}
                placeholder="Please explain why these permissions are being granted..."
                className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 focus:ring-2 focus:ring-purple-500 resize-none"
              />
              {errors.reason && (
                <p className="mt-1 text-sm text-red-600">{errors.reason.message}</p>
              )}
            </div>

            {/* Action Buttons */}
            <div className="flex justify-end gap-4 pt-6 border-t border-gray-200 dark:border-gray-600">
              <button
                type="button"
                onClick={onClose}
                className="px-6 py-3 border border-gray-300 text-gray-700 rounded-2xl hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={selectedPermissions.length === 0 || isSubmitting}
                className="px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-2xl hover:from-purple-700 hover:to-pink-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 transition-all"
              >
                {isSubmitting ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    Granting...
                  </>
                ) : (
                  <>
                    <Plus className="h-5 w-5" />
                    Grant {selectedPermissions.length} Permission{selectedPermissions.length !== 1 ? 's' : ''}
                  </>
                )}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  )
}