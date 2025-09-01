'use client'

import React, { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Shield, Trash2, AlertTriangle, RefreshCcw } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'

interface UserTokenActionsProps {
  userId: string
  userEmail: string
  className?: string
}

export function UserTokenActions({ userId, userEmail, className }: UserTokenActionsProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [action, setAction] = useState<'revoke-permission' | 'revoke-all'>('revoke-permission')
  const [permission, setPermission] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  // Common permission templates for quick selection
  const commonPermissions = [
    'admin:users:modify',
    'admin:users:view',
    'admin:system:configure',
    'epsx:analytics:view',
    'epsx:analytics:export',
    'epsx:realtime:access',
    'epsx:billing:manage'
  ]

  // Handle granular permission revocation for user
  const handleGranularRevocation = async () => {
    if (!permission.trim()) {
      toast({
        title: "Validation Error",
        description: "Please select or enter a permission to revoke",
        variant: "destructive",
      })
      return
    }

    setIsLoading(true)

    try {
      // This would typically get the user's current token through an admin API
      // For now, we'll show how it would work with a token
      const response = await fetch('/api/admin/users/revoke-permission', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          userId,
          permission: permission.trim(),
          action: 'granular'
        }),
        credentials: 'include',
      })

      if (!response.ok) {
        throw new Error(`Revocation failed: ${response.status}`)
      }

      const result = await response.json()
      
      toast({
        title: "Permission Revoked",
        description: `Permission "${permission}" has been revoked from ${userEmail}`,
      })

      setPermission('')
      setIsOpen(false)

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to revoke permission'
      toast({
        title: "Revocation Failed",
        description: errorMessage,
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  // Handle full token revocation for user
  const handleFullRevocation = async () => {
    setIsLoading(true)

    try {
      const response = await fetch('/api/admin/users/revoke-tokens', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          userId,
          action: 'full'
        }),
        credentials: 'include',
      })

      if (!response.ok) {
        throw new Error(`Token revocation failed: ${response.status}`)
      }

      toast({
        title: "Tokens Revoked",
        description: `All active tokens for ${userEmail} have been revoked. User will need to re-authenticate.`,
      })

      setIsOpen(false)

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to revoke tokens'
      toast({
        title: "Revocation Failed",
        description: errorMessage,
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleAction = () => {
    if (action === 'revoke-permission') {
      handleGranularRevocation()
    } else {
      handleFullRevocation()
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm" className={className}>
          <Shield className="h-4 w-4 mr-2" />
          Token Actions
        </Button>
      </DialogTrigger>
      
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Token Management for {userEmail}
          </DialogTitle>
          <DialogDescription>
            Manage user tokens with granular control using OpenID Connect standard endpoints
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Action Selection */}
          <div className="space-y-2">
            <Label htmlFor="action">Action Type</Label>
            <Select value={action} onValueChange={(value: 'revoke-permission' | 'revoke-all') => setAction(value)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="revoke-permission">Revoke Specific Permission</SelectItem>
                <SelectItem value="revoke-all">Revoke All Tokens (Force Re-auth)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Permission Selection for Granular Revocation */}
          {action === 'revoke-permission' && (
            <div className="space-y-4">
              <Separator />
              
              <div className="space-y-2">
                <Label htmlFor="permission">Permission to Revoke</Label>
                <Input
                  id="permission"
                  type="text"
                  placeholder="admin:users:modify"
                  value={permission}
                  onChange={(e) => setPermission(e.target.value)}
                />
                <p className="text-xs text-gray-500">
                  Format: platform:resource:action
                </p>
              </div>

              {/* Quick Permission Templates */}
              <div className="space-y-2">
                <Label>Common Permissions</Label>
                <div className="flex flex-wrap gap-2">
                  {commonPermissions.map((perm) => (
                    <Button
                      key={perm}
                      variant="outline"
                      size="sm"
                      onClick={() => setPermission(perm)}
                      className="text-xs"
                    >
                      {perm}
                    </Button>
                  ))}
                </div>
              </div>

              <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
                <div className="flex items-start gap-2">
                  <AlertTriangle className="h-4 w-4 text-blue-600 mt-0.5" />
                  <div className="text-sm text-blue-800">
                    <strong>Granular Revocation:</strong> This will remove the specific permission from the user's active tokens without forcing re-authentication. The user will lose access to this specific functionality immediately.
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Full Revocation Warning */}
          {action === 'revoke-all' && (
            <div className="space-y-4">
              <Separator />
              
              <div className="bg-red-50 border border-red-200 rounded-md p-3">
                <div className="flex items-start gap-2">
                  <AlertTriangle className="h-4 w-4 text-red-600 mt-0.5" />
                  <div className="text-sm text-red-800">
                    <strong>Full Token Revocation:</strong> This will revoke all active tokens for this user. They will be immediately logged out from all devices and must re-authenticate to access the system.
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="outline"
              onClick={() => setIsOpen(false)}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button
              onClick={handleAction}
              disabled={isLoading || (action === 'revoke-permission' && !permission.trim())}
              variant={action === 'revoke-all' ? 'destructive' : 'default'}
            >
              {isLoading ? (
                <RefreshCcw className="h-4 w-4 mr-2 animate-spin" />
              ) : (
                <Trash2 className="h-4 w-4 mr-2" />
              )}
              {isLoading 
                ? 'Processing...'
                : action === 'revoke-permission' 
                  ? 'Revoke Permission'
                  : 'Revoke All Tokens'
              }
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

export default UserTokenActions