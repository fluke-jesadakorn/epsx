'use client'

import React, { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { AlertTriangle, Clock, Eye, Shield, Trash2, RefreshCcw } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'

interface TokenIntrospectionResult {
  active: boolean
  sub?: string
  email?: string
  exp?: number
  permissions?: Array<{
    permission: string
    base_permission: string
    expires_at?: number
    permission_type: 'permanent' | 'temporary'
    hours_remaining?: number
    platform?: string
    resource?: string
    action?: string
  }>
  permission_count?: number
  has_admin_access?: boolean
  package_tier?: string
  expiring_permissions_count?: number
}

interface GranularTokenManagementProps {
  userId?: string
  className?: string
}

export function GranularTokenManagement({ userId, className }: GranularTokenManagementProps) {
  const [activeTab, setActiveTab] = useState<'introspect' | 'revoke'>('introspect')
  const [token, setToken] = useState('')
  const [permission, setPermission] = useState('')
  const [revokeType, setRevokeType] = useState<'full' | 'granular'>('full')
  const [introspectionResult, setIntrospectionResult] = useState<TokenIntrospectionResult | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Handle token introspection
  const handleIntrospection = async () => {
    if (!token.trim()) {
      setError('Please enter a token to introspect')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
      const response = await fetch(`${backendUrl}/oauth/introspect`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          token: token.trim(),
          token_type_hint: 'access_token'
        }),
        credentials: 'include',
      })

      if (!response.ok) {
        throw new Error(`Introspection failed: ${response.status}`)
      }

      const result = await response.json()
      setIntrospectionResult(result)
      
      toast({
        title: "Token Introspection Successful",
        description: `Token is ${result.active ? 'active' : 'inactive'}`,
      })

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Introspection failed'
      setError(errorMessage)
      toast({
        title: "Introspection Error",
        description: errorMessage,
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  // Handle token revocation
  const handleRevocation = async () => {
    if (!token.trim()) {
      setError('Please enter a token to revoke')
      return
    }

    if (revokeType === 'granular' && !permission.trim()) {
      setError('Please enter a permission for granular revocation')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
      
      const params: Record<string, string> = {
        token: token.trim(),
        token_type_hint: 'access_token'
      }

      if (revokeType === 'granular') {
        params.permission = permission.trim()
        params.revoke_type = 'granular'
      }

      const response = await fetch(`${backendUrl}/oauth/revoke`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams(params),
        credentials: 'include',
      })

      if (!response.ok) {
        throw new Error(`Revocation failed: ${response.status}`)
      }

      toast({
        title: "Token Revocation Successful",
        description: revokeType === 'granular' 
          ? `Permission "${permission}" revoked from token`
          : "Token completely revoked",
      })

      // Clear the form after successful revocation
      setToken('')
      setPermission('')
      setIntrospectionResult(null)

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Revocation failed'
      setError(errorMessage)
      toast({
        title: "Revocation Error",
        description: errorMessage,
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  // Format time until expiry
  const formatTimeUntilExpiry = (hours?: number) => {
    if (!hours) return 'Never'
    if (hours <= 0) return 'Expired'
    
    if (hours < 1) {
      const minutes = Math.round(hours * 60)
      return `${minutes}m`
    } else if (hours < 24) {
      return `${Math.round(hours)}h`
    } else {
      const days = Math.round(hours / 24)
      return `${days}d`
    }
  }

  return (
    <div className={`space-y-6 ${className}`}>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Granular Token Management
          </CardTitle>
          <CardDescription>
            Inspect tokens and manage permissions with granular control using standard OpenID Connect endpoints
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Tab Selection */}
          <div className="flex space-x-1 bg-gray-100 rounded-lg p-1">
            <Button
              variant={activeTab === 'introspect' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setActiveTab('introspect')}
              className="flex-1"
            >
              <Eye className="h-4 w-4 mr-2" />
              Introspect Token
            </Button>
            <Button
              variant={activeTab === 'revoke' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setActiveTab('revoke')}
              className="flex-1"
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Revoke Token
            </Button>
          </div>

          {/* Token Input */}
          <div className="space-y-2">
            <Label htmlFor="token">JWT Token</Label>
            <Input
              id="token"
              type="text"
              placeholder="eyJhbGciOiJSUzI1NiIs..."
              value={token}
              onChange={(e) => setToken(e.target.value)}
              className="font-mono text-sm"
            />
          </div>

          {/* Revocation-specific controls */}
          {activeTab === 'revoke' && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="revoke-type">Revocation Type</Label>
                <Select value={revokeType} onValueChange={(value: 'full' | 'granular') => setRevokeType(value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="full">Full Token Revocation</SelectItem>
                    <SelectItem value="granular">Granular Permission Revocation</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {revokeType === 'granular' && (
                <div className="space-y-2">
                  <Label htmlFor="permission">Permission to Revoke</Label>
                  <Input
                    id="permission"
                    type="text"
                    placeholder="admin:users:modify"
                    value={permission}
                    onChange={(e) => setPermission(e.target.value)}
                  />
                  <p className="text-sm text-gray-500">
                    Format: platform:resource:action (e.g., "admin:users:modify", "epsx:analytics:view")
                  </p>
                </div>
              )}
            </div>
          )}

          {/* Error Display */}
          {error && (
            <div className="flex items-center gap-2 text-red-600 bg-red-50 p-3 rounded-md">
              <AlertTriangle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
            </div>
          )}

          {/* Action Button */}
          <Button
            onClick={activeTab === 'introspect' ? handleIntrospection : handleRevocation}
            disabled={isLoading}
            className="w-full"
          >
            {isLoading ? (
              <RefreshCcw className="h-4 w-4 mr-2 animate-spin" />
            ) : activeTab === 'introspect' ? (
              <Eye className="h-4 w-4 mr-2" />
            ) : (
              <Trash2 className="h-4 w-4 mr-2" />
            )}
            {isLoading 
              ? `${activeTab === 'introspect' ? 'Introspecting' : 'Revoking'}...`
              : activeTab === 'introspect' 
                ? 'Introspect Token'
                : 'Revoke Token'
            }
          </Button>
        </CardContent>
      </Card>

      {/* Introspection Results */}
      {introspectionResult && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Eye className="h-5 w-5" />
              Token Introspection Results
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Token Status */}
            <div className="flex items-center gap-2">
              <Badge variant={introspectionResult.active ? 'default' : 'destructive'}>
                {introspectionResult.active ? 'Active' : 'Inactive'}
              </Badge>
              {introspectionResult.has_admin_access && (
                <Badge variant="secondary">Admin Access</Badge>
              )}
              {introspectionResult.package_tier && (
                <Badge variant="outline">{introspectionResult.package_tier}</Badge>
              )}
            </div>

            {/* User Info */}
            {introspectionResult.active && (
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <strong>User:</strong> {introspectionResult.sub}
                </div>
                <div>
                  <strong>Email:</strong> {introspectionResult.email}
                </div>
                <div>
                  <strong>Permissions:</strong> {introspectionResult.permission_count || 0}
                </div>
                <div>
                  <strong>Expiring Soon:</strong> {introspectionResult.expiring_permissions_count || 0}
                </div>
              </div>
            )}

            <Separator />

            {/* Permissions List */}
            {introspectionResult.permissions && introspectionResult.permissions.length > 0 && (
              <div className="space-y-3">
                <h4 className="font-medium">Permissions Details</h4>
                <div className="space-y-2 max-h-60 overflow-y-auto">
                  {introspectionResult.permissions.map((perm, index) => (
                    <div key={index} className="border rounded-md p-3 space-y-2">
                      <div className="flex items-center justify-between">
                        <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                          {perm.permission}
                        </code>
                        <div className="flex items-center gap-2">
                          <Badge variant={perm.permission_type === 'permanent' ? 'default' : 'secondary'}>
                            {perm.permission_type}
                          </Badge>
                          {perm.hours_remaining !== undefined && (
                            <Badge 
                              variant={perm.hours_remaining <= 24 ? 'destructive' : 'outline'}
                              className="flex items-center gap-1"
                            >
                              <Clock className="h-3 w-3" />
                              {formatTimeUntilExpiry(perm.hours_remaining)}
                            </Badge>
                          )}
                        </div>
                      </div>
                      
                      <div className="text-xs text-gray-600 grid grid-cols-3 gap-2">
                        <div><strong>Platform:</strong> {perm.platform}</div>
                        <div><strong>Resource:</strong> {perm.resource}</div>
                        <div><strong>Action:</strong> {perm.action}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}

export default GranularTokenManagement