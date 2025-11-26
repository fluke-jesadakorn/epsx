// ============================================================================
// ADMIN FRONTEND PERMISSION ERROR UI WRAPPER
// Uses unified shared component with admin-specific UI components
// ============================================================================

'use client'

import { 
  AlertTriangle, 
  ShieldAlert, 
  RefreshCw, 
  ArrowUp, 
  LogIn,
  HelpCircle,
  Clock,
  Zap,
  Shield,
  User,
  CreditCard,
  Star,
  CheckCircle,
  XCircle,
  Timer,
  TrendingUp,
  Lock
} from 'lucide-react'
import React from 'react'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import {
  UnifiedPermissionErrorUI,
  UIComponentProps,
  IconComponentProps
} from '@/shared/components/errors/PermissionErrorUI'
import { ApiError } from '@/shared/utils/response-handler'

// ============================================================================
// UI COMPONENT MAPPINGS
// ============================================================================

const uiComponents: UIComponentProps = {
  Alert,
  AlertDescription,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Badge,
  Progress
}

const iconComponents: IconComponentProps = {
  AlertTriangle,
  ShieldAlert,
  RefreshCw,
  ArrowUp,
  LogIn,
  HelpCircle,
  Clock,
  Zap,
  Shield,
  User,
  CreditCard,
  Star,
  CheckCircle,
  XCircle,
  Timer,
  TrendingUp,
  Lock
}

// ============================================================================
// ADMIN FRONTEND PERMISSION ERROR UI WRAPPER
// ============================================================================

interface PermissionErrorUIProps {
  error: ApiError
  onRetry?: () => void
  onUpgrade?: (tier?: string) => void
  onLogin?: () => void
  onSupport?: (context?: any) => void
  showRetry?: boolean
  showSupport?: boolean
  variant?: 'alert' | 'card' | 'full-page'
  className?: string
}

/**
 *
 * @param props
 */
export function PermissionErrorUI(props: PermissionErrorUIProps) {
  return (
    <UnifiedPermissionErrorUI
      {...props}
      platform="admin"
      components={uiComponents}
      icons={iconComponents}
    />
  )
}

export default PermissionErrorUI

// ============================================================================
// ADMIN FRONTEND PERMISSION ERROR UI WRAPPER COMPLETE
// ============================================================================
//
// 🎉 ADMIN FRONTEND PERMISSION ERROR UI WRAPPER COMPLETE!
//
// This wrapper provides:
// ✅ Same API as original component (backward compatibility)
// ✅ Uses shared unified component for consistency
// ✅ Provides admin-specific UI components
// ✅ Platform-aware error messaging (admin context)
// ✅ Eliminates 770+ lines of duplicate UI code
//
// The admin frontend now uses the unified error handling system! 🚀
// ============================================================================