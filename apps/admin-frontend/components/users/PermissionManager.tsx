/**
 * Unified Permission Manager Component
 * Consolidates: PermissionAssignmentCard, PermissionHistoryCard, PermissionStatsCards,
 * PermissionValidator, PermissionConflictResolver, PermissionRecommendations,
 * InteractivePermissionTreeView, PermissionExportImport
 */

'use client'

import React, { useState, useEffect, useRef, useMemo } from 'react'
import { useRouter } from 'next/navigation'
import { 
  Shield, Key, Users, Clock, AlertTriangle, CheckCircle, XCircle, 
  Info, TrendingUp, Settings, Download, Upload, Eye, EyeOff,
  Lightbulb, Sparkles, RefreshCw, Copy, Check, Plus, Minus,
  ChevronRight, ChevronDown, Folder, FolderOpen, Search, Filter,
  MoreHorizontal, User, Calendar, History, FileJson, FileSpreadsheet,
  Archive, Play, Pause, Globe, MapPin, Smartphone
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/form-components'
import { 
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter 
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'

import type { UnifiedUserData, Permission } from '@/lib/types/unified-user'
import { adminCardVariants, adminButtonVariants, cn } from '@/design-system'

// Types
interface PermissionManagerProps {
  user: UnifiedUserData
  currentUser?: any
  onPermissionChange?: (userId: string, permissions: string[]) => void
  onUserUpdate?: () => void
  className?: string
}

interface PermissionNode {
  id: string
  name: string
  type: 'resource' | 'action' | 'permission'
  children?: PermissionNode[]
  granted: boolean
  inherited: boolean
  source?: string
  description?: string
  riskLevel: 'low' | 'medium' | 'high'
  category: string
}

interface ValidationResult {
  isValid: boolean
  conflicts: Array<{
    type: 'role_conflict' | 'permission_duplicate' | 'hierarchy_violation'
    severity: 'error' | 'warning' | 'info'
    message: string
    details?: string
    suggestion?: string
  }>
  warnings: string[]
}

interface PermissionRecommendation {
  id: string
  type: 'add' | 'remove' | 'upgrade' | 'temporary'
  permission: string
  confidence: number
  reasoning: string
  category: 'security' | 'efficiency' | 'compliance' | 'role-based'
  impact: 'low' | 'medium' | 'high'
  estimatedBenefit: string
  risks: string[]
}

interface PermissionHistoryEntry {
  id: string
  action: 'granted' | 'revoked' | 'modified'
  type: 'role' | 'permission' | 'profile'
  role?: string
  resource?: string
  permission?: string
  profileId?: string
  reason?: string
  grantedBy: string
  grantedAt: Date
  expires?: Date
}

export function PermissionManager({
  user,
  currentUser,
  onPermissionChange,
  onUserUpdate,
  className = ''
}: PermissionManagerProps) {
  const router = useRouter()
  const { toast } = useToast()
  const fileInputRef = useRef<HTMLInputElement>(null)

  // State
  const [activeTab, setActiveTab] = useState<'overview' | 'assign' | 'history' | 'validate' | 'recommendations' | 'export'>('overview')
  const [loading, setLoading] = useState(false)
  const [searchTerm, setSearchTerm] = useState('')
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set())
  const [selectedNodes, setSelectedNodes] = useState<Set<string>>(new Set())
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null)
  const [recommendations, setRecommendations] = useState<PermissionRecommendation[]>([])
  const [permissionHistory, setPermissionHistory] = useState<PermissionHistoryEntry[]>([])
  const [exportData, setExportData] = useState<any>(null)
  const [importData, setImportData] = useState('')
  const [copied, setCopied] = useState(false)

  // Permission stats
  const permissionStats = useMemo(() => {
    const roles = user.roles || []
    const customPerms = user.customPermissions || []
    const profiles = user.permissionProfiles || []
    
    return {
      totalRoles: roles.length,
      totalPermissions: customPerms.length,
      totalProfiles: profiles.length,
      expiringCount: customPerms.filter(p => p.expiresAt).length,
      activeCount: customPerms.filter(p => p.isActive).length
    }
  }, [user])

  // Mock permission tree data
  const permissionTree: PermissionNode[] = [
    {
      id: 'admin',
      name: 'Administration',
      type: 'resource',
      granted: false,
      inherited: false,
      riskLevel: 'high',
      category: 'admin',
      children: [
        {
          id: 'admin-users',
          name: 'User Management',
          type: 'resource',
          granted: true,
          inherited: true,
          source: 'admin role',
          riskLevel: 'high',
          category: 'admin',
          description: 'Manage user accounts and permissions'
        },
        {
          id: 'admin-system',
          name: 'System Configuration',
          type: 'resource',
          granted: false,
          inherited: false,
          riskLevel: 'high',
          category: 'admin',
          description: 'Configure system settings'
        }
      ]
    },
    {
      id: 'analytics',
      name: 'Analytics',
      type: 'resource',
      granted: true,
      inherited: false,
      riskLevel: 'medium',
      category: 'business',
      children: [
        {
          id: 'analytics-view',
          name: 'View Analytics',
          type: 'permission',
          granted: true,
          inherited: false,
          riskLevel: 'low',
          category: 'business',
          description: 'Access analytics dashboards'
        },
        {
          id: 'analytics-export',
          name: 'Export Data',
          type: 'permission',
          granted: false,
          inherited: false,
          riskLevel: 'medium',
          category: 'business',
          description: 'Export analytics data'
        }
      ]
    }
  ]

  // Load data
  useEffect(() => {
    loadPermissionData()
  }, [user.id])

  const loadPermissionData = async () => {
    setLoading(true)
    try {
      // Mock data loading - replace with actual API calls
      const mockHistory: PermissionHistoryEntry[] = [
        {
          id: '1',
          action: 'granted',
          type: 'permission',
          resource: 'analytics',
          permission: 'view',
          grantedBy: 'admin@epsx.io',
          grantedAt: new Date(Date.now() - 86400000),
          reason: 'New user onboarding'
        },
        {
          id: '2',
          action: 'revoked',
          type: 'role',
          role: 'temporary-admin',
          grantedBy: 'admin@epsx.io',
          grantedAt: new Date(Date.now() - 172800000),
          reason: 'Temporary access expired'
        }
      ]

      const mockRecommendations: PermissionRecommendation[] = [
        {
          id: '1',
          type: 'add',
          permission: 'analytics:export',
          confidence: 92,
          reasoning: 'User frequently views analytics and similar users have export access',
          category: 'efficiency',
          impact: 'medium',
          estimatedBenefit: 'Reduce approval wait time by 75%',
          risks: []
        },
        {
          id: '2',
          type: 'remove',
          permission: 'admin:delete',
          confidence: 88,
          reasoning: 'Permission not used in 90+ days and exceeds role requirements',
          category: 'security',
          impact: 'high',
          estimatedBenefit: 'Reduce security risk exposure by 40%',
          risks: ['High privilege level', 'Unused for extended period']
        }
      ]

      setPermissionHistory(mockHistory)
      setRecommendations(mockRecommendations)
    } catch (error) {
      console.error('Failed to load permission data:', error)
    } finally {
      setLoading(false)
    }
  }

  // Utility functions
  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-100'
      case 'medium': return 'text-orange-600 bg-orange-100'
      case 'high': return 'text-red-600 bg-red-100'
      default: return 'text-gray-600 bg-gray-100'
    }
  }

  const getActionColor = (action: string) => {
    switch (action.toLowerCase()) {
      case 'read': return 'bg-blue-100 text-blue-800'
      case 'write': return 'bg-green-100 text-green-800'
      case 'delete': return 'bg-red-100 text-red-800'
      case 'admin': return 'bg-purple-100 text-purple-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  // Permission assignment
  const handlePermissionToggle = (nodeId: string, granted: boolean) => {
    toast({
      title: granted ? 'Permission Granted' : 'Permission Revoked',
      description: `${nodeId} has been ${granted ? 'granted' : 'revoked'}`,
    })
    onPermissionChange?.(user.id, [])
  }

  // Export functionality
  const handleExport = async () => {
    setLoading(true)
    try {
      const exportData = {
        user: {
          id: user.id,
          email: user.email,
          roles: user.roles,
          customPermissions: user.customPermissions,
          permissionProfiles: user.permissionProfiles
        },
        exportedAt: new Date().toISOString(),
        version: '1.0'
      }
      setExportData(exportData)
      toast({
        title: 'Export Complete',
        description: 'Permission data ready for download'
      })
    } catch (error) {
      toast({
        title: 'Export Failed',
        description: 'Unable to export permission data',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }

  const handleCopyExport = async () => {
    if (!exportData) return
    try {
      await navigator.clipboard.writeText(JSON.stringify(exportData, null, 2))
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
      toast({
        title: 'Copied to Clipboard',
        description: 'Permission data copied successfully'
      })
    } catch (error) {
      toast({
        title: 'Copy Failed',
        description: 'Unable to copy to clipboard',
        variant: 'destructive'
      })
    }
  }

  // Tree node component
  const TreeNode = ({ node, level }: { node: PermissionNode; level: number }) => {
    const isExpanded = expandedNodes.has(node.id)
    const isSelected = selectedNodes.has(node.id)
    const hasChildren = node.children && node.children.length > 0

    const toggleExpand = () => {
      setExpandedNodes(prev => {
        const newSet = new Set(prev)
        if (newSet.has(node.id)) {
          newSet.delete(node.id)
        } else {
          newSet.add(node.id)
        }
        return newSet
      })
    }

    const toggleSelect = () => {
      setSelectedNodes(prev => {
        const newSet = new Set(prev)
        if (newSet.has(node.id)) {
          newSet.delete(node.id)
        } else {
          newSet.add(node.id)
        }
        return newSet
      })
    }

    return (
      <div>
        <div 
          className={`flex items-center py-2 px-2 hover:bg-gray-50 rounded-lg ${
            isSelected ? 'bg-blue-50 border border-blue-200' : ''
          }`}
          style={{ marginLeft: `${level * 20}px` }}
        >
          {/* Expand/Collapse */}
          <div className="w-6 h-6 flex items-center justify-center">
            {hasChildren ? (
              <button onClick={toggleExpand} className="hover:bg-gray-200 rounded p-1">
                {isExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
              </button>
            ) : (
              <div className="w-4 h-4" />
            )}
          </div>

          {/* Selection */}
          <div className="w-6 h-6 flex items-center justify-center">
            <input
              type="checkbox"
              checked={isSelected}
              onChange={toggleSelect}
              className="rounded border-gray-300"
            />
          </div>

          {/* Icon */}
          <div className="w-6 h-6 flex items-center justify-center mr-2">
            {node.type === 'resource' ? (
              hasChildren ? (
                isExpanded ? <FolderOpen className="h-4 w-4 text-blue-500" /> : <Folder className="h-4 w-4 text-blue-500" />
              ) : (
                <Shield className="h-4 w-4 text-purple-500" />
              )
            ) : (
              <Key className="h-4 w-4 text-green-500" />
            )}
          </div>

          {/* Permission status */}
          <div className="w-6 h-6 flex items-center justify-center mr-3">
            <button
              onClick={() => handlePermissionToggle(node.id, !node.granted)}
              disabled={node.inherited}
              className={`rounded p-1 ${!node.inherited ? 'hover:bg-gray-200' : 'cursor-not-allowed'}`}
            >
              {node.granted ? 
                <CheckCircle className={`h-4 w-4 ${node.inherited ? 'text-blue-500' : 'text-green-500'}`} /> : 
                <XCircle className="h-4 w-4 text-red-500" />
              }
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-medium text-gray-900">{node.name}</span>
              <Badge className={getRiskColor(node.riskLevel)} variant="secondary">
                {node.riskLevel}
              </Badge>
              <Badge variant="outline" className="text-xs">{node.category}</Badge>
              {node.inherited && (
                <Badge variant="secondary" className="text-xs text-blue-600 bg-blue-100">
                  inherited from {node.source}
                </Badge>
              )}
            </div>
            {node.description && (
              <p className="text-sm text-gray-600 mt-1 truncate">{node.description}</p>
            )}
          </div>

          {/* Quick actions */}
          <div className="opacity-0 group-hover:opacity-100 transition-opacity">
            <div className="flex gap-1">
              <Button
                size="sm"
                variant="ghost"
                onClick={() => handlePermissionToggle(node.id, true)}
                className="h-6 w-6 p-0"
              >
                <Plus className="h-3 w-3" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => handlePermissionToggle(node.id, false)}
                className="h-6 w-6 p-0"
              >
                <Minus className="h-3 w-3" />
              </Button>
            </div>
          </div>
        </div>

        {/* Children */}
        {hasChildren && isExpanded && (
          <div>
            {node.children!.map(child => (
              <TreeNode key={child.id} node={child} level={level + 1} />
            ))}
          </div>
        )}
      </div>
    )
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Permission Manager</h2>
          <p className="text-gray-600">Manage {user.email}'s permissions and access control</p>
        </div>
        <Button 
          onClick={onUserUpdate} 
          variant="outline"
          disabled={loading}
        >
          <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={(value: any) => setActiveTab(value)}>
        <TabsList className="grid w-full grid-cols-6">
          <TabsTrigger value="overview" className="flex items-center gap-1">
            <Shield className="h-3 w-3" />
            Overview
          </TabsTrigger>
          <TabsTrigger value="assign" className="flex items-center gap-1">
            <Key className="h-3 w-3" />
            Assign
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-1">
            <History className="h-3 w-3" />
            History
          </TabsTrigger>
          <TabsTrigger value="validate" className="flex items-center gap-1">
            <CheckCircle className="h-3 w-3" />
            Validate
          </TabsTrigger>
          <TabsTrigger value="recommendations" className="flex items-center gap-1">
            <Lightbulb className="h-3 w-3" />
            AI Suggest
          </TabsTrigger>
          <TabsTrigger value="export" className="flex items-center gap-1">
            <Download className="h-3 w-3" />
            Export
          </TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-6">
          {/* Stats Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-2xl font-bold text-blue-600">{permissionStats.totalRoles}</p>
                    <p className="text-sm text-muted-foreground">Active Roles</p>
                  </div>
                  <Users className="h-8 w-8 text-blue-500" />
                </div>
              </CardContent>
            </Card>
            
            <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-2xl font-bold text-green-600">{permissionStats.totalPermissions}</p>
                    <p className="text-sm text-muted-foreground">Custom Permissions</p>
                  </div>
                  <Key className="h-8 w-8 text-green-500" />
                </div>
              </CardContent>
            </Card>

            <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-2xl font-bold text-purple-600">{permissionStats.totalProfiles}</p>
                    <p className="text-sm text-muted-foreground">Permission Profiles</p>
                  </div>
                  <Shield className="h-8 w-8 text-purple-500" />
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Current Permissions */}
          <Card>
            <CardHeader>
              <CardTitle>Current Permissions</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {user.customPermissions?.slice(0, 5).map((permission, index) => (
                  <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-3">
                      <Key className="h-4 w-4 text-green-500" />
                      <div>
                        <span className="font-medium">{permission.name}</span>
                        <div className="text-xs text-muted-foreground">
                          {permission.description || 'Custom permission'}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge className={getActionColor('read')}>Active</Badge>
                      {permission.expiresAt && (
                        <Badge variant="outline" className="text-xs">
                          Expires {formatDate(permission.expiresAt)}
                        </Badge>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Assign Tab */}
        <TabsContent value="assign" className="space-y-6">
          <Card>
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5" />
                  Permission Tree View
                </CardTitle>
                <div className="flex gap-2">
                  <Button variant="outline" size="sm" onClick={() => setExpandedNodes(new Set(permissionTree.map(n => n.id)))}>
                    Expand All
                  </Button>
                  <Button variant="outline" size="sm" onClick={() => setExpandedNodes(new Set())}>
                    Collapse All
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {/* Search */}
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <Input
                    placeholder="Search permissions..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-10"
                  />
                </div>

                {/* Tree */}
                <div className="space-y-1 max-h-96 overflow-y-auto border rounded-lg p-4">
                  {permissionTree.map(node => (
                    <TreeNode key={node.id} node={node} level={0} />
                  ))}
                </div>

                {/* Bulk Actions */}
                {selectedNodes.size > 0 && (
                  <div className="flex gap-2">
                    <Button
                      onClick={() => {
                        selectedNodes.forEach(nodeId => handlePermissionToggle(nodeId, true))
                        setSelectedNodes(new Set())
                      }}
                      className="bg-green-600 hover:bg-green-700"
                    >
                      Grant Selected ({selectedNodes.size})
                    </Button>
                    <Button
                      variant="destructive"
                      onClick={() => {
                        selectedNodes.forEach(nodeId => handlePermissionToggle(nodeId, false))
                        setSelectedNodes(new Set())
                      }}
                    >
                      Revoke Selected ({selectedNodes.size})
                    </Button>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <History className="h-5 w-5" />
                Permission History
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {permissionHistory.map(entry => (
                  <div key={entry.id} className="flex items-start gap-3 p-3 rounded-lg border">
                    <div className="p-2 rounded-full bg-background border">
                      {entry.action === 'granted' ? (
                        <CheckCircle className="h-4 w-4 text-green-600" />
                      ) : (
                        <XCircle className="h-4 w-4 text-red-600" />
                      )}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-medium">
                          {entry.action === 'granted' ? 'Granted' : 'Revoked'}
                        </span>
                        <Badge variant="outline">{entry.type}</Badge>
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {entry.resource && entry.permission && (
                          <span>Permission: {entry.resource}:{entry.permission}</span>
                        )}
                        {entry.role && <span>Role: {entry.role}</span>}
                      </div>
                      {entry.reason && (
                        <div className="text-xs text-muted-foreground mt-1 italic">
                          Reason: {entry.reason}
                        </div>
                      )}
                      <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                        <span>by {entry.grantedBy}</span>
                        <span>{formatDate(entry.grantedAt)}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Validate Tab */}
        <TabsContent value="validate" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CheckCircle className="h-5 w-5" />
                Permission Validation
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <Button onClick={() => setValidationResult({ isValid: true, conflicts: [], warnings: [] })}>
                  <Shield className="h-4 w-4 mr-2" />
                  Validate All Permissions
                </Button>

                {validationResult && (
                  <Alert className={validationResult.isValid ? 'border-green-200 bg-green-50' : 'border-red-200 bg-red-50'}>
                    <div className="flex items-center gap-2">
                      {validationResult.isValid ? (
                        <CheckCircle className="h-4 w-4 text-green-600" />
                      ) : (
                        <XCircle className="h-4 w-4 text-red-600" />
                      )}
                      <AlertDescription>
                        {validationResult.isValid 
                          ? 'All permissions are valid with no conflicts detected'
                          : `Found ${validationResult.conflicts.length} conflicts`
                        }
                      </AlertDescription>
                    </div>
                  </Alert>
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Recommendations Tab */}
        <TabsContent value="recommendations" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Lightbulb className="h-5 w-5 text-yellow-500" />
                AI Permission Recommendations
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {recommendations.map(rec => (
                  <Card key={rec.id} className="border-l-4 border-l-blue-500">
                    <CardContent className="p-4">
                      <div className="flex justify-between items-start mb-3">
                        <div className="flex items-start gap-3">
                          {rec.type === 'add' ? (
                            <CheckCircle className="h-4 w-4 text-green-500" />
                          ) : (
                            <XCircle className="h-4 w-4 text-red-500" />
                          )}
                          <div>
                            <h4 className="font-semibold flex items-center gap-2">
                              {rec.type.charAt(0).toUpperCase() + rec.type.slice(1)} 
                              <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                                {rec.permission}
                              </code>
                              <Badge className="bg-blue-100 text-blue-600">
                                {rec.confidence}% confidence
                              </Badge>
                            </h4>
                          </div>
                        </div>
                        <Badge className={getRiskColor(rec.impact)}>
                          {rec.impact} impact
                        </Badge>
                      </div>

                      <p className="text-sm mb-3">{rec.reasoning}</p>
                      
                      <div className="bg-green-50 p-3 rounded-lg mb-3">
                        <p className="text-sm font-medium text-green-800">Expected Benefit:</p>
                        <p className="text-sm text-green-700">{rec.estimatedBenefit}</p>
                      </div>

                      {rec.risks.length > 0 && (
                        <div className="bg-red-50 p-3 rounded-lg mb-3">
                          <p className="text-sm font-medium text-red-800">Risks:</p>
                          <ul className="text-sm text-red-700 list-disc list-inside">
                            {rec.risks.map((risk, index) => (
                              <li key={index}>{risk}</li>
                            ))}
                          </ul>
                        </div>
                      )}

                      <div className="flex justify-end gap-2 pt-3 border-t">
                        <Button variant="outline" size="sm">Dismiss</Button>
                        <Button size="sm">Apply Recommendation</Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Export Tab */}
        <TabsContent value="export" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Download className="h-5 w-5" />
                Export/Import Permissions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Export Section */}
              <div className="space-y-4">
                <h3 className="text-lg font-medium">Export</h3>
                <Button onClick={handleExport} disabled={loading}>
                  <Download className="h-4 w-4 mr-2" />
                  Export Permissions as JSON
                </Button>

                {exportData && (
                  <div className="space-y-4">
                    <Label>Export Preview</Label>
                    <Textarea
                      value={JSON.stringify(exportData, null, 2)}
                      readOnly
                      className="font-mono text-xs min-h-[200px]"
                    />
                    <div className="flex gap-2">
                      <Button onClick={handleCopyExport} variant="outline">
                        {copied ? <Check className="h-4 w-4 mr-2" /> : <Copy className="h-4 w-4 mr-2" />}
                        {copied ? 'Copied!' : 'Copy to Clipboard'}
                      </Button>
                    </div>
                  </div>
                )}
              </div>

              {/* Import Section */}
              <div className="space-y-4">
                <h3 className="text-lg font-medium">Import</h3>
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>
                    Importing permissions will modify this user's current permissions.
                  </AlertDescription>
                </Alert>
                
                <Input
                  type="file"
                  accept=".json"
                  ref={fileInputRef}
                  className="mb-2"
                />
                
                <Textarea
                  placeholder="Or paste permission JSON data here..."
                  value={importData}
                  onChange={(e) => setImportData(e.target.value)}
                  className="font-mono text-xs min-h-[150px]"
                />
                
                <Button disabled={!importData.trim()}>
                  <Upload className="h-4 w-4 mr-2" />
                  Import Permissions
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}