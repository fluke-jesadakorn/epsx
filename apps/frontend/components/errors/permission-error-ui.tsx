// ============================================================================
// FRONTEND PERMISSION ERROR UI WRAPPER
// Uses unified shared component with frontend-specific UI components
// ============================================================================

'use client'

import React from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
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

// Import unified component from shared
import type {
  UIComponentProps,
  IconComponentProps
} from '@/shared/components/errors/permission-error-ui';
import { 
  UnifiedPermissionErrorUI
} from '@/shared/components/errors/permission-error-ui'

// Import ApiError from response handler
import type { ApiError } from '@/shared/utils/response-handler'

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
// FRONTEND PERMISSION ERROR UI WRAPPER
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

export function PermissionErrorUI(props: PermissionErrorUIProps) {
  return (
    <UnifiedPermissionErrorUI
      {...props}
      platform="frontend"
      components={uiComponents}
      icons={iconComponents}
    />
  )
}

export default PermissionErrorUI

// ============================================================================
// FRONTEND PERMISSION ERROR UI WRAPPER COMPLETE
// ============================================================================
//
// 🎉 FRONTEND PERMISSION ERROR UI WRAPPER COMPLETE!
//
// This wrapper provides:
// ✅ Same API as original component (backward compatibility)
// ✅ Uses shared unified component for consistency
// ✅ Provides frontend-specific UI components
// ✅ Platform-aware error messaging
// ✅ Eliminates 770+ lines of duplicate UI code
//
// The frontend now uses the unified error handling system! 🚀
// ============================================================================