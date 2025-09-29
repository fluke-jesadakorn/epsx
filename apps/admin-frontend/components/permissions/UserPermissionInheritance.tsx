/**
 * User Permission Inheritance Component
 * Displays and manages permission inheritance hierarchies for users
 */

'use client'

import React, { useState, useCallback, useMemo } from 'react'
import { 
  GitBranch, Shield, User, Clock, AlertTriangle, CheckCircle, 
  Eye, EyeOff, ChevronDown, ChevronRight, Star, Crown, Key
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { 
  Collapsible, CollapsibleContent, CollapsibleTrigger 
} from '@/components/ui/collapsible'

import { adminCardVariants } from '@/design-system'
import { cn } from '@/lib/shared'

export interface PermissionSource {
  id: string;
  type: 'direct' | 'group' | 'role' | 'inherited';
  name: string;
  description?: string;
  permissions: string[];
  priority: number;
  expiresAt?: string;
  grantedBy?: string;
  grantedAt: string;
}

export interface UserPermissionInheritanceData {
  userId: string;
  userName: string;
  email: string;
  sources: PermissionSource[];
  effectivePermissions: string[];
  conflictingPermissions: string[];
  expiredPermissions: string[];
}

interface UserPermissionInheritanceProps {
  data: UserPermissionInheritanceData;
  onRefresh?: () => void;
  className?: string;
}

export function UserPermissionInheritance({ 
  data, 
  onRefresh, 
  className 
}: UserPermissionInheritanceProps) {
  const [expandedSources, setExpandedSources] = useState<Set<string>>(new Set())
  const [showExpired, setShowExpired] = useState(false)
  const [activeTab, setActiveTab] = useState('overview')

  const toggleSourceExpansion = useCallback((sourceId: string) => {
    setExpandedSources(prev => {
      const newSet = new Set(prev)
      if (newSet.has(sourceId)) {
        newSet.delete(sourceId)
      } else {
        newSet.add(sourceId)
      }
      return newSet
    })
  }, [])

  // Group permissions by type
  const groupedSources = useMemo(() => {
    const groups = {
      direct: [] as PermissionSource[],
      group: [] as PermissionSource[],
      role: [] as PermissionSource[],
      inherited: [] as PermissionSource[]
    }

    data.sources.forEach(source => {
      groups[source.type].push(source)
    })

    // Sort by priority (higher first)
    Object.values(groups).forEach(group => {
      group.sort((a, b) => b.priority - a.priority)
    })

    return groups
  }, [data.sources])

  // Calculate permission conflicts
  const permissionConflicts = useMemo(() => {
    const permissionMap = new Map<string, PermissionSource[]>()
    
    data.sources.forEach(source => {
      source.permissions.forEach(permission => {
        if (!permissionMap.has(permission)) {
          permissionMap.set(permission, [])
        }
        permissionMap.get(permission)!.push(source)
      })
    })

    const conflicts: Array<{
      permission: string;
      sources: PermissionSource[];
      resolution: 'highest_priority' | 'most_recent' | 'direct_override';
    }> = []

    permissionMap.forEach((sources, permission) => {
      if (sources.length > 1) {
        // Determine conflict resolution strategy
        const hasDirectSource = sources.some(s => s.type === 'direct')
        const resolution = hasDirectSource 
          ? 'direct_override' 
          : sources[0].priority > sources[1].priority 
            ? 'highest_priority' 
            : 'most_recent'

        conflicts.push({
          permission,
          sources,
          resolution
        })
      }
    })

    return conflicts
  }, [data.sources])

  const getSourceIcon = (type: PermissionSource['type']) => {
    switch (type) {
      case 'direct': return <User className="h-4 w-4 text-blue-600" />
      case 'group': return <Shield className="h-4 w-4 text-green-600" />
      case 'role': return <Crown className="h-4 w-4 text-purple-600" />
      case 'inherited': return <GitBranch className="h-4 w-4 text-orange-600" />
      default: return <Key className="h-4 w-4 text-gray-600" />
    }
  }

  const getSourceBadgeVariant = (type: PermissionSource['type']) => {
    switch (type) {
      case 'direct': return 'default'
      case 'group': return 'secondary'
      case 'role': return 'outline'
      case 'inherited': return 'destructive'
      default: return 'secondary'
    }
  }

  const isExpired = (source: PermissionSource) => {
    return source.expiresAt && new Date(source.expiresAt) < new Date()
  }

  const renderPermissionSource = (source: PermissionSource) => {
    const isExpanded = expandedSources.has(source.id)
    const expired = isExpired(source)
    
    if (expired && !showExpired) return null

    return (
      <Card 
        key={source.id} 
        className={cn(
          adminCardVariants({ variant: 'default' }),
          expired ? 'opacity-60 border-red-200' : ''
        )}
      >
        <Collapsible>
          <CollapsibleTrigger asChild>
            <CardHeader 
              className="cursor-pointer hover:bg-gray-50 transition-colors"
              onClick={() => toggleSourceExpansion(source.id)}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  {getSourceIcon(source.type)}
                  <div>
                    <CardTitle className="text-base">{source.name}</CardTitle>
                    {source.description && (
                      <p className="text-sm text-gray-600 mt-1">{source.description}</p>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant={getSourceBadgeVariant(source.type)}>
                    {source.type}
                  </Badge>
                  <Badge variant="outline">
                    Priority: {source.priority}
                  </Badge>
                  <Badge variant="outline">
                    {source.permissions.length} perms
                  </Badge>
                  {expired && (
                    <Badge variant="destructive">Expired</Badge>
                  )}
                  {isExpanded ? (
                    <ChevronDown className="h-4 w-4" />
                  ) : (
                    <ChevronRight className="h-4 w-4" />
                  )}
                </div>
              </div>
            </CardHeader>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <CardContent className="pt-0">
              <div className="space-y-4">
                {/* Source Metadata */}
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-3 bg-gray-50 rounded-lg">
                  <div>
                    <p className="text-xs font-medium text-gray-600">Granted By</p>
                    <p className="text-sm">{source.grantedBy || 'System'}</p>
                  </div>
                  <div>
                    <p className="text-xs font-medium text-gray-600">Granted At</p>
                    <p className="text-sm">
                      {new Date(source.grantedAt).toLocaleDateString()}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs font-medium text-gray-600">Expires At</p>
                    <p className="text-sm">
                      {source.expiresAt 
                        ? new Date(source.expiresAt).toLocaleDateString()
                        : 'Never'
                      }
                    </p>
                  </div>
                  <div>
                    <p className="text-xs font-medium text-gray-600">Status</p>
                    <div className="flex items-center gap-1">
                      {expired ? (
                        <>
                          <AlertTriangle className="h-3 w-3 text-red-600" />
                          <span className="text-sm text-red-600">Expired</span>
                        </>
                      ) : (
                        <>
                          <CheckCircle className="h-3 w-3 text-green-600" />
                          <span className="text-sm text-green-600">Active</span>
                        </>
                      )}
                    </div>
                  </div>
                </div>

                {/* Permissions List */}
                <div>
                  <h4 className="font-medium text-sm mb-2">Permissions</h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {source.permissions.map((permission, index) => (
                      <div 
                        key={index}
                        className="flex items-center justify-between p-2 bg-white border rounded text-sm"
                      >
                        <span>{permission}</span>
                        {data.conflictingPermissions.includes(permission) && (
                          <Badge variant="destructive" className="text-xs">
                            Conflict
                          </Badge>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </CollapsibleContent>
        </Collapsible>
      </Card>
    )
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h2 className="text-xl font-semibold text-gray-900">Permission Inheritance</h2>
          <p className="text-sm text-gray-600 mt-1">
            {data.userName} ({data.email})
          </p>
        </div>
        <div className="flex gap-2">
          <Button 
            variant="outline" 
            size="sm"
            onClick={() => setShowExpired(!showExpired)}
          >
            {showExpired ? <EyeOff className="h-4 w-4 mr-2" /> : <Eye className="h-4 w-4 mr-2" />}
            {showExpired ? 'Hide' : 'Show'} Expired
          </Button>
          {onRefresh && (
            <Button variant="outline" size="sm" onClick={onRefresh}>
              Refresh
            </Button>
          )}
        </div>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Effective Permissions</p>
                <p className="text-2xl font-bold text-gray-900">{data.effectivePermissions.length}</p>
              </div>
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Permission Sources</p>
                <p className="text-2xl font-bold text-gray-900">{data.sources.length}</p>
              </div>
              <GitBranch className="h-8 w-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Conflicts</p>
                <p className="text-2xl font-bold text-gray-900">{permissionConflicts.length}</p>
              </div>
              <AlertTriangle className="h-8 w-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Expired</p>
                <p className="text-2xl font-bold text-gray-900">{data.expiredPermissions.length}</p>
              </div>
              <Clock className="h-8 w-8 text-red-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Alerts */}
      {data.conflictingPermissions.length > 0 && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {data.conflictingPermissions.length} permission conflicts detected. 
            Review the conflicts tab for resolution details.
          </AlertDescription>
        </Alert>
      )}

      {data.expiredPermissions.length > 0 && (
        <Alert>
          <Clock className="h-4 w-4" />
          <AlertDescription>
            {data.expiredPermissions.length} permissions have expired and should be cleaned up.
          </AlertDescription>
        </Alert>
      )}

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="sources">Sources</TabsTrigger>
          <TabsTrigger value="conflicts">Conflicts</TabsTrigger>
          <TabsTrigger value="effective">Effective</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {Object.entries(groupedSources).map(([type, sources]) => (
              <Card key={type} className={adminCardVariants({ variant: 'default' })}>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 capitalize">
                    {getSourceIcon(type as PermissionSource['type'])}
                    {type} Sources ({sources.length})
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {sources.length === 0 ? (
                    <p className="text-sm text-gray-500">No {type} sources</p>
                  ) : (
                    <div className="space-y-2">
                      {sources.map(source => (
                        <div key={source.id} className="flex items-center justify-between p-2 bg-gray-50 rounded">
                          <span className="text-sm font-medium">{source.name}</span>
                          <Badge variant="outline" className="text-xs">
                            {source.permissions.length} perms
                          </Badge>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="sources" className="space-y-4">
          <div className="space-y-4">
            {data.sources.map(renderPermissionSource)}
          </div>
        </TabsContent>

        <TabsContent value="conflicts" className="space-y-4">
          {permissionConflicts.length === 0 ? (
            <Card className={adminCardVariants({ variant: 'default' })}>
              <CardContent className="py-8 text-center">
                <CheckCircle className="h-12 w-12 text-green-600 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No Conflicts</h3>
                <p className="text-gray-600">All permissions are resolved without conflicts.</p>
              </CardContent>
            </Card>
          ) : (
            permissionConflicts.map((conflict, index) => (
              <Card key={index} className={adminCardVariants({ variant: 'default' })}>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <AlertTriangle className="h-5 w-5 text-orange-600" />
                    {conflict.permission}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-3">
                    <div>
                      <p className="text-sm font-medium text-gray-900">Conflicting Sources:</p>
                      <div className="space-y-2 mt-1">
                        {conflict.sources.map(source => (
                          <div key={source.id} className="flex items-center justify-between p-2 bg-gray-50 rounded">
                            <div className="flex items-center gap-2">
                              {getSourceIcon(source.type)}
                              <span className="text-sm">{source.name}</span>
                            </div>
                            <Badge variant="outline">Priority: {source.priority}</Badge>
                          </div>
                        ))}
                      </div>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-900">Resolution Strategy:</p>
                      <Badge variant="secondary" className="mt-1">
                        {conflict.resolution.replace('_', ' ').toUpperCase()}
                      </Badge>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))
          )}
        </TabsContent>

        <TabsContent value="effective" className="space-y-4">
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle>Effective Permissions ({data.effectivePermissions.length})</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
                {data.effectivePermissions.map((permission, index) => (
                  <div key={index} className="p-2 bg-green-50 border border-green-200 rounded text-sm">
                    {permission}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}

export default UserPermissionInheritance