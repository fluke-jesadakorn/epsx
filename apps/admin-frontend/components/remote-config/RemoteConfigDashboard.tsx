'use client'

/**
 * Remote Config Dashboard - Main Admin Interface
 * Provides comprehensive management of Firebase Remote Config parameters
 */

import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui'
import { logger } from '@/lib/logger'
import { 
  Settings, 
  Plus, 
  RefreshCw, 
  Eye, 
  Edit, 
  Trash2, 
  Users, 
  Flag, 
  BarChart3, 
  TestTube, 
  AlertTriangle,
  CheckCircle,
  Clock,
  Activity
} from 'lucide-react'
import { 
  PARAMETER_CATEGORIES, 
  DEFAULT_PARAMETERS, 
  COMMON_CONDITIONS,
  getAllParametersByCategory,
  getParameterCategory,
  formatParameterValue,
  type RemoteConfigParameter,
  type UserCondition
} from '@/lib/admin-remote-config'

interface RemoteConfigDashboardProps {
  // Props for integration with Firebase Admin SDK
  projectId?: string
  className?: string
}

export function RemoteConfigDashboard({ projectId, className }: RemoteConfigDashboardProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [parameters, setParameters] = useState<Record<string, RemoteConfigParameter>>(DEFAULT_PARAMETERS)
  const [conditions, setConditions] = useState<UserCondition[]>(COMMON_CONDITIONS)
  const [lastModified, setLastModified] = useState<Date | null>(null)
  const [etag, setEtag] = useState<string>('')
  const [activeTab, setActiveTab] = useState('overview')
  const [error, setError] = useState<string | null>(null)

  // Mock data loading for now
  useEffect(() => {
    loadRemoteConfig()
  }, [])

  const loadRemoteConfig = async () => {
    try {
      setIsLoading(true)
      setError(null)
      
      // TODO: Replace with actual Firebase Admin SDK calls
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      setParameters(DEFAULT_PARAMETERS)
      setConditions(COMMON_CONDITIONS)
      setLastModified(new Date())
      setEtag('mock-etag-' + Date.now())
      
    } catch (err) {
      logger.error('Error loading Remote Config', { error: err })
      setError(err instanceof Error ? err.message : 'Failed to load configuration')
    } finally {
      setIsLoading(false)
    }
  }

  const handlePublishChanges = async () => {
    try {
      setIsLoading(true)
      // TODO: Implement Firebase Admin SDK publish
      await new Promise(resolve => setTimeout(resolve, 2000))
      setLastModified(new Date())
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to publish changes')
    } finally {
      setIsLoading(false)
    }
  }

  const getTotalParameterCount = () => Object.keys(parameters).length
  const getActiveParameterCount = () => Object.values(parameters).filter(p => p.value).length

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Remote Config Management</h1>
          <p className="text-muted-foreground">
            Manage user settings and feature flags remotely
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Button
            onClick={loadRemoteConfig}
            disabled={isLoading}
            variant="outline"
            size="sm"
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button
            onClick={handlePublishChanges}
            disabled={isLoading}
            size="sm"
          >
            <CheckCircle className="h-4 w-4 mr-2" />
            Publish Changes
          </Button>
        </div>
      </div>

      {/* Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Settings className="h-4 w-4 text-blue-600" />
              <div>
                <div className="text-2xl font-bold">{getTotalParameterCount()}</div>
                <div className="text-xs text-muted-foreground">Total Parameters</div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-4 w-4 text-green-600" />
              <div>
                <div className="text-2xl font-bold">{getActiveParameterCount()}</div>
                <div className="text-xs text-muted-foreground">Active Parameters</div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Users className="h-4 w-4 text-purple-600" />
              <div>
                <div className="text-2xl font-bold">{conditions.length}</div>
                <div className="text-xs text-muted-foreground">User Conditions</div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Clock className="h-4 w-4 text-orange-600" />
              <div>
                <div className="text-sm font-bold">
                  {lastModified ? lastModified.toLocaleDateString() : 'Never'}
                </div>
                <div className="text-xs text-muted-foreground">Last Published</div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Error Display */}
      {error && (
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-red-600">
              <AlertTriangle className="h-4 w-4" />
              <span className="font-medium">Error:</span>
              <span>{error}</span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Main Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
        <TabsList className="grid w-full grid-cols-6">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="parameters">Parameters</TabsTrigger>
          <TabsTrigger value="conditions">Conditions</TabsTrigger>
          <TabsTrigger value="experiments">A/B Tests</TabsTrigger>
          <TabsTrigger value="analytics">Analytics</TabsTrigger>
          <TabsTrigger value="history">History</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview">
          <ParameterOverview parameters={parameters} />
        </TabsContent>

        {/* Parameters Tab */}
        <TabsContent value="parameters">
          <ParameterManagement 
            parameters={parameters}
            setParameters={setParameters}
            isLoading={isLoading}
          />
        </TabsContent>

        {/* Conditions Tab */}
        <TabsContent value="conditions">
          <ConditionsManagement 
            conditions={conditions}
            setConditions={setConditions}
            isLoading={isLoading}
          />
        </TabsContent>

        {/* A/B Tests Tab */}
        <TabsContent value="experiments">
          <ExperimentsManagement parameters={parameters} />
        </TabsContent>

        {/* Analytics Tab */}
        <TabsContent value="analytics">
          <ConfigAnalytics parameters={parameters} />
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history">
          <ConfigHistory />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// ============================================================================
// Sub Components
// ============================================================================

function ParameterOverview({ parameters }: { parameters: Record<string, RemoteConfigParameter> }) {
  const categorizedParameters = getAllParametersByCategory()

  return (
    <div className="grid gap-6">
      {Object.entries(PARAMETER_CATEGORIES).map(([categoryKey, categoryInfo]) => (
        <Card key={categoryKey}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <div 
                className={`w-3 h-3 rounded-full bg-${categoryInfo.color}-500`}
                style={{ backgroundColor: `var(--${categoryInfo.color}-500)` }}
              />
              {categoryInfo.name}
              <Badge variant="outline">
                {categoryInfo.parameters.length} parameters
              </Badge>
            </CardTitle>
            <p className="text-sm text-muted-foreground">
              {categoryInfo.description}
            </p>
          </CardHeader>
          <CardContent>
            <div className="grid gap-2">
              {categoryInfo.parameters.map(paramKey => {
                const param = parameters[paramKey]
                if (!param) return null
                
                return (
                  <div key={paramKey} className="flex items-center justify-between p-2 border rounded">
                    <div>
                      <div className="font-medium text-sm">{param.key}</div>
                      <div className="text-xs text-muted-foreground">{param.description}</div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm font-mono">{formatParameterValue(param)}</div>
                      <div className="text-xs text-muted-foreground">{param.valueType}</div>
                    </div>
                  </div>
                )
              })}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}

function ParameterManagement({ 
  parameters, 
  setParameters, 
  isLoading 
}: {
  parameters: Record<string, RemoteConfigParameter>
  setParameters: (params: Record<string, RemoteConfigParameter>) => void
  isLoading: boolean
}) {
  const [filter, setFilter] = useState('')

  const filteredParameters = Object.values(parameters).filter(param =>
    param.key.toLowerCase().includes(filter.toLowerCase()) ||
    param.description.toLowerCase().includes(filter.toLowerCase())
  )

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">Parameter Management</h3>
          <p className="text-sm text-muted-foreground">
            Create, edit, and manage Remote Config parameters
          </p>
        </div>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Add Parameter
        </Button>
      </div>

      <Card>
        <CardContent className="p-6">
          <div className="space-y-4">
            <input
              type="text"
              placeholder="Filter parameters..."
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              className="w-full p-2 border rounded"
            />

            <div className="space-y-2">
              {filteredParameters.map(param => {
                const category = getParameterCategory(param.key)
                
                return (
                  <div key={param.key} className="flex items-center justify-between p-3 border rounded">
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{param.key}</span>
                        {category && (
                          <Badge 
                            variant="outline" 
                            className={`text-${category.info.color}-700 border-${category.info.color}-300`}
                          >
                            {category.info.name}
                          </Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground">{param.description}</p>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-mono bg-muted px-2 py-1 rounded">
                        {formatParameterValue(param)}
                      </span>
                      <Button size="sm" variant="outline">
                        <Edit className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function ConditionsManagement({
  conditions,
  setConditions,
  isLoading
}: {
  conditions: UserCondition[]
  setConditions: (conditions: UserCondition[]) => void
  isLoading: boolean
}) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">User Targeting Conditions</h3>
          <p className="text-sm text-muted-foreground">
            Define conditions for targeting specific user groups
          </p>
        </div>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Add Condition
        </Button>
      </div>

      <div className="grid gap-4">
        {conditions.map(condition => (
          <Card key={condition.name}>
            <CardContent className="p-4">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <h4 className="font-medium">{condition.name}</h4>
                    {condition.tagColor && (
                      <Badge 
                        variant="outline"
                        className={`bg-${condition.tagColor}-100 text-${condition.tagColor}-800`}
                      >
                        {condition.tagColor}
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm text-muted-foreground mb-2">{condition.description}</p>
                  <code className="text-xs bg-muted p-2 rounded block">
                    {condition.expression}
                  </code>
                </div>
                <div className="flex gap-2">
                  <Button size="sm" variant="outline">
                    <Edit className="h-3 w-3" />
                  </Button>
                  <Button size="sm" variant="outline">
                    <Trash2 className="h-3 w-3" />
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}

function ExperimentsManagement({ parameters }: { parameters: Record<string, RemoteConfigParameter> }) {
  const abTestParameters = Object.values(parameters).filter(p => p.key.startsWith('ab_'))

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">A/B Testing Experiments</h3>
          <p className="text-sm text-muted-foreground">
            Manage experiments and feature rollouts
          </p>
        </div>
        <Button>
          <TestTube className="h-4 w-4 mr-2" />
          Create Experiment
        </Button>
      </div>

      <div className="grid gap-4">
        {abTestParameters.map(param => (
          <Card key={param.key}>
            <CardContent className="p-4">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="font-medium">{param.key.replace('ab_', '').replace('_', ' ')}</h4>
                  <p className="text-sm text-muted-foreground">{param.description}</p>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">Active</Badge>
                  <span className="text-sm font-mono">{String(param.value)}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}

function ConfigAnalytics({ parameters }: { parameters: Record<string, RemoteConfigParameter> }) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium">Configuration Analytics</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Parameter Usage</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Analytics coming soon...
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>User Impact</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Impact analysis coming soon...
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

function ConfigHistory() {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium">Configuration History</h3>
      
      <Card>
        <CardContent className="p-6">
          <p className="text-sm text-muted-foreground text-center">
            No configuration history available
          </p>
        </CardContent>
      </Card>
    </div>
  )
}