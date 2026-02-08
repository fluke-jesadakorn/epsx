
import type React from 'react'
import type { ApiError } from '../../../utils/response-handler'

// ============================================================================
// TYPES FOR UI COMPONENT IMPORTS
// ============================================================================

// These interfaces define what UI components the consuming app must provide
export interface UIComponentProps {
    Alert?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    AlertDescription?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    Button?: React.ComponentType<{ className?: string, children?: React.ReactNode, variant?: string, size?: string, disabled?: boolean, onClick?: () => void, [key: string]: unknown }>
    Card?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    CardContent?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    CardDescription?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    CardHeader?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    CardTitle?: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    Badge?: React.ComponentType<{ className?: string, children?: React.ReactNode, variant?: string, [key: string]: unknown }>
    Progress?: React.ComponentType<{ className?: string, value?: number, [key: string]: unknown }>
}

export interface IconComponentProps {
    AlertTriangle?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    ShieldAlert?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    RefreshCw?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    ArrowUp?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    LogIn?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    HelpCircle?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Clock?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Zap?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Shield?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    User?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    CreditCard?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Star?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    CheckCircle?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    XCircle?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Timer?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    TrendingUp?: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Lock?: React.ComponentType<{ className?: string, [key: string]: unknown }>
}

export interface PermissionErrorUIProps {
    error: ApiError
    onRetry?: () => void
    onUpgrade?: (tier?: string) => void
    onLogin?: () => void
    onSupport?: (context?: unknown) => void
    showRetry?: boolean
    showSupport?: boolean
    variant?: 'alert' | 'card' | 'full-page'
    className?: string
    platform?: 'frontend' | 'admin'
    components: UIComponentProps
    icons: IconComponentProps
}
