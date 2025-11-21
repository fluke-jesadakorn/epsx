'use client'

/**
 * Settings Tab Component
 * Basic settings interface without complex providers
 */

import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Alert, AlertDescription } from '@/components/ui'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs'
import { 
  Settings, 
  Palette, 
  Zap, 
  Flag, 
  BarChart3, 
  Info
} from 'lucide-react'

export function RemoteConfigTab() {
  const [activeSubTab, setActiveSubTab] = useState('overview')

  return (
    <div className="space-y-6">
      {/* Basic Settings Info */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Application Settings
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              Settings are managed through your account preferences and backend configuration.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Settings Categories */}
      <Tabs value={activeSubTab} onValueChange={setActiveSubTab} className="space-y-4">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="overview" className="flex items-center gap-1">
            <Settings className="h-3 w-3" />
            <span className="hidden sm:inline">Overview</span>
          </TabsTrigger>
          <TabsTrigger value="appearance" className="flex items-center gap-1">
            <Palette className="h-3 w-3" />
            <span className="hidden sm:inline">Preferences</span>
          </TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview">
          <BasicOverview />
        </TabsContent>

        {/* Preferences Tab */}
        <TabsContent value="appearance">
          <BasicPreferences />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// ============================================================================
// Basic Components
// ============================================================================

function BasicOverview() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Settings Overview</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Application Settings</div>
              <div className="text-sm text-muted-foreground">Managed by backend configuration</div>
            </div>
            <Badge variant="outline">Active</Badge>
          </div>
          
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Permission System</div>
              <div className="text-sm text-muted-foreground">Backend handles access control</div>
            </div>
            <Badge variant="outline">Enabled</Badge>
          </div>
        </div>
        
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>
            Settings are managed through your backend configuration and will be applied automatically based on your permission group.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}

function BasicPreferences() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>User Preferences</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Theme</div>
              <div className="text-sm text-muted-foreground">Interface appearance</div>
            </div>
            <Badge variant="outline">System</Badge>
          </div>
          
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Notifications</div>
              <div className="text-sm text-muted-foreground">Alert preferences</div>
            </div>
            <Badge variant="outline">Enabled</Badge>
          </div>
        </div>
        
        <Alert>
          <Palette className="h-4 w-4" />
          <AlertDescription>
            Preferences are synchronized across devices based on your account settings.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}