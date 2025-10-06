'use client'

/**
 * Permission Denied Modal
 * Pure UI component displaying backend permission errors
 * Shows upgrade paths, benefits, and actions from backend
 */

import { AlertTriangle, Lock, Shield, Zap } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type { BackendPermissionError } from '@/lib/errors/permission-error-handler'

interface PermissionDeniedModalProps {
  error: BackendPermissionError
  open: boolean
  onClose: () => void
}

/**
 *
 * @param root0
 * @param root0.error
 * @param root0.open
 * @param root0.onClose
 */
export function PermissionDeniedModal({ error, open, onClose }: PermissionDeniedModalProps): React.JSX.Element {
  const handleUpgrade = () => {
    if (error.upgrade_url) {
      window.location.href = error.upgrade_url
    } else {
      window.location.href = '/plans'
    }
  }

  const getIcon = () => {
    switch (error.error_type) {
      case 'SecurityRestriction':
        return <Shield className="h-12 w-12 text-red-500" />
      case 'UsageLimitExceeded':
        return <Zap className="h-12 w-12 text-yellow-500" />
      case 'InsufficientGroup':
      case 'PermissionDenied':
        return <Lock className="h-12 w-12 text-orange-500" />
      default:
        return <AlertTriangle className="h-12 w-12 text-gray-500" />
    }
  }

  const getRiskBadge = () => {
    if (!error.risk_level) {return null}
    const colors = {
      low: 'bg-blue-100 text-blue-800',
      medium: 'bg-yellow-100 text-yellow-800',
      high: 'bg-orange-100 text-orange-800',
      critical: 'bg-red-100 text-red-800'
    }
    return (
      <Badge className={colors[error.risk_level]}>
        {error.risk_level.toUpperCase()} RISK
      </Badge>
    )
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <div className="flex items-center justify-center mb-4">
            {getIcon()}
          </div>
          <DialogTitle className="text-center text-xl">
            {error.error_type === 'InsufficientGroup' ? 'Upgrade Required' : 'Access Restricted'}
          </DialogTitle>
          <DialogDescription className="text-center">
            {error.message}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Current vs Required Group */}
          {error.current_group && error.required_group && (
            <div className="bg-slate-50 dark:bg-slate-900 p-4 rounded-lg space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-slate-600 dark:text-slate-400">Current Plan:</span>
                <Badge variant="outline">{error.current_group}</Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-slate-600 dark:text-slate-400">Required Plan:</span>
                <Badge className="bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300">
                  {error.required_group}
                </Badge>
              </div>
            </div>
          )}

          {/* Usage Limit Info */}
          {error.current_usage !== undefined && error.limit !== undefined && (
            <div className="bg-slate-50 dark:bg-slate-900 p-4 rounded-lg">
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-slate-600 dark:text-slate-400">Usage:</span>
                <span className="font-semibold">{error.current_usage} / {error.limit}</span>
              </div>
              <div className="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-2">
                <div
                  className="bg-orange-500 h-2 rounded-full"
                  style={{ width: `${Math.min((error.current_usage / error.limit) * 100, 100)}%` }}
                />
              </div>
              {error.reset_at && (
                <p className="text-xs text-slate-500 mt-2">
                  Resets: {new Date(error.reset_at).toLocaleString()}
                </p>
              )}
            </div>
          )}

          {/* Benefits List */}
          {error.benefits && error.benefits.length > 0 && (
            <div>
              <h4 className="font-semibold text-sm mb-2">Upgrade Benefits:</h4>
              <ul className="space-y-1">
                {error.benefits.map((benefit, index) => (
                  <li key={index} className="text-sm text-slate-600 dark:text-slate-400 flex items-start">
                    <span className="text-green-500 mr-2">✓</span>
                    {benefit}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Suggested Actions */}
          {error.suggested_actions && error.suggested_actions.length > 0 && (
            <div>
              <h4 className="font-semibold text-sm mb-2">Suggested Actions:</h4>
              <ul className="space-y-1">
                {error.suggested_actions.map((action, index) => (
                  <li key={index} className="text-sm text-slate-600 dark:text-slate-400 flex items-start">
                    <span className="text-blue-500 mr-2">→</span>
                    {action}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Security Restriction Warning */}
          {error.error_type === 'SecurityRestriction' && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-4 rounded-lg">
              <div className="flex items-start gap-2">
                {getRiskBadge()}
              </div>
              <p className="text-sm text-red-800 dark:text-red-200 mt-2">
                {error.reason ?? 'This action has been restricted for security reasons.'}
              </p>
              {error.contact_support && (
                <p className="text-xs text-red-600 dark:text-red-400 mt-2">
                  Please contact support if you believe this is an error.
                </p>
              )}
            </div>
          )}

          {/* Permission Expired */}
          {error.error_type === 'PermissionExpired' && error.expired_at && (
            <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 p-4 rounded-lg">
              <p className="text-sm text-yellow-800 dark:text-yellow-200">
                Expired: {new Date(error.expired_at).toLocaleString()}
              </p>
            </div>
          )}
        </div>

        <DialogFooter className="flex gap-2">
          <Button variant="outline" onClick={onClose}>
            Close
          </Button>
          {(error.upgrade_url ?? error.renewal_url) && (
            <Button
              onClick={handleUpgrade}
              className="bg-gradient-to-r from-orange-500 to-pink-500 hover:from-orange-600 hover:to-pink-600"
            >
              {error.renewal_url ? 'Renew Access' : 'Upgrade Now'}
            </Button>
          )}
          {error.contact_support && (
            <Button variant="outline" onClick={() => window.location.href = '/support'}>
              Contact Support
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
