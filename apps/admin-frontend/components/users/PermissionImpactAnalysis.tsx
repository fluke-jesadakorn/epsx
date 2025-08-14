/**
 * Permission Impact Analysis Component
 * Shows what a user can and cannot access
 */

'use client'

import React, { useState, useEffect } from 'react'
import { Shield, Check, X, RefreshCw, Eye, AlertTriangle, Info } from 'lucide-react'
import { Button } from '@epsx/ui'
import { Badge } from '@epsx/ui'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui'
import { Progress } from '@/components/ui/progress'
import { getPermissionImpact } from '@/lib/actions/user-actions'

interface PermissionImpactAnalysisProps {
  userId: string
  className?: string
}

interface PermissionImpact {
  canAccess: string[]
  cannotAccess: string[]
  totalResources: number
}

export function PermissionImpactAnalysis({ userId, className = '' }: PermissionImpactAnalysisProps) {
  const [impact, setImpact] = useState<PermissionImpact | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadImpactAnalysis()
  }, [userId])

  const loadImpactAnalysis = async () => {
    try {
      setLoading(true)
      setError(null)
      
      const result = await getPermissionImpact(userId)
      
      if (result.success) {
        setImpact(result.data)
      } else {
        setError(result.error?.message || 'Failed to analyze permission impact')
      }
    } catch (err) {
      setError('An unexpected error occurred')
      console.error('Permission impact analysis error:', err)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <div className={`space-y-4 ${className}`}>
        <div className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className={`space-y-4 ${className}`}>
        <div className="text-center text-red-600 py-4">
          <AlertTriangle className="h-8 w-8 mx-auto mb-2" />
          <p>{error}</p>
          <Button 
            variant="outline" 
            size="sm" 
            onClick={loadImpactAnalysis}
            className="mt-2"
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            Retry
          </Button>
        </div>
      </div>
    )
  }

  if (!impact) {
    return (
      <div className={`text-center text-muted-foreground py-8 ${className}`}>
        <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
        <p>No permission data available</p>
      </div>
    )
  }

  const accessPercentage = Math.round((impact.canAccess.length / impact.totalResources) * 100)
  const hasFullAccess = accessPercentage === 100
  const hasLimitedAccess = accessPercentage > 0 && accessPercentage < 100
  const hasNoAccess = accessPercentage === 0

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Access Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="flex flex-col items-center p-4 border rounded-lg bg-card">
          <div className="text-2xl font-bold text-green-600">{impact.canAccess.length}</div>
          <div className="text-sm text-muted-foreground">Can Access</div>
          <Check className="h-5 w-5 text-green-600 mt-1" />
        </div>
        
        <div className="flex flex-col items-center p-4 border rounded-lg bg-card">
          <div className="text-2xl font-bold text-red-600">{impact.cannotAccess.length}</div>
          <div className="text-sm text-muted-foreground">Cannot Access</div>
          <X className="h-5 w-5 text-red-600 mt-1" />
        </div>
        
        <div className="flex flex-col items-center p-4 border rounded-lg bg-card">
          <div className="text-2xl font-bold">{impact.totalResources}</div>
          <div className="text-sm text-muted-foreground">Total Resources</div>
          <Shield className="h-5 w-5 text-muted-foreground mt-1" />
        </div>
      </div>

      {/* Access Level Indicator */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Access Level</span>
          <div className="flex items-center gap-2">
            <Badge 
              variant={hasFullAccess ? "default" : hasLimitedAccess ? "secondary" : "destructive"}
              className="text-xs"
            >
              {hasFullAccess ? 'Full Access' : hasLimitedAccess ? 'Limited Access' : 'No Access'}
            </Badge>
            <span className="text-sm text-muted-foreground">{accessPercentage}%</span>
          </div>
        </div>
        <Progress 
          value={accessPercentage} 
          className={`h-2 ${hasNoAccess ? 'bg-red-100' : hasLimitedAccess ? 'bg-yellow-100' : 'bg-green-100'}`}
        />
      </div>

      {/* Access Status Message */}
      <div className={`p-3 rounded-lg flex items-start gap-2 text-sm ${hasFullAccess ? 'bg-green-50 text-green-800 border border-green-200' : hasLimitedAccess ? 'bg-yellow-50 text-yellow-800 border border-yellow-200' : 'bg-red-50 text-red-800 border border-red-200'}`}>
        {hasFullAccess ? (
          <>
            <Check className="h-4 w-4 mt-0.5 flex-shrink-0" />
            <div>
              <div className="font-medium">Full Access Granted</div>
              <div className="text-xs opacity-75">User has permissions to access all system resources.</div>
            </div>
          </>
        ) : hasLimitedAccess ? (
          <>
            <Info className="h-4 w-4 mt-0.5 flex-shrink-0" />
            <div>
              <div className="font-medium">Limited Access</div>
              <div className="text-xs opacity-75">User can access {impact.canAccess.length} of {impact.totalResources} resources. Review permissions if broader access is needed.</div>
            </div>
          </>
        ) : (
          <>
            <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" />
            <div>
              <div className="font-medium">No Access</div>
              <div className="text-xs opacity-75">User cannot access any system resources. Assign permissions to enable access.</div>
            </div>
          </>
        )}
      </div>

      {/* Detailed Access Lists */}
      <Tabs defaultValue="accessible" className="w-full">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="accessible" className="flex items-center gap-2">
            <Check className="h-4 w-4" />
            Accessible ({impact.canAccess.length})
          </TabsTrigger>
          <TabsTrigger value="restricted" className="flex items-center gap-2">
            <X className="h-4 w-4" />
            Restricted ({impact.cannotAccess.length})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="accessible" className="space-y-2 mt-4">
          {impact.canAccess.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">
              <Eye className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No accessible resources</p>
              <p className="text-xs">User needs permissions to access system resources</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
              {impact.canAccess.map((resource, index) => (
                <div 
                  key={index}
                  className="flex items-center gap-2 p-2 rounded border bg-green-50 border-green-200"
                >
                  <Check className="h-4 w-4 text-green-600 flex-shrink-0" />
                  <span className="font-mono text-sm text-green-800">{resource}</span>
                </div>
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="restricted" className="space-y-2 mt-4">
          {impact.cannotAccess.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">
              <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No restricted resources</p>
              <p className="text-xs">User has access to all system resources</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
              {impact.cannotAccess.map((resource, index) => (
                <div 
                  key={index}
                  className="flex items-center gap-2 p-2 rounded border bg-red-50 border-red-200"
                >
                  <X className="h-4 w-4 text-red-600 flex-shrink-0" />
                  <span className="font-mono text-sm text-red-800">{resource}</span>
                </div>
              ))}
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* Refresh Button */}
      <div className="flex justify-center pt-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={loadImpactAnalysis}
          disabled={loading}
        >
          <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
          Refresh Analysis
        </Button>
      </div>
    </div>
  )
}