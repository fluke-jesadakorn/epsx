'use client'

import { useState, useEffect } from 'react'
import { Play, Pause, Square, RefreshCw, Download, Upload, AlertTriangle, CheckCircle, XCircle, Info, Database, Users, Shield, ArrowRight } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Progress } from '@/components/ui/progress'
import { Separator } from '@/components/ui/separator'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ScrollArea } from '@/components/ui/scroll-area'

// Types
interface LegacyPermission {
  id: string
  permission_string: string
  user_count: number
  last_used?: string
  suggested_mapping: {
    platform: string
    resource: string
    action: string
    confidence: 'high' | 'medium' | 'low'
    rbac_permission: string
  }
  conflicts: string[]
}

interface MigrationJob {
  id: string
  name: string
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
  progress: number
  created_at: string
  started_at?: string
  completed_at?: string
  config: {
    dry_run: boolean
    backup_first: boolean
    notify_users: boolean
    migration_strategy: 'conservative' | 'aggressive' | 'custom'
    batch_size: number
    selected_permissions: string[]
  }
  results?: {
    total_permissions: number
    migrated_successfully: number
    failed_migrations: number
    users_affected: number
    roles_created: number
    logs: MigrationLogEntry[]
  }
  error_message?: string
}

interface MigrationLogEntry {
  id: string
  timestamp: string
  level: 'info' | 'warning' | 'error'
  message: string
  details?: any
}

interface MigrationStats {
  legacy_permissions: {
    total: number
    mapped: number
    conflicts: number
    unmapped: number
  }
  users: {
    total_affected: number
    with_legacy_permissions: number
    estimated_role_assignments: number
  }
  confidence_breakdown: {
    high: number
    medium: number
    low: number
  }
}

export default function LegacyMigrationTool() {
  const [activeTab, setActiveTab] = useState<'analyze' | 'migrate' | 'monitor'>('analyze')
  const [legacyPermissions, setLegacyPermissions] = useState<LegacyPermission[]>([])
  const [migrationJobs, setMigrationJobs] = useState<MigrationJob[]>([])
  const [stats, setStats] = useState<MigrationStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [analyzing, setAnalyzing] = useState(false)
  
  // Migration form state
  const [migrationConfig, setMigrationConfig] = useState({
    name: '',
    dry_run: true,
    backup_first: true,
    notify_users: false,
    migration_strategy: 'conservative' as 'conservative' | 'aggressive' | 'custom',
    batch_size: 100,
    selected_permissions: [] as string[]
  })

  // Mock API functions - replace with real API calls
  const mockAnalyzeLegacyPermissions = async (): Promise<{
    permissions: LegacyPermission[]
    stats: MigrationStats
  }> => {
    // Simulate analysis time
    await new Promise(resolve => setTimeout(resolve, 3000))
    
    const permissions: LegacyPermission[] = [
      {
        id: '1',
        permission_string: 'view_eps',
        user_count: 45,
        last_used: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(),
        suggested_mapping: {
          platform: 'epsx',
          resource: 'analytics',
          action: 'view',
          confidence: 'high',
          rbac_permission: 'epsx:analytics:view'
        },
        conflicts: []
      },
      {
        id: '2',
        permission_string: 'export_data',
        user_count: 28,
        last_used: new Date(Date.now() - 1000 * 60 * 60 * 6).toISOString(),
        suggested_mapping: {
          platform: 'epsx',
          resource: 'analytics',
          action: 'export',
          confidence: 'high',
          rbac_permission: 'epsx:analytics:export'
        },
        conflicts: []
      },
      {
        id: '3',
        permission_string: 'admin',
        user_count: 8,
        last_used: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
        suggested_mapping: {
          platform: 'admin',
          resource: '*',
          action: '*',
          confidence: 'high',
          rbac_permission: 'admin:*:*'
        },
        conflicts: []
      },
      {
        id: '4',
        permission_string: 'advanced_filters',
        user_count: 15,
        last_used: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
        suggested_mapping: {
          platform: 'epsx',
          resource: 'analytics',
          action: 'advanced',
          confidence: 'medium',
          rbac_permission: 'epsx:analytics:advanced'
        },
        conflicts: ['Could also map to epsx:filters:use']
      },
      {
        id: '5',
        permission_string: 'temp_access_2024',
        user_count: 3,
        last_used: new Date(Date.now() - 1000 * 60 * 60 * 24 * 60).toISOString(),
        suggested_mapping: {
          platform: 'epsx',
          resource: 'unknown',
          action: 'access',
          confidence: 'low',
          rbac_permission: 'epsx:unknown:access'
        },
        conflicts: ['Appears to be a temporary permission', 'May be expired or obsolete']
      }
    ]
    
    const stats: MigrationStats = {
      legacy_permissions: {
        total: permissions.length,
        mapped: permissions.filter(p => p.suggested_mapping.confidence !== 'low').length,
        conflicts: permissions.filter(p => p.conflicts.length > 0).length,
        unmapped: permissions.filter(p => p.suggested_mapping.confidence === 'low').length
      },
      users: {
        total_affected: permissions.reduce((sum, p) => sum + p.user_count, 0),
        with_legacy_permissions: 73,
        estimated_role_assignments: 45
      },
      confidence_breakdown: {
        high: permissions.filter(p => p.suggested_mapping.confidence === 'high').length,
        medium: permissions.filter(p => p.suggested_mapping.confidence === 'medium').length,
        low: permissions.filter(p => p.suggested_mapping.confidence === 'low').length
      }
    }
    
    return { permissions, stats }
  }

  const mockStartMigration = async (config: typeof migrationConfig): Promise<MigrationJob> => {
    const job: MigrationJob = {
      id: Math.random().toString(36).substring(7),
      name: config.name,
      status: 'pending',
      progress: 0,
      created_at: new Date().toISOString(),
      config
    }
    
    return job
  }

  const mockGetMigrationJobs = async (): Promise<MigrationJob[]> => {
    return [
      {
        id: 'job1',
        name: 'Initial RBAC Migration - Dry Run',
        status: 'completed',
        progress: 100,
        created_at: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
        started_at: new Date(Date.now() - 1000 * 60 * 60 * 24 + 1000 * 60 * 2).toISOString(),
        completed_at: new Date(Date.now() - 1000 * 60 * 60 * 23).toISOString(),
        config: {
          dry_run: true,
          backup_first: true,
          notify_users: false,
          migration_strategy: 'conservative',
          batch_size: 50,
          selected_permissions: ['view_eps', 'export_data', 'admin']
        },
        results: {
          total_permissions: 3,
          migrated_successfully: 3,
          failed_migrations: 0,
          users_affected: 81,
          roles_created: 4,
          logs: [
            {
              id: '1',
              timestamp: new Date(Date.now() - 1000 * 60 * 60 * 23.5).toISOString(),
              level: 'info',
              message: 'Migration started with 3 permissions',
              details: { total: 3, batch_size: 50 }
            },
            {
              id: '2',
              timestamp: new Date(Date.now() - 1000 * 60 * 60 * 23.4).toISOString(),
              level: 'info',
              message: 'Created role: analytics_user',
              details: { role: 'analytics_user', permissions: ['epsx:analytics:view'] }
            },
            {
              id: '3',
              timestamp: new Date(Date.now() - 1000 * 60 * 60 * 23).toISOString(),
              level: 'info',
              message: 'Migration completed successfully',
              details: { migrated: 3, failed: 0 }
            }
          ]
        }
      },
      {
        id: 'job2',
        name: 'Production Migration - Conservative',
        status: 'running',
        progress: 65,
        created_at: new Date(Date.now() - 1000 * 60 * 15).toISOString(),
        started_at: new Date(Date.now() - 1000 * 60 * 10).toISOString(),
        config: {
          dry_run: false,
          backup_first: true,
          notify_users: true,
          migration_strategy: 'conservative',
          batch_size: 25,
          selected_permissions: ['view_eps', 'export_data', 'admin', 'advanced_filters']
        }
      }
    ]
  }

  useEffect(() => {
    loadInitialData()
  }, [])

  const loadInitialData = async () => {
    setLoading(true)
    try {
      const [analysisResult, jobs] = await Promise.all([
        mockAnalyzeLegacyPermissions(),
        mockGetMigrationJobs()
      ])
      
      setLegacyPermissions(analysisResult.permissions)
      setStats(analysisResult.stats)
      setMigrationJobs(jobs)
    } catch (error) {
      console.error('Failed to load migration data:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleAnalyze = async () => {
    setAnalyzing(true)
    try {
      const result = await mockAnalyzeLegacyPermissions()
      setLegacyPermissions(result.permissions)
      setStats(result.stats)
    } catch (error) {
      console.error('Analysis failed:', error)
    } finally {
      setAnalyzing(false)
    }
  }

  const handleStartMigration = async () => {
    if (!migrationConfig.name) return
    
    try {
      const job = await mockStartMigration(migrationConfig)
      setMigrationJobs(prev => [job, ...prev])
      setActiveTab('monitor')
      
      // Reset form
      setMigrationConfig({
        name: '',
        dry_run: true,
        backup_first: true,
        notify_users: false,
        migration_strategy: 'conservative',
        batch_size: 100,
        selected_permissions: []
      })
    } catch (error) {
      console.error('Failed to start migration:', error)
    }
  }

  const getConfidenceColor = (confidence: string) => {
    switch (confidence) {
      case 'high': return 'bg-green-100 text-green-800'
      case 'medium': return 'bg-yellow-100 text-yellow-800'
      case 'low': return 'bg-red-100 text-red-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  const getJobStatusColor = (status: MigrationJob['status']) => {
    switch (status) {
      case 'pending': return 'bg-blue-100 text-blue-800'
      case 'running': return 'bg-orange-100 text-orange-800'
      case 'completed': return 'bg-green-100 text-green-800'
      case 'failed': return 'bg-red-100 text-red-800'
      case 'cancelled': return 'bg-gray-100 text-gray-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  const getJobIcon = (status: MigrationJob['status']) => {
    switch (status) {
      case 'pending': return <Info className="w-4 h-4" />
      case 'running': return <Play className="w-4 h-4" />
      case 'completed': return <CheckCircle className="w-4 h-4" />
      case 'failed': return <XCircle className="w-4 h-4" />
      case 'cancelled': return <Square className="w-4 h-4" />
      default: return <Info className="w-4 h-4" />
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="flex items-center space-x-2">
          <RefreshCw className="h-6 w-6 animate-spin" />
          <span>Loading migration tools...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <h2 className="text-3xl font-bold">Legacy Migration Tool</h2>
          <p className="text-muted-foreground">Convert string-based permissions to structured RBAC system</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Download className="w-4 h-4 mr-2" />
            Export Analysis
          </Button>
          <Button variant="outline">
            <Upload className="w-4 h-4 mr-2" />
            Import Config
          </Button>
        </div>
      </div>

      {/* Overview Stats */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Legacy Permissions</CardTitle>
              <Database className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.legacy_permissions.total}</div>
              <p className="text-xs text-muted-foreground">
                {stats.legacy_permissions.conflicts} with conflicts
              </p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Affected Users</CardTitle>
              <Users className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.users.with_legacy_permissions}</div>
              <p className="text-xs text-muted-foreground">
                ~{stats.users.estimated_role_assignments} role assignments
              </p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">High Confidence</CardTitle>
              <Shield className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-600">{stats.confidence_breakdown.high}</div>
              <p className="text-xs text-muted-foreground">
                {((stats.confidence_breakdown.high / stats.legacy_permissions.total) * 100).toFixed(1)}% ready
              </p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Migration Progress</CardTitle>
              <RefreshCw className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.legacy_permissions.mapped}</div>
              <p className="text-xs text-muted-foreground">
                of {stats.legacy_permissions.total} mapped
              </p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Main Tabs */}
      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as any)}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="analyze">Analyze Legacy</TabsTrigger>
          <TabsTrigger value="migrate">Configure Migration</TabsTrigger>
          <TabsTrigger value="monitor">Monitor Jobs</TabsTrigger>
        </TabsList>

        {/* Analyze Tab */}
        <TabsContent value="analyze" className="space-y-6">
          <div className="flex justify-between items-center">
            <div>
              <h3 className="text-lg font-semibold">Legacy Permission Analysis</h3>
              <p className="text-sm text-muted-foreground">Review existing permissions and suggested RBAC mappings</p>
            </div>
            <Button onClick={handleAnalyze} disabled={analyzing}>
              {analyzing ? (
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <RefreshCw className="w-4 h-4 mr-2" />
              )}
              {analyzing ? 'Analyzing...' : 'Re-analyze'}
            </Button>
          </div>

          {/* Confidence Distribution */}
          {stats && (
            <Card>
              <CardHeader>
                <CardTitle>Mapping Confidence Distribution</CardTitle>
                <CardDescription>How confident the system is about each permission mapping</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <Badge className="bg-green-100 text-green-800">High Confidence</Badge>
                      <span className="text-sm">Can migrate automatically</span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className="text-sm font-medium">{stats.confidence_breakdown.high}</span>
                      <div className="w-24 bg-muted rounded-full h-2">
                        <div 
                          className="bg-green-500 h-2 rounded-full"
                          style={{ width: `${(stats.confidence_breakdown.high / stats.legacy_permissions.total) * 100}%` }}
                        />
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <Badge className="bg-yellow-100 text-yellow-800">Medium Confidence</Badge>
                      <span className="text-sm">Review recommended</span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className="text-sm font-medium">{stats.confidence_breakdown.medium}</span>
                      <div className="w-24 bg-muted rounded-full h-2">
                        <div 
                          className="bg-yellow-500 h-2 rounded-full"
                          style={{ width: `${(stats.confidence_breakdown.medium / stats.legacy_permissions.total) * 100}%` }}
                        />
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <Badge className="bg-red-100 text-red-800">Low Confidence</Badge>
                      <span className="text-sm">Manual review required</span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className="text-sm font-medium">{stats.confidence_breakdown.low}</span>
                      <div className="w-24 bg-muted rounded-full h-2">
                        <div 
                          className="bg-red-500 h-2 rounded-full"
                          style={{ width: `${(stats.confidence_breakdown.low / stats.legacy_permissions.total) * 100}%` }}
                        />
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Permission Mappings */}
          <Card>
            <CardHeader>
              <CardTitle>Legacy Permissions</CardTitle>
              <CardDescription>Current permissions and their suggested RBAC mappings</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {legacyPermissions.map(permission => (
                  <div key={permission.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-3">
                      <div className="flex items-center space-x-3">
                        <code className="bg-muted px-2 py-1 rounded text-sm font-mono">
                          {permission.permission_string}
                        </code>
                        <Badge variant="outline">
                          {permission.user_count} users
                        </Badge>
                        <Badge className={getConfidenceColor(permission.suggested_mapping.confidence)}>
                          {permission.suggested_mapping.confidence} confidence
                        </Badge>
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Last used: {permission.last_used 
                          ? new Date(permission.last_used).toLocaleDateString()
                          : 'Never'
                        }
                      </div>
                    </div>
                    
                    <div className="flex items-center space-x-2 mb-2">
                      <span className="text-sm text-muted-foreground">Suggested mapping:</span>
                      <ArrowRight className="w-4 h-4 text-muted-foreground" />
                      <code className="bg-blue-50 px-2 py-1 rounded text-sm font-mono text-blue-800">
                        {permission.suggested_mapping.rbac_permission}
                      </code>
                    </div>
                    
                    {permission.conflicts.length > 0 && (
                      <Alert variant="destructive" className="mt-2">
                        <AlertTriangle className="h-4 w-4" />
                        <AlertDescription>
                          <div className="font-medium mb-1">Mapping Conflicts:</div>
                          <ul className="text-xs space-y-1">
                            {permission.conflicts.map((conflict, i) => (
                              <li key={i}>• {conflict}</li>
                            ))}
                          </ul>
                        </AlertDescription>
                      </Alert>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Migration Configuration Tab */}
        <TabsContent value="migrate" className="space-y-6">
          <div>
            <h3 className="text-lg font-semibold">Configure Migration</h3>
            <p className="text-sm text-muted-foreground">Set up parameters for the legacy permission migration</p>
          </div>

          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              <div className="font-medium mb-1">Migration Safety</div>
              Always start with a dry run to preview changes before running the actual migration. 
              Backups are automatically created when backup_first is enabled.
            </AlertDescription>
          </Alert>

          <Card>
            <CardHeader>
              <CardTitle>Migration Configuration</CardTitle>
              <CardDescription>Configure how the migration should be executed</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div>
                <Label htmlFor="migration-name">Migration Name</Label>
                <Input
                  id="migration-name"
                  value={migrationConfig.name}
                  onChange={(e) => setMigrationConfig(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="e.g., Production RBAC Migration"
                  className="mt-1"
                />
              </div>

              <div>
                <Label>Migration Strategy</Label>
                <Select 
                  value={migrationConfig.migration_strategy} 
                  onValueChange={(value: any) => setMigrationConfig(prev => ({ ...prev, migration_strategy: value }))}
                >
                  <SelectTrigger className="mt-1">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="conservative">
                      Conservative - Only migrate high-confidence mappings
                    </SelectItem>
                    <SelectItem value="aggressive">
                      Aggressive - Migrate all permissions with automatic fallbacks
                    </SelectItem>
                    <SelectItem value="custom">
                      Custom - Use selected permissions only
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="batch-size">Batch Size</Label>
                  <Select 
                    value={migrationConfig.batch_size.toString()} 
                    onValueChange={(value) => setMigrationConfig(prev => ({ ...prev, batch_size: parseInt(value) }))}
                  >
                    <SelectTrigger className="mt-1">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="25">25 - Slow but safe</SelectItem>
                      <SelectItem value="50">50 - Balanced</SelectItem>
                      <SelectItem value="100">100 - Fast</SelectItem>
                      <SelectItem value="500">500 - Bulk migration</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="space-y-3">
                <Label className="text-base font-medium">Migration Options</Label>
                <div className="space-y-3">
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id="dry-run"
                      checked={migrationConfig.dry_run}
                      onCheckedChange={(checked) => 
                        setMigrationConfig(prev => ({ ...prev, dry_run: checked as boolean }))
                      }
                    />
                    <Label htmlFor="dry-run" className="text-sm">
                      Dry Run - Preview changes without applying them
                    </Label>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id="backup-first"
                      checked={migrationConfig.backup_first}
                      onCheckedChange={(checked) => 
                        setMigrationConfig(prev => ({ ...prev, backup_first: checked as boolean }))
                      }
                    />
                    <Label htmlFor="backup-first" className="text-sm">
                      Create backup before migration
                    </Label>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id="notify-users"
                      checked={migrationConfig.notify_users}
                      onCheckedChange={(checked) => 
                        setMigrationConfig(prev => ({ ...prev, notify_users: checked as boolean }))
                      }
                    />
                    <Label htmlFor="notify-users" className="text-sm">
                      Notify users about permission changes
                    </Label>
                  </div>
                </div>
              </div>

              {migrationConfig.migration_strategy === 'custom' && (
                <div>
                  <Label className="text-base font-medium">Select Permissions to Migrate</Label>
                  <div className="mt-2 space-y-2 border rounded-lg p-3 max-h-48 overflow-y-auto">
                    {legacyPermissions.map(permission => (
                      <div key={permission.id} className="flex items-center space-x-2">
                        <Checkbox
                          id={`perm-${permission.id}`}
                          checked={migrationConfig.selected_permissions.includes(permission.permission_string)}
                          onCheckedChange={(checked) => {
                            if (checked) {
                              setMigrationConfig(prev => ({
                                ...prev,
                                selected_permissions: [...prev.selected_permissions, permission.permission_string]
                              }))
                            } else {
                              setMigrationConfig(prev => ({
                                ...prev,
                                selected_permissions: prev.selected_permissions.filter(p => p !== permission.permission_string)
                              }))
                            }
                          }}
                        />
                        <Label htmlFor={`perm-${permission.id}`} className="text-sm flex items-center space-x-2">
                          <code className="bg-muted px-1 rounded text-xs">{permission.permission_string}</code>
                          <Badge className={getConfidenceColor(permission.suggested_mapping.confidence)} variant="outline">
                            {permission.suggested_mapping.confidence}
                          </Badge>
                        </Label>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          <div className="flex justify-end space-x-2">
            <Button variant="outline" onClick={() => {
              setMigrationConfig({
                name: '',
                dry_run: true,
                backup_first: true,
                notify_users: false,
                migration_strategy: 'conservative',
                batch_size: 100,
                selected_permissions: []
              })
            }}>
              Reset
            </Button>
            <Button 
              onClick={handleStartMigration}
              disabled={!migrationConfig.name || (migrationConfig.migration_strategy === 'custom' && migrationConfig.selected_permissions.length === 0)}
            >
              <Play className="w-4 h-4 mr-2" />
              Start Migration
            </Button>
          </div>
        </TabsContent>

        {/* Monitor Tab */}
        <TabsContent value="monitor" className="space-y-6">
          <div>
            <h3 className="text-lg font-semibold">Migration Jobs</h3>
            <p className="text-sm text-muted-foreground">Monitor the progress and results of migration jobs</p>
          </div>

          <div className="space-y-4">
            {migrationJobs.map(job => (
              <Card key={job.id}>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      {getJobIcon(job.status)}
                      <CardTitle className="text-base">{job.name}</CardTitle>
                    </div>
                    <Badge className={getJobStatusColor(job.status)}>
                      {job.status}
                    </Badge>
                  </div>
                  <CardDescription>
                    Created: {new Date(job.created_at).toLocaleString()}
                    {job.started_at && ` • Started: ${new Date(job.started_at).toLocaleString()}`}
                    {job.completed_at && ` • Completed: ${new Date(job.completed_at).toLocaleString()}`}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* Progress for running jobs */}
                  {job.status === 'running' && (
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span>Progress</span>
                        <span>{job.progress}%</span>
                      </div>
                      <Progress value={job.progress} className="h-2" />
                    </div>
                  )}

                  {/* Job Configuration */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <div className="text-muted-foreground">Strategy</div>
                      <div className="font-medium capitalize">{job.config.migration_strategy}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Batch Size</div>
                      <div className="font-medium">{job.config.batch_size}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Dry Run</div>
                      <div className="font-medium">{job.config.dry_run ? 'Yes' : 'No'}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Notify Users</div>
                      <div className="font-medium">{job.config.notify_users ? 'Yes' : 'No'}</div>
                    </div>
                  </div>

                  {/* Results for completed jobs */}
                  {job.results && (
                    <div className="space-y-3">
                      <Separator />
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                        <div>
                          <div className="text-muted-foreground">Total Processed</div>
                          <div className="font-medium text-blue-600">{job.results.total_permissions}</div>
                        </div>
                        <div>
                          <div className="text-muted-foreground">Successful</div>
                          <div className="font-medium text-green-600">{job.results.migrated_successfully}</div>
                        </div>
                        <div>
                          <div className="text-muted-foreground">Failed</div>
                          <div className="font-medium text-red-600">{job.results.failed_migrations}</div>
                        </div>
                        <div>
                          <div className="text-muted-foreground">Roles Created</div>
                          <div className="font-medium text-purple-600">{job.results.roles_created}</div>
                        </div>
                      </div>

                      {/* Recent logs */}
                      {job.results.logs && job.results.logs.length > 0 && (
                        <div className="space-y-2">
                          <Label className="text-sm font-medium">Recent Activity</Label>
                          <ScrollArea className="h-32 border rounded-lg p-3">
                            <div className="space-y-1">
                              {job.results.logs.slice(-5).map(log => (
                                <div key={log.id} className="text-xs">
                                  <span className="text-muted-foreground">
                                    {new Date(log.timestamp).toLocaleTimeString()}
                                  </span>
                                  <span className={`ml-2 font-medium ${
                                    log.level === 'error' ? 'text-red-600' :
                                    log.level === 'warning' ? 'text-yellow-600' :
                                    'text-green-600'
                                  }`}>
                                    [{log.level.toUpperCase()}]
                                  </span>
                                  <span className="ml-1">{log.message}</span>
                                </div>
                              ))}
                            </div>
                          </ScrollArea>
                        </div>
                      )}
                    </div>
                  )}

                  {/* Error message for failed jobs */}
                  {job.error_message && (
                    <Alert variant="destructive">
                      <XCircle className="h-4 w-4" />
                      <AlertDescription>
                        <div className="font-medium">Migration Failed</div>
                        <div className="text-sm mt-1">{job.error_message}</div>
                      </AlertDescription>
                    </Alert>
                  )}

                  {/* Action buttons */}
                  <div className="flex justify-end space-x-2">
                    {job.status === 'running' && (
                      <Button variant="outline" size="sm">
                        <Pause className="w-4 h-4 mr-2" />
                        Pause
                      </Button>
                    )}
                    {(job.status === 'running' || job.status === 'pending') && (
                      <Button variant="destructive" size="sm">
                        <Square className="w-4 h-4 mr-2" />
                        Cancel
                      </Button>
                    )}
                    {job.status === 'completed' && (
                      <Button variant="outline" size="sm">
                        <Download className="w-4 h-4 mr-2" />
                        Download Report
                      </Button>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}

            {migrationJobs.length === 0 && (
              <div className="text-center py-12">
                <Database className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium text-muted-foreground mb-2">No migration jobs yet</h3>
                <p className="text-sm text-muted-foreground">
                  Start your first migration from the Configure tab to begin converting legacy permissions.
                </p>
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}