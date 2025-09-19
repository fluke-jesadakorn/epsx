'use client'

/**
 * Remote Config Dashboard - Main Admin Interface
 * Provides comprehensive management of Firebase Remote Config parameters
 */

import { useState, useEffect } from 'react'
import { Badge } from '@/components/ui'
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
  getCommonUserSegments,
  type RemoteConfigParameter,
  type UserCondition,
  type UserSegment
} from '@/lib/admin-remote-config'

interface RemoteConfigDashboardProps {
  // Props for integration with Firebase Admin SDK
  projectId?: string
  className?: string
  initialTab?: string
  initialFilter?: string
}

export function RemoteConfigDashboard({ projectId, className, initialTab = 'overview', initialFilter = '' }: RemoteConfigDashboardProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [parameters, setParameters] = useState<Record<string, RemoteConfigParameter>>(DEFAULT_PARAMETERS)
  const [conditions, setConditions] = useState<UserCondition[]>(COMMON_CONDITIONS)
  const [lastModified, setLastModified] = useState<Date | null>(null)
  const [etag, setEtag] = useState<string>('')
  const [activeTab, setActiveTab] = useState(initialTab)
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
    <div className="relative z-10">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>
      
      <div className={`space-y-8 ${className}`}>
        {/* Page Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 bg-clip-text text-transparent mb-4">
            Remote Config Management
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
            Manage Firebase Remote Config parameters, user targeting, and A/B testing experiments
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mt-6">
            <button
              onClick={loadRemoteConfig}
              disabled={isLoading}
              className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-2xl hover:from-blue-600 hover:to-purple-700 disabled:opacity-50 font-medium"
            >
              <RefreshCw className="h-4 w-4" />
              Refresh Config
            </button>
            <button
              onClick={handlePublishChanges}
              disabled={isLoading}
              className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-green-500 to-emerald-600 text-white rounded-2xl hover:from-green-600 hover:to-emerald-700 disabled:opacity-50 font-medium"
            >
              <CheckCircle className="h-4 w-4" />
              Publish Changes
            </button>
          </div>
        </div>

        {/* PancakeSwap Stats Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 via-orange-400/20 to-pink-400/20 p-0.5 group">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-8 h-8 bg-gradient-to-br from-yellow-300/30 to-orange-400/30 rounded-full blur-sm"></div>
              <div className="flex items-center justify-between mb-4">
                <div className="text-sm font-medium text-gray-600 dark:text-gray-400">Total</div>
                <div className="p-3 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-2xl">
                  <Settings className="h-6 w-6 text-orange-600" />
                </div>
              </div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{getTotalParameterCount()}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Parameters</div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5 group">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-8 h-8 bg-gradient-to-br from-green-300/30 to-emerald-400/30 rounded-full blur-sm"></div>
              <div className="flex items-center justify-between mb-4">
                <div className="text-sm font-medium text-gray-600 dark:text-gray-400">Active</div>
                <div className="p-3 bg-gradient-to-r from-green-400/20 to-emerald-500/20 rounded-2xl">
                  <CheckCircle className="h-6 w-6 text-green-600" />
                </div>
              </div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{getActiveParameterCount()}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Parameters</div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-red-400/20 p-0.5 group">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-8 h-8 bg-gradient-to-br from-purple-300/30 to-pink-400/30 rounded-full blur-sm"></div>
              <div className="flex items-center justify-between mb-4">
                <div className="text-sm font-medium text-gray-600 dark:text-gray-400">Users</div>
                <div className="p-3 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-2xl">
                  <Users className="h-6 w-6 text-purple-600" />
                </div>
              </div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{conditions.length}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Conditions</div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-cyan-400/20 p-0.5 group">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-8 h-8 bg-gradient-to-br from-blue-300/30 to-indigo-400/30 rounded-full blur-sm"></div>
              <div className="flex items-center justify-between mb-4">
                <div className="text-sm font-medium text-gray-600 dark:text-gray-400">Last</div>
                <div className="p-3 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-2xl">
                  <Clock className="h-6 w-6 text-blue-600" />
                </div>
              </div>
              <div className="text-xl font-bold text-gray-900 dark:text-white mb-1">
                {lastModified ? lastModified.toLocaleDateString() : 'Never'}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Published</div>
            </div>
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-red-400/20 via-pink-400/20 to-orange-400/20 p-0.5 mb-8">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="flex items-center gap-3 text-red-600">
                <div className="p-2 bg-red-100 dark:bg-red-900/20 rounded-xl">
                  <AlertTriangle className="h-5 w-5" />
                </div>
                <div>
                  <div className="font-semibold">Configuration Error</div>
                  <div className="text-sm text-gray-600 dark:text-gray-400">{error}</div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Navigation Tabs */}
        <div className="mb-8">
          <div className="flex flex-wrap gap-2 sm:gap-4 border-b border-gray-200/50 dark:border-gray-700/50 pb-4">
            {[
              { value: 'overview', label: 'Overview', icon: '📊' },
              { value: 'parameters', label: 'Parameters', icon: '⚙️' },
              { value: 'conditions', label: 'Conditions', icon: '🎯' },
              { value: 'targeting', label: 'Users', icon: '👥' },
              { value: 'experiments', label: 'A/B Tests', icon: '🧪' },
              { value: 'analytics', label: 'Analytics', icon: '📈' },
              { value: 'history', label: 'History', icon: '📝' }
            ].map(tab => (
              <button
                key={tab.value}
                onClick={() => setActiveTab(tab.value)}
                className={`flex items-center gap-2 px-4 py-2 rounded-2xl text-sm font-medium ${
                  activeTab === tab.value
                    ? 'bg-gradient-to-r from-yellow-400/20 to-orange-500/20 text-orange-700 dark:text-orange-300'
                    : 'text-gray-600 dark:text-gray-400 hover:text-orange-600 dark:hover:text-orange-400'
                }`}
              >
                <span>{tab.icon}</span>
                <span className="hidden sm:inline">{tab.label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Tab Content */}
        {activeTab === 'overview' && <ParameterOverview parameters={parameters} />}
        {activeTab === 'parameters' && (
          <ParameterManagement 
            parameters={parameters}
            setParameters={setParameters}
            isLoading={isLoading}
          />
        )}
        {activeTab === 'conditions' && (
          <ConditionsManagement 
            conditions={conditions}
            setConditions={setConditions}
            isLoading={isLoading}
          />
        )}
        {activeTab === 'targeting' && (
          <UserTargetingManagement 
            parameters={parameters}
            conditions={conditions}
            isLoading={isLoading}
          />
        )}
        {activeTab === 'experiments' && <ExperimentsManagement parameters={parameters} />}
        {activeTab === 'analytics' && <ConfigAnalytics parameters={parameters} />}
        {activeTab === 'history' && <ConfigHistory />}
      </div>
    </div>
  )
}

// ============================================================================
// Sub Components
// ============================================================================

function ParameterOverview({ parameters }: { parameters: Record<string, RemoteConfigParameter> }) {

  return (
    <div className="grid gap-6">
      {Object.entries(PARAMETER_CATEGORIES).map(([categoryKey, categoryInfo]) => (
        <div key={categoryKey} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
          <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
            <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-blue-300/30 to-purple-400/30 rounded-full blur-sm"></div>
            
            <div className="flex items-center gap-3 mb-4">
              <div 
                className={`w-4 h-4 rounded-full bg-${categoryInfo.color}-500`}
                style={{ backgroundColor: `var(--${categoryInfo.color}-500)` }}
              />
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{categoryInfo.name}</h3>
              <Badge className="bg-gradient-to-r from-yellow-400/20 to-orange-500/20 text-orange-700 dark:text-orange-300 border-orange-300/50">
                {categoryInfo.parameters.length} parameters
              </Badge>
            </div>
            
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-6">
              {categoryInfo.description}
            </p>
            
            <div className="grid gap-3">
              {categoryInfo.parameters.map(paramKey => {
                const param = parameters[paramKey]
                if (!param) return null
                
                return (
                  <div key={paramKey} className="flex flex-col lg:flex-row items-start lg:items-center justify-between p-4 bg-white/50 dark:bg-gray-800/50 rounded-2xl border border-gray-200/50 dark:border-gray-700/50 gap-3">
                    <div className="flex-1">
                      <div className="font-medium text-sm text-gray-900 dark:text-white">{param.key}</div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">{param.description}</div>
                    </div>
                    <div className="text-left lg:text-right w-full lg:w-auto">
                      <div className="text-sm font-mono bg-gradient-to-r from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600 px-3 py-2 rounded-xl text-gray-900 dark:text-white">
                        {formatParameterValue(param)}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">{param.valueType}</div>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        </div>
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
    <div className="space-y-6">
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
        <div>
          <h3 className="text-2xl font-bold text-gray-900 dark:text-white">Parameter Management</h3>
          <p className="text-gray-600 dark:text-gray-400">
            Create, edit, and manage Remote Config parameters
          </p>
        </div>
        <button className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-emerald-500 to-green-600 text-white rounded-2xl hover:from-emerald-600 hover:to-green-700 font-medium">
          <Plus className="h-4 w-4" />
          Add Parameter
        </button>
      </div>

      {/* Search Bar */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/10 to-purple-400/10 p-0.5">
        <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-4">
          <input
            type="text"
            placeholder="Filter parameters..."
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="w-full p-4 bg-white/50 dark:bg-gray-800/50 border border-gray-200/50 dark:border-gray-700/50 rounded-2xl text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-orange-500/50"
          />
        </div>
      </div>

      {/* Parameters Grid */}
      <div className="grid gap-4">
        {filteredParameters.map(param => {
          const category = getParameterCategory(param.key)
          
          return (
            <div key={param.key} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-red-400/20 p-0.5">
              <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
                <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-purple-300/30 to-pink-400/30 rounded-full blur-sm"></div>
                
                <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex flex-col sm:flex-row sm:items-center gap-2 mb-2">
                      <span className="font-semibold text-gray-900 dark:text-white">{param.key}</span>
                      {category && (
                        <Badge className="bg-gradient-to-r from-yellow-400/20 to-orange-500/20 text-orange-700 dark:text-orange-300 border-orange-300/50 w-fit">
                          {category.info.name}
                        </Badge>
                      )}
                    </div>
                    <p className="text-sm text-gray-600 dark:text-gray-400">{param.description}</p>
                  </div>
                  <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3">
                    <span className="text-sm font-mono bg-gradient-to-r from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600 px-4 py-2 rounded-xl text-gray-900 dark:text-white w-full sm:w-auto text-center">
                      {formatParameterValue(param)}
                    </span>
                    <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-blue-500 to-indigo-600 text-white rounded-xl hover:from-blue-600 hover:to-indigo-700 font-medium w-full sm:w-auto">
                      <Edit className="h-3 w-3" />
                      Edit
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )
        })}
      </div>
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
    <div className="space-y-6">
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
        <div>
          <h3 className="text-2xl font-bold text-gray-900 dark:text-white">User Targeting Conditions</h3>
          <p className="text-gray-600 dark:text-gray-400">
            Define conditions for targeting specific user groups
          </p>
        </div>
        <button className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-cyan-500 to-blue-600 text-white rounded-2xl hover:from-cyan-600 hover:to-blue-700 font-medium">
          <Plus className="h-4 w-4" />
          Add Condition
        </button>
      </div>

      <div className="grid gap-4">
        {conditions.map(condition => (
          <div key={condition.name} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-cyan-400/20 via-blue-400/20 to-indigo-400/20 p-0.5">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-cyan-300/30 to-blue-400/30 rounded-full blur-sm"></div>
              
              <div className="flex flex-col lg:flex-row lg:items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex flex-col sm:flex-row sm:items-center gap-2 mb-3">
                    <h4 className="font-semibold text-gray-900 dark:text-white">{condition.name}</h4>
                    {condition.tagColor && (
                      <Badge className={`bg-${condition.tagColor}-100 text-${condition.tagColor}-800 dark:bg-${condition.tagColor}-900/20 dark:text-${condition.tagColor}-300 w-fit`}>
                        {condition.tagColor}
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">{condition.description}</p>
                  <code className="text-xs bg-gradient-to-r from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600 p-4 rounded-xl block overflow-x-auto text-gray-900 dark:text-white">
                    {condition.expression}
                  </code>
                </div>
                <div className="flex flex-row lg:flex-col gap-2 w-full lg:w-auto">
                  <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-emerald-500 to-green-600 text-white rounded-xl hover:from-emerald-600 hover:to-green-700 font-medium flex-1 lg:flex-none">
                    <Edit className="h-3 w-3" />
                    Edit
                  </button>
                  <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-red-500 to-pink-600 text-white rounded-xl hover:from-red-600 hover:to-pink-700 font-medium flex-1 lg:flex-none">
                    <Trash2 className="h-3 w-3" />
                    Delete
                  </button>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function ExperimentsManagement({ parameters }: { parameters: Record<string, RemoteConfigParameter> }) {
  const abTestParameters = Object.values(parameters).filter(p => p.key.startsWith('ab_'))

  return (
    <div className="space-y-6">
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
        <div>
          <h3 className="text-2xl font-bold text-gray-900 dark:text-white">A/B Testing Experiments</h3>
          <p className="text-gray-600 dark:text-gray-400">
            Manage experiments and feature rollouts
          </p>
        </div>
        <button className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-purple-500 to-pink-600 text-white rounded-2xl hover:from-purple-600 hover:to-pink-700 font-medium">
          <TestTube className="h-4 w-4" />
          Create Experiment
        </button>
      </div>

      <div className="grid gap-4">
        {abTestParameters.map(param => (
          <div key={param.key} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-red-400/20 p-0.5">
            <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-purple-300/30 to-pink-400/30 rounded-full blur-sm"></div>
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="font-semibold text-gray-900 dark:text-white">{param.key.replace('ab_', '').replace('_', ' ')}</h4>
                  <p className="text-sm text-gray-600 dark:text-gray-400">{param.description}</p>
                </div>
                <div className="flex items-center gap-3">
                  <Badge className="bg-gradient-to-r from-green-400/20 to-emerald-500/20 text-green-700 dark:text-green-300 border-green-300/50">Active</Badge>
                  <span className="text-sm font-mono bg-gradient-to-r from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600 px-3 py-1 rounded-xl text-gray-900 dark:text-white">{String(param.value)}</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function ConfigAnalytics({ parameters }: { parameters: Record<string, RemoteConfigParameter> }) {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Configuration Analytics</h3>
        <p className="text-gray-600 dark:text-gray-400">Monitor parameter usage and user impact</p>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-emerald-400/20 via-green-400/20 to-teal-400/20 p-0.5">
          <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
            <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-emerald-300/30 to-green-400/30 rounded-full blur-sm"></div>
            <div className="flex items-center gap-3 mb-4">
              <div className="p-3 bg-gradient-to-r from-emerald-400/20 to-green-500/20 rounded-2xl">
                <BarChart3 className="h-6 w-6 text-emerald-600" />
              </div>
              <h4 className="font-semibold text-gray-900 dark:text-white">Parameter Usage</h4>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Analytics coming soon...
            </p>
          </div>
        </div>

        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-purple-400/20 p-0.5">
          <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
            <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-blue-300/30 to-indigo-400/30 rounded-full blur-sm"></div>
            <div className="flex items-center gap-3 mb-4">
              <div className="p-3 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-2xl">
                <Activity className="h-6 w-6 text-blue-600" />
              </div>
              <h4 className="font-semibold text-gray-900 dark:text-white">User Impact</h4>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Impact analysis coming soon...
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

function UserTargetingManagement({ 
  parameters, 
  conditions, 
  isLoading 
}: {
  parameters: Record<string, RemoteConfigParameter>
  conditions: UserCondition[]
  isLoading: boolean
}) {
  const [targetingMode, setTargetingMode] = useState<'percentage' | 'specific' | 'conditions'>('percentage')
  const [userSegments] = useState<UserSegment[]>(getCommonUserSegments())
  
  return (
    <div className="space-y-6">
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
        <div>
          <h3 className="text-2xl font-bold text-gray-900 dark:text-white">User Targeting</h3>
          <p className="text-gray-600 dark:text-gray-400">
            Configure which users receive specific Remote Config values
          </p>
        </div>
        <div className="flex flex-col sm:flex-row gap-2">
          <button className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-indigo-500 to-purple-600 text-white rounded-2xl hover:from-indigo-600 hover:to-purple-700 font-medium">
            <Users className="h-4 w-4" />
            View Users
          </button>
          <button className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-pink-500 to-rose-600 text-white rounded-2xl hover:from-pink-600 hover:to-rose-700 font-medium">
            <Plus className="h-4 w-4" />
            Create Segment
          </button>
        </div>
      </div>

      {/* Targeting Mode Selection */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-orange-400/20 via-amber-400/20 to-yellow-400/20 p-0.5">
        <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
          <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-orange-300/30 to-amber-400/30 rounded-full blur-sm"></div>
          <div className="space-y-4">
            <h4 className="font-semibold text-gray-900 dark:text-white">Targeting Strategy</h4>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              {[
                { value: 'percentage', label: 'Percentage Rollout', desc: 'Target % of users', icon: '📊' },
                { value: 'specific', label: 'Specific Users', desc: 'Target individual users', icon: '👤' },
                { value: 'conditions', label: 'User Conditions', desc: 'Target based on user properties', icon: '⚡' }
              ].map(mode => (
                <div 
                  key={mode.value}
                  className={`p-4 border rounded-2xl cursor-pointer hover:bg-white/50 dark:hover:bg-gray-800/50 ${
                    targetingMode === mode.value 
                      ? 'border-orange-400 bg-gradient-to-r from-orange-400/10 to-amber-400/10' 
                      : 'border-gray-200/50 dark:border-gray-700/50'
                  }`}
                  onClick={() => setTargetingMode(mode.value as any)}
                >
                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-lg">{mode.icon}</span>
                    <div className="font-medium text-sm text-gray-900 dark:text-white">{mode.label}</div>
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">{mode.desc}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* User Segments */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-emerald-400/20 via-teal-400/20 to-cyan-400/20 p-0.5">
        <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
          <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-emerald-300/30 to-teal-400/30 rounded-full blur-sm"></div>
          <div className="space-y-4">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <h4 className="font-semibold text-gray-900 dark:text-white">Active User Segments</h4>
              <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-teal-500 to-cyan-600 text-white rounded-xl hover:from-teal-600 hover:to-cyan-700 font-medium w-full sm:w-auto">
                <RefreshCw className="h-4 w-4" />
                Sync from Users
              </button>
            </div>
            
            <div className="grid gap-3">
              {userSegments.map(segment => (
                <div key={segment.id} className="flex flex-col lg:flex-row lg:items-center justify-between p-4 bg-white/50 dark:bg-gray-800/50 rounded-2xl border border-gray-200/50 dark:border-gray-700/50 gap-3">
                  <div className="flex-1">
                    <div className="flex flex-col sm:flex-row sm:items-center gap-2 mb-2">
                      <span className="font-medium text-sm text-gray-900 dark:text-white">{segment.name}</span>
                      <Badge className={`w-fit ${segment.isActive ? 'bg-gradient-to-r from-green-400/20 to-emerald-500/20 text-green-700 dark:text-green-300 border-green-300/50' : 'bg-gradient-to-r from-gray-400/20 to-slate-500/20 text-gray-700 dark:text-gray-300 border-gray-300/50'}`}>
                        {segment.userCount.toLocaleString()} users
                      </Badge>
                      {segment.color && (
                        <Badge className={`bg-${segment.color}-100 text-${segment.color}-800 dark:bg-${segment.color}-900/20 dark:text-${segment.color}-300 w-fit`}>
                          {segment.color}
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-gray-600 dark:text-gray-400 mb-2">{segment.description}</p>
                    <code className="text-xs bg-gradient-to-r from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600 px-3 py-2 rounded-xl block overflow-x-auto text-gray-900 dark:text-white">
                      {segment.condition}
                    </code>
                  </div>
                  <div className="flex gap-2 w-full lg:w-auto">
                    <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-blue-500 to-indigo-600 text-white rounded-xl hover:from-blue-600 hover:to-indigo-700 font-medium flex-1 lg:flex-none">
                      <Eye className="h-3 w-3" />
                      View
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-purple-500 to-pink-600 text-white rounded-xl hover:from-purple-600 hover:to-pink-700 font-medium flex-1 lg:flex-none">
                      <Edit className="h-3 w-3" />
                      Edit
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
      
      {/* Parameter Assignment */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-rose-400/20 via-pink-400/20 to-purple-400/20 p-0.5">
        <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
          <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-rose-300/30 to-pink-400/30 rounded-full blur-sm"></div>
          <div className="space-y-4">
            <div>
              <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Parameter-User Assignments</h4>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Assign different parameter values to specific user segments
              </p>
            </div>
            
            <div className="grid gap-4">
              {Object.values(parameters).slice(0, 3).map(param => (
                <div key={param.key} className="p-4 bg-white/50 dark:bg-gray-800/50 rounded-2xl border border-gray-200/50 dark:border-gray-700/50">
                  <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                    <div className="flex-1">
                      <div className="font-medium text-sm text-gray-900 dark:text-white">{param.key}</div>
                      <div className="text-xs text-gray-600 dark:text-gray-400">{param.description}</div>
                    </div>
                    <div className="flex flex-col sm:flex-row gap-3 w-full lg:w-auto">
                      <select className="px-4 py-2 bg-white/70 dark:bg-gray-700/70 border border-gray-200/50 dark:border-gray-600/50 rounded-xl text-sm text-gray-900 dark:text-white flex-1 lg:flex-none focus:outline-none focus:ring-2 focus:ring-pink-500/50">
                        <option>All Users: {formatParameterValue(param)}</option>
                        <option>Premium Users: Enhanced</option>
                        <option>Mobile Users: Optimized</option>
                      </select>
                      <button className="flex items-center justify-center gap-2 px-4 py-2 bg-gradient-to-r from-rose-500 to-pink-600 text-white rounded-xl hover:from-rose-600 hover:to-pink-700 font-medium w-full sm:w-auto">
                        Configure
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function ConfigHistory() {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Configuration History</h3>
        <p className="text-gray-600 dark:text-gray-400">Track changes and configuration updates</p>
      </div>
      
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-gray-400/20 via-slate-400/20 to-zinc-400/20 p-0.5">
        <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-12">
          <div className="text-center">
            <div className="p-4 bg-gradient-to-r from-gray-400/20 to-slate-500/20 rounded-2xl w-16 h-16 mx-auto mb-4 flex items-center justify-center">
              <Clock className="h-8 w-8 text-gray-500" />
            </div>
            <p className="text-gray-600 dark:text-gray-400">
              No configuration history available
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}