'use client'

import { useState, useEffect, useMemo } from 'react'
import { 
  Users, 
  Clock, 
  Shield, 
  Filter, 
  Search,
  Calendar,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  Plus,
  Settings,
  BarChart3,
  Activity,
  Target,
  Zap,
  Eye,
  Edit,
  Trash2,
  Copy,
  Download,
  Upload
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Separator } from '@/components/ui/separator'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { format, formatDistance, isAfter, isBefore, addHours, addDays } from 'date-fns'

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface PermissionAssignment {
  id: string
  userId: string
  userName: string
  userEmail: string
  basePermission: string
  platform: string
  resource: string
  action: string
  expiryTimestamp: number
  assignedAt: Date
  assignedBy: string
  reason?: string
  status: 'active' | 'expired' | 'expiring_soon'
  isEmbeddedTimestamp: boolean
}

interface PermissionTemplate {
  id: string
  name: string
  description: string
  basePermission: string
  defaultDuration: number // minutes
  category: 'analytics' | 'admin' | 'premium' | 'emergency'
  tags: string[]
}

interface DashboardStats {
  totalAssignments: number
  activeAssignments: number
  expiredAssignments: number
  expiringSoonAssignments: number
  mostUsedPermission: string
  averagePermissionDuration: number
}

interface DynamicPermissionAssignmentDashboardProps {
  className?: string
}

// ============================================================================
// MOCK DATA (In real implementation, this would come from API)
// ============================================================================

const MOCK_ASSIGNMENTS: PermissionAssignment[] = [
  {
    id: '1',
    userId: 'user-1',
    userName: 'John Doe',
    userEmail: 'john@example.com',
    basePermission: 'epsx:analytics:view',
    platform: 'epsx',
    resource: 'analytics',
    action: 'view',
    expiryTimestamp: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
    assignedAt: new Date(Date.now() - 1800000), // 30 minutes ago
    assignedBy: 'admin',
    reason: 'Temporary analytics access for data review',
    status: 'active',
    isEmbeddedTimestamp: true
  },
  {
    id: '2',
    userId: 'user-2',
    userName: 'Jane Smith',
    userEmail: 'jane@example.com',
    basePermission: 'epsx:rankings:view:100',
    platform: 'epsx',
    resource: 'rankings',
    action: 'view',
    expiryTimestamp: Math.floor(Date.now() / 1000) + 86400, // 24 hours from now
    assignedAt: new Date(Date.now() - 7200000), // 2 hours ago
    assignedBy: 'admin',
    reason: 'Premium ranking access for client demo',
    status: 'active',
    isEmbeddedTimestamp: true
  },
  {
    id: '3',
    userId: 'user-3',
    userName: 'Bob Wilson',
    userEmail: 'bob@example.com',
    basePermission: 'admin:users:manage',
    platform: 'admin',
    resource: 'users',
    action: 'manage',
    expiryTimestamp: Math.floor(Date.now() / 1000) - 1800, // Expired 30 minutes ago
    assignedAt: new Date(Date.now() - 10800000), // 3 hours ago
    assignedBy: 'super-admin',
    reason: 'Emergency user management access',
    status: 'expired',
    isEmbeddedTimestamp: true
  }
]

const MOCK_TEMPLATES: PermissionTemplate[] = [
  {
    id: 'temp-1',
    name: 'Analytics Viewer',
    description: 'Temporary access to analytics dashboard',
    basePermission: 'epsx:analytics:view',
    defaultDuration: 240, // 4 hours
    category: 'analytics',
    tags: ['analytics', 'dashboard', 'temporary']
  },
  {
    id: 'temp-2',
    name: 'Premium Rankings',
    description: 'Access to premium ranking data',
    basePermission: 'epsx:rankings:view:100',
    defaultDuration: 1440, // 24 hours
    category: 'premium',
    tags: ['rankings', 'premium', 'data']
  },
  {
    id: 'temp-3',
    name: 'Emergency Admin',
    description: 'Emergency administrative access',
    basePermission: 'admin:users:manage',
    defaultDuration: 480, // 8 hours
    category: 'emergency',
    tags: ['admin', 'emergency', 'users']
  }
]

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function DynamicPermissionAssignmentDashboard({ 
  className = '' 
}: DynamicPermissionAssignmentDashboardProps) {
  const { toast } = useToast()
  
  // State management
  const [assignments, setAssignments] = useState<PermissionAssignment[]>(MOCK_ASSIGNMENTS)
  const [templates] = useState<PermissionTemplate[]>(MOCK_TEMPLATES)
  const [selectedAssignments, setSelectedAssignments] = useState<string[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'expired' | 'expiring_soon'>('all')
  const [platformFilter, setPlatformFilter] = useState<'all' | 'epsx' | 'admin' | 'epsx-pay' | 'epsx-token'>('all')
  const [showBulkActions, setShowBulkActions] = useState(false)
  const [loading, setLoading] = useState(false)

  // Update assignment statuses based on current time
  useEffect(() => {
    const updateStatuses = () => {
      const now = Date.now() / 1000
      setAssignments(prev => prev.map(assignment => {
        if (assignment.expiryTimestamp <= now) {
          return { ...assignment, status: 'expired' }
        } else if (assignment.expiryTimestamp <= now + 3600) { // Expiring within 1 hour
          return { ...assignment, status: 'expiring_soon' }
        }
        return { ...assignment, status: 'active' }
      }))
    }

    updateStatuses()
    const interval = setInterval(updateStatuses, 60000) // Update every minute
    return () => clearInterval(interval)
  }, [])

  // Computed values
  const filteredAssignments = useMemo(() => {
    return assignments.filter(assignment => {
      // Status filter
      if (statusFilter !== 'all' && assignment.status !== statusFilter) {
        return false
      }
      
      // Platform filter
      if (platformFilter !== 'all' && assignment.platform !== platformFilter) {
        return false
      }
      
      // Search filter
      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase()
        return (
          assignment.userName.toLowerCase().includes(searchLower) ||
          assignment.userEmail.toLowerCase().includes(searchLower) ||
          assignment.basePermission.toLowerCase().includes(searchLower) ||
          assignment.reason?.toLowerCase().includes(searchLower)
        )
      }
      
      return true
    })
  }, [assignments, statusFilter, platformFilter, searchTerm])

  const dashboardStats = useMemo(() => {
    const stats: DashboardStats = {
      totalAssignments: assignments.length,
      activeAssignments: assignments.filter(a => a.status === 'active').length,
      expiredAssignments: assignments.filter(a => a.status === 'expired').length,
      expiringSoonAssignments: assignments.filter(a => a.status === 'expiring_soon').length,
      mostUsedPermission: '',
      averagePermissionDuration: 0
    }

    // Calculate most used permission
    const permissionCounts: Record<string, number> = {}
    assignments.forEach(assignment => {
      permissionCounts[assignment.basePermission] = (permissionCounts[assignment.basePermission] || 0) + 1
    })
    
    stats.mostUsedPermission = Object.keys(permissionCounts).reduce((a, b) => 
      permissionCounts[a] > permissionCounts[b] ? a : b
    ) || 'None'

    // Calculate average duration
    const totalDuration = assignments.reduce((sum, assignment) => {
      const duration = assignment.expiryTimestamp - Math.floor(assignment.assignedAt.getTime() / 1000)
      return sum + duration
    }, 0)
    
    stats.averagePermissionDuration = assignments.length > 0 ? totalDuration / assignments.length : 0

    return stats
  }, [assignments])

  // Event handlers
  const handleBulkRevoke = async () => {
    if (selectedAssignments.length === 0) return
    
    setLoading(true)
    try {
      // In real implementation, this would call an API
      toast({
        title: 'Success',
        description: `Revoked ${selectedAssignments.length} permission${selectedAssignments.length !== 1 ? 's' : ''}`,
      })
      setSelectedAssignments([])
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to revoke permissions',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleBulkExtend = async (additionalMinutes: number) => {
    if (selectedAssignments.length === 0) return
    
    setLoading(true)
    try {
      setAssignments(prev => prev.map(assignment => {
        if (selectedAssignments.includes(assignment.id)) {
          return {
            ...assignment,
            expiryTimestamp: assignment.expiryTimestamp + (additionalMinutes * 60)
          }
        }
        return assignment
      }))
      
      toast({
        title: 'Success',
        description: `Extended ${selectedAssignments.length} permission${selectedAssignments.length !== 1 ? 's' : ''} by ${additionalMinutes} minutes`,
      })
      setSelectedAssignments([])
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to extend permissions',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleSelectAll = () => {
    if (selectedAssignments.length === filteredAssignments.length) {
      setSelectedAssignments([])
    } else {
      setSelectedAssignments(filteredAssignments.map(a => a.id))
    }
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Permission Management Dashboard</h2>
          <p className="text-muted-foreground">
            Manage embedded timestamp permissions across all users
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
          <Button size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Bulk Assign
          </Button>
        </div>
      </div>

      {/* Dashboard Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-blue-500" />
              <div>
                <p className="text-2xl font-bold">{dashboardStats.totalAssignments}</p>
                <p className="text-sm text-muted-foreground">Total Assignments</p>
              </div>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-2xl font-bold">{dashboardStats.activeAssignments}</p>
                <p className="text-sm text-muted-foreground">Active</p>
              </div>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              <div>
                <p className="text-2xl font-bold">{dashboardStats.expiringSoonAssignments}</p>
                <p className="text-sm text-muted-foreground">Expiring Soon</p>
              </div>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-red-500" />
              <div>
                <p className="text-2xl font-bold">{dashboardStats.expiredAssignments}</p>
                <p className="text-sm text-muted-foreground">Expired</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters and Search */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Filter className="h-5 w-5" />
            Filters & Search
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="space-y-2">
              <Label>Search</Label>
              <div className="relative">
                <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search users, permissions..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-9"
                />
              </div>
            </div>
            
            <div className="space-y-2">
              <Label>Status</Label>
              <Select value={statusFilter} onValueChange={(value: any) => setStatusFilter(value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Statuses</SelectItem>
                  <SelectItem value="active">Active Only</SelectItem>
                  <SelectItem value="expiring_soon">Expiring Soon</SelectItem>
                  <SelectItem value="expired">Expired Only</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="space-y-2">
              <Label>Platform</Label>
              <Select value={platformFilter} onValueChange={(value: any) => setPlatformFilter(value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Platforms</SelectItem>
                  <SelectItem value="epsx">EPSX</SelectItem>
                  <SelectItem value="admin">Admin</SelectItem>
                  <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                  <SelectItem value="epsx-token">EPSX Token</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="space-y-2">
              <Label>Actions</Label>
              <div className="flex gap-2">
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={handleSelectAll}
                >
                  {selectedAssignments.length === filteredAssignments.length ? 'Deselect All' : 'Select All'}
                </Button>
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => setShowBulkActions(!showBulkActions)}
                  disabled={selectedAssignments.length === 0}
                >
                  Bulk Actions ({selectedAssignments.length})
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Bulk Actions */}
      {showBulkActions && selectedAssignments.length > 0 && (
        <Card className="border-blue-200 bg-blue-50 dark:bg-blue-900/20">
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Zap className="h-5 w-5 text-blue-600" />
                <span className="font-medium">
                  {selectedAssignments.length} permission{selectedAssignments.length !== 1 ? 's' : ''} selected
                </span>
              </div>
              <div className="flex gap-2">
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => handleBulkExtend(60)}
                  disabled={loading}
                >
                  Extend +1h
                </Button>
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => handleBulkExtend(240)}
                  disabled={loading}
                >
                  Extend +4h
                </Button>
                <Button 
                  variant="destructive" 
                  size="sm"
                  onClick={handleBulkRevoke}
                  disabled={loading}
                >
                  Revoke All
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Assignments Table */}
      <Card>
        <CardHeader>
          <CardTitle>Permission Assignments ({filteredAssignments.length})</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {filteredAssignments.length === 0 ? (
              <div className="text-center py-8">
                <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                <h3 className="text-lg font-medium mb-2">No assignments found</h3>
                <p className="text-muted-foreground">
                  {searchTerm || statusFilter !== 'all' || platformFilter !== 'all' 
                    ? 'Try adjusting your filters'
                    : 'No permission assignments have been created yet'
                  }
                </p>
              </div>
            ) : (
              filteredAssignments.map((assignment) => (
                <PermissionAssignmentCard
                  key={assignment.id}
                  assignment={assignment}
                  isSelected={selectedAssignments.includes(assignment.id)}
                  onSelect={(selected) => {
                    if (selected) {
                      setSelectedAssignments(prev => [...prev, assignment.id])
                    } else {
                      setSelectedAssignments(prev => prev.filter(id => id !== assignment.id))
                    }
                  }}
                />
              ))
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// ============================================================================
// ASSIGNMENT CARD COMPONENT
// ============================================================================

interface PermissionAssignmentCardProps {
  assignment: PermissionAssignment
  isSelected: boolean
  onSelect: (selected: boolean) => void
}

function PermissionAssignmentCard({ assignment, isSelected, onSelect }: PermissionAssignmentCardProps) {
  const getStatusBadge = (status: string) => {
    const variants = {
      active: { variant: 'default' as const, icon: CheckCircle, className: 'bg-green-100 text-green-800' },
      expired: { variant: 'secondary' as const, icon: Clock, className: 'bg-red-100 text-red-800' },
      expiring_soon: { variant: 'destructive' as const, icon: AlertTriangle, className: 'bg-yellow-100 text-yellow-800' }
    }
    
    const config = variants[status as keyof typeof variants] || variants.active
    const Icon = config.icon
    
    return (
      <Badge className={config.className}>
        <Icon className="h-3 w-3 mr-1" />
        {status.replace('_', ' ')}
      </Badge>
    )
  }

  const getTimeRemaining = () => {
    const now = Date.now() / 1000
    const remaining = assignment.expiryTimestamp - now
    
    if (remaining <= 0) {
      return 'Expired'
    }
    
    return formatDistance(new Date(assignment.expiryTimestamp * 1000), new Date(), { addSuffix: true })
  }

  return (
    <Card className={`transition-colors ${isSelected ? 'ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/20' : ''}`}>
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3 flex-1">
            <Checkbox 
              checked={isSelected} 
              onCheckedChange={onSelect}
            />
            
            <div className="flex-1 space-y-3">
              {/* Header */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div>
                    <p className="font-medium">{assignment.userName}</p>
                    <p className="text-sm text-muted-foreground">{assignment.userEmail}</p>
                  </div>
                </div>
                {getStatusBadge(assignment.status)}
              </div>

              {/* Permission Details */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <p className="text-xs text-muted-foreground">Permission</p>
                  <p className="font-mono text-sm">{assignment.basePermission}</p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Assigned</p>
                  <p className="text-sm">{format(assignment.assignedAt, 'MMM d, HH:mm')}</p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Expires</p>
                  <p className="text-sm">{getTimeRemaining()}</p>
                </div>
              </div>

              {/* Reason */}
              {assignment.reason && (
                <div>
                  <p className="text-xs text-muted-foreground">Reason</p>
                  <p className="text-sm">{assignment.reason}</p>
                </div>
              )}

              {/* Embedded Timestamp Indicator */}
              {assignment.isEmbeddedTimestamp && (
                <div className="flex items-center gap-1 text-xs text-blue-600">
                  <Zap className="h-3 w-3" />
                  Embedded timestamp permission
                </div>
              )}
            </div>
          </div>
          
          {/* Actions */}
          <div className="flex gap-1">
            <Button variant="ghost" size="sm">
              <Eye className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Edit className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}