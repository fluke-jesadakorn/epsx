'use client'

/**
 * Remote Config Settings Tab Component
 * Allows users to view and manage their remote configuration settings
 */

import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Alert, AlertDescription } from '@/components/ui'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { 
  Settings, 
  Palette, 
  Zap, 
  Flag, 
  BarChart3, 
  TestTube, 
  RefreshCw, 
  CheckCircle, 
  AlertTriangle, 
  Clock,
  Wifi,
  WifiOff
} from 'lucide-react'
import { 
  useRemoteConfigContext,
  useUXSettingsContext,
  usePerformanceSettingsContext,
  useFeatureFlagsContext,
  useBusinessSettingsContext,
  useABTestSettingsContext
} from '@/providers/RemoteConfigProvider'

export function RemoteConfigTab() {
  const { 
    isLoading, 
    error, 
    lastFetchTime, 
    isRefreshing, 
    refreshSettings,
    status
  } = useRemoteConfigContext()

  const [activeSubTab, setActiveSubTab] = useState('overview')

  const handleRefresh = async () => {
    await refreshSettings()
  }

  const getStatusBadge = () => {
    if (isLoading) {
      return <Badge variant="outline" className="bg-blue-100 text-blue-800">Loading...</Badge>
    }
    if (error) {
      return <Badge variant="destructive">Error</Badge>
    }
    if (status.isReady && status.activeConfig) {
      return <Badge variant="default" className="bg-green-100 text-green-800">
        <CheckCircle className="h-3 w-3 mr-1" />
        Active
      </Badge>
    }
    return <Badge variant="outline" className="bg-yellow-100 text-yellow-800">
      <AlertTriangle className="h-3 w-3 mr-1" />
      Inactive
    </Badge>
  }

  const getConnectionIcon = () => {
    if (status.isReady && status.activeConfig) {
      return <Wifi className="h-4 w-4 text-green-600" />
    }
    return <WifiOff className="h-4 w-4 text-red-600" />
  }

  return (
    <div className="space-y-6">
      {/* Status Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {getConnectionIcon()}
            Remote Configuration Status
            {getStatusBadge()}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Connection:</span>
                <span className={status.isReady ? 'text-green-600' : 'text-red-600'}>
                  {status.isReady ? 'Connected' : 'Disconnected'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Active Config:</span>
                <span className={status.activeConfig ? 'text-green-600' : 'text-yellow-600'}>
                  {status.activeConfig ? 'Yes' : 'No'}
                </span>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Last Update:</span>
                <span className="text-xs">
                  {lastFetchTime ? lastFetchTime.toLocaleTimeString() : 'Never'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Fetch Status:</span>
                <span className="text-xs">
                  {status.lastFetchStatus}
                </span>
              </div>
            </div>
          </div>

          {error && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                {error}
              </AlertDescription>
            </Alert>
          )}

          <div className="flex gap-3">
            <Button 
              onClick={handleRefresh}
              disabled={isRefreshing}
              variant="outline"
              size="sm"
              className="flex items-center gap-2"
            >
              {isRefreshing ? (
                <RefreshCw className="h-4 w-4 animate-spin" />
              ) : (
                <RefreshCw className="h-4 w-4" />
              )}
              Refresh Settings
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Settings Categories */}
      <Tabs value={activeSubTab} onValueChange={setActiveSubTab} className="space-y-4">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="overview" className="flex items-center gap-1">
            <Settings className="h-3 w-3" />
            <span className="hidden sm:inline">Overview</span>
          </TabsTrigger>
          <TabsTrigger value="appearance" className="flex items-center gap-1">
            <Palette className="h-3 w-3" />
            <span className="hidden sm:inline">Theme</span>
          </TabsTrigger>
          <TabsTrigger value="performance" className="flex items-center gap-1">
            <Zap className="h-3 w-3" />
            <span className="hidden sm:inline">Speed</span>
          </TabsTrigger>
          <TabsTrigger value="features" className="flex items-center gap-1">
            <Flag className="h-3 w-3" />
            <span className="hidden sm:inline">Features</span>
          </TabsTrigger>
          <TabsTrigger value="business" className="flex items-center gap-1">
            <BarChart3 className="h-3 w-3" />
            <span className="hidden sm:inline">Display</span>
          </TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview">
          <RemoteConfigOverview />
        </TabsContent>

        {/* Appearance Tab */}
        <TabsContent value="appearance">
          <UXSettingsPanel />
        </TabsContent>

        {/* Performance Tab */}
        <TabsContent value="performance">
          <PerformanceSettingsPanel />
        </TabsContent>

        {/* Features Tab */}
        <TabsContent value="features">
          <FeatureFlagsPanel />
        </TabsContent>

        {/* Business Tab */}
        <TabsContent value="business">
          <BusinessSettingsPanel />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// ============================================================================
// Sub Components
// ============================================================================

function RemoteConfigOverview() {
  const { settings } = useRemoteConfigContext()
  
  return (
    <div className="grid gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Current Configuration Summary</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <h4 className="font-medium mb-2">Appearance</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>Theme: {settings.ux.theme}</li>
                <li>Compact: {settings.ux.compactMode ? 'Yes' : 'No'}</li>
                <li>Animations: {settings.ux.animationsEnabled ? 'On' : 'Off'}</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium mb-2">Performance</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>Refresh: {settings.performance.refreshInterval / 1000}s</li>
                <li>Real-time: {settings.performance.realTimeUpdates ? 'On' : 'Off'}</li>
                <li>Cache: {settings.performance.dataCacheMinutes}m</li>
              </ul>
            </div>
          </div>
          
          <div className="bg-blue-50 dark:bg-blue-950 p-3 rounded-lg">
            <div className="text-sm text-blue-700 dark:text-blue-300">
              <strong>💡 About Remote Config:</strong> These settings are managed remotely and update automatically. 
              Changes made by administrators will sync to your app without requiring updates.
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function UXSettingsPanel() {
  const { uxSettings, isLoading } = useUXSettingsContext()

  if (isLoading) {
    return <div className="text-center py-4">Loading UX settings...</div>
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Appearance & Experience</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Theme</div>
              <div className="text-sm text-muted-foreground">Current interface theme</div>
            </div>
            <Badge variant="outline">{uxSettings.theme}</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Compact Mode</div>
              <div className="text-sm text-muted-foreground">Reduces spacing and component sizes</div>
            </div>
            <Badge variant={uxSettings.compactMode ? 'default' : 'outline'}>
              {uxSettings.compactMode ? 'On' : 'Off'}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Animations</div>
              <div className="text-sm text-muted-foreground">Interface transitions and effects</div>
            </div>
            <Badge variant={uxSettings.animationsEnabled ? 'default' : 'outline'}>
              {uxSettings.animationsEnabled ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Mobile Optimization</div>
              <div className="text-sm text-muted-foreground">Enhanced mobile experience</div>
            </div>
            <Badge variant={uxSettings.mobileOptimized ? 'default' : 'outline'}>
              {uxSettings.mobileOptimized ? 'On' : 'Off'}
            </Badge>
          </div>
        </div>

        <Alert>
          <Settings className="h-4 w-4" />
          <AlertDescription>
            These appearance settings are managed remotely and will sync automatically across all your devices.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}

function PerformanceSettingsPanel() {
  const { performanceSettings, isLoading } = usePerformanceSettingsContext()

  if (isLoading) {
    return <div className="text-center py-4">Loading performance settings...</div>
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Performance & Speed</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Refresh Interval</div>
              <div className="text-sm text-muted-foreground">How often data updates automatically</div>
            </div>
            <Badge variant="outline">{performanceSettings.refreshInterval / 1000}s</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Real-time Updates</div>
              <div className="text-sm text-muted-foreground">Live data streaming</div>
            </div>
            <Badge variant={performanceSettings.realTimeUpdates ? 'default' : 'outline'}>
              {performanceSettings.realTimeUpdates ? 'On' : 'Off'}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Data Cache Duration</div>
              <div className="text-sm text-muted-foreground">How long to store data locally</div>
            </div>
            <Badge variant="outline">{performanceSettings.dataCacheMinutes}m</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Lazy Load Images</div>
              <div className="text-sm text-muted-foreground">Load images only when visible</div>
            </div>
            <Badge variant={performanceSettings.lazyLoadImages ? 'default' : 'outline'}>
              {performanceSettings.lazyLoadImages ? 'On' : 'Off'}
            </Badge>
          </div>
        </div>

        <Alert>
          <Zap className="h-4 w-4" />
          <AlertDescription>
            Performance settings are optimized based on your device capabilities and connection speed.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}

function FeatureFlagsPanel() {
  const { featureFlags, isLoading } = useFeatureFlagsContext()

  if (isLoading) {
    return <div className="text-center py-4">Loading feature flags...</div>
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Feature Access</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Advanced Charts</div>
              <div className="text-sm text-muted-foreground">Enhanced charting and analysis tools</div>
            </div>
            <Badge variant={featureFlags.advancedChartsEnabled ? 'default' : 'outline'}>
              {featureFlags.advancedChartsEnabled ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Beta Analytics</div>
              <div className="text-sm text-muted-foreground">Experimental analytics features</div>
            </div>
            <Badge variant={featureFlags.betaAnalyticsEnabled ? 'default' : 'outline'}>
              {featureFlags.betaAnalyticsEnabled ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Experimental UI</div>
              <div className="text-sm text-muted-foreground">New interface components</div>
            </div>
            <Badge variant={featureFlags.experimentalUIEnabled ? 'default' : 'outline'}>
              {featureFlags.experimentalUIEnabled ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">New Notifications</div>
              <div className="text-sm text-muted-foreground">Enhanced notification system</div>
            </div>
            <Badge variant={featureFlags.newNotificationSystem ? 'default' : 'outline'}>
              {featureFlags.newNotificationSystem ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>
        </div>

        <Alert>
          <Flag className="h-4 w-4" />
          <AlertDescription>
            Feature flags control access to new and experimental functionality. Access is managed by administrators.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}

function BusinessSettingsPanel() {
  const { businessSettings, isLoading } = useBusinessSettingsContext()

  if (isLoading) {
    return <div className="text-center py-4">Loading display settings...</div>
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Display & Data</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Stocks Per Page</div>
              <div className="text-sm text-muted-foreground">Number of stocks shown at once</div>
            </div>
            <Badge variant="outline">{businessSettings.stocksPerPage}</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Default Chart Type</div>
              <div className="text-sm text-muted-foreground">Initial chart visualization style</div>
            </div>
            <Badge variant="outline">{businessSettings.defaultChartType}</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Max Watchlist Items</div>
              <div className="text-sm text-muted-foreground">Maximum stocks in watchlists</div>
            </div>
            <Badge variant="outline">{businessSettings.maxWatchlistItems}</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Data Export</div>
              <div className="text-sm text-muted-foreground">Export data to CSV/Excel</div>
            </div>
            <Badge variant={businessSettings.dataExportEnabled ? 'default' : 'outline'}>
              {businessSettings.dataExportEnabled ? 'Available' : 'Disabled'}
            </Badge>
          </div>
        </div>

        <Alert>
          <BarChart3 className="h-4 w-4" />
          <AlertDescription>
            Display settings control how data is presented and the limits for various features.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}