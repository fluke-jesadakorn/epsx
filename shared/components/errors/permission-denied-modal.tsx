'use client'

/**
 * SHARED Permission Denied Modal
 * Pure UI component displaying backend permission errors
 * Shows upgrade paths, benefits, and actions from backend
 * 
 * Uses dependency injection for UI components to avoid cross-package imports
 */

import { useRouter } from 'next/navigation'
import type { BackendPermissionError } from '../../utils/permission-error-handler'

// Platform type for customization
export type Platform = 'admin' | 'frontend'

// ============================================================================
// COMPONENT PROPS INTERFACES
// ============================================================================

export interface UIComponents {
    Badge: React.ComponentType<{ className?: string, children?: React.ReactNode, variant?: string, [key: string]: unknown }>
    Button: React.ComponentType<{ className?: string, children?: React.ReactNode, variant?: string, onClick?: () => void, [key: string]: unknown }>
    Dialog: React.ComponentType<{ open?: boolean, onOpenChange?: (open: boolean) => void, children?: React.ReactNode, [key: string]: unknown }>
    DialogContent: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    DialogDescription: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    DialogFooter: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    DialogHeader: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
    DialogTitle: React.ComponentType<{ className?: string, children?: React.ReactNode, [key: string]: unknown }>
}

export interface IconComponents {
    AlertTriangle: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Lock: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Shield: React.ComponentType<{ className?: string, [key: string]: unknown }>
    Zap: React.ComponentType<{ className?: string, [key: string]: unknown }>
}

export interface PermissionDeniedModalProps {
    error: BackendPermissionError
    open: boolean
    onClose: () => void
    platform?: Platform
    components: UIComponents
    icons: IconComponents
}

// ============================================================================
// PLATFORM-SPECIFIC LABELS
// ============================================================================

interface PlatformLabels {
    upgradeTitle: string
    accessTitle: string
    currentGroupLabel: string
    requiredGroupLabel: string
    usageLabel: string
    expiredLabel: string
    renewLabel: string
    upgradeLabel: string
    securityReason: string
}

const getPlatformLabels = (platform: Platform): PlatformLabels => ({
    upgradeTitle: platform === 'admin' ? 'Admin Upgrade Required' : 'Upgrade Required',
    accessTitle: platform === 'admin' ? 'Admin Access Restricted' : 'Access Restricted',
    currentGroupLabel: platform === 'admin' ? 'Current Admin Group:' : 'Current Plan:',
    requiredGroupLabel: platform === 'admin' ? 'Required Admin Group:' : 'Required Plan:',
    usageLabel: platform === 'admin' ? 'Admin Usage:' : 'Usage:',
    expiredLabel: platform === 'admin' ? 'Admin permission expired:' : 'Expired:',
    renewLabel: platform === 'admin' ? 'Renew Admin Access' : 'Renew Access',
    upgradeLabel: platform === 'admin' ? 'Upgrade Admin Access' : 'Upgrade Now',
    securityReason: platform === 'admin'
        ? 'This admin action has been restricted for security reasons.'
        : 'This action has been restricted for security reasons.'
})

// ============================================================================
// SHARED PERMISSION DENIED MODAL COMPONENT
// ============================================================================

export function SharedPermissionDeniedModal({
    error,
    open,
    onClose,
    platform = 'frontend',
    components,
    icons
}: PermissionDeniedModalProps) {
    const {
        Badge, Button, Dialog, DialogContent, DialogDescription,
        DialogFooter, DialogHeader, DialogTitle
    } = components
    const { AlertTriangle, Lock, Shield, Zap } = icons
    const labels = getPlatformLabels(platform)
    const router = useRouter()

    const handleUpgrade = () => {
        if (typeof error.upgrade_url === 'string' && error.upgrade_url !== '') {
            router.push(error.upgrade_url)
        } else {
            router.push('/plans')
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
        if (!error.risk_level) { return null }
        const colors = {
            low: 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300',
            medium: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300',
            high: 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300',
            critical: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300'
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
                        {error.error_type === 'InsufficientGroup' ? labels.upgradeTitle : labels.accessTitle}
                    </DialogTitle>
                    <DialogDescription className="text-center">
                        {error.message}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4">
                    {/* Current vs Required Group */}
                    <GroupInfo error={error} Badge={Badge} labels={labels} />

                    {/* Usage Limit Info */}
                    <UsageLimitInfo error={error} />

                    {/* Benefits List */}
                    <BenefitsList benefits={error.benefits} />

                    {/* Suggested Actions */}
                    <SuggestedActionsList actions={error.suggested_actions} />

                    {/* Security Restriction Warning */}
                    <SecurityWarning error={error} labels={labels} getRiskBadge={getRiskBadge} />

                    {/* Permission Expired */}
                    <ExpirationInfo error={error} labels={labels} />
                </div>

                <DialogFooter className="flex gap-2">
                    <Button variant="outline" onClick={onClose}>
                        Close
                    </Button>
                    {(error.upgrade_url !== undefined || error.renewal_url !== undefined) && (
                        <Button
                            onClick={handleUpgrade}
                            className="bg-gradient-to-r from-orange-500 to-pink-500 hover:from-orange-600 hover:to-pink-600"
                        >
                            {error.renewal_url !== undefined ? labels.renewLabel : labels.upgradeLabel}
                        </Button>
                    )}
                    {Boolean(error.contact_support) && (
                        <Button variant="outline" onClick={() => { router.push('/support'); }}>
                            Contact Support
                        </Button>
                    )}
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function GroupInfo({ error, Badge, labels }: {
    error: BackendPermissionError,
    Badge: UIComponents['Badge'],
    labels: PlatformLabels
}) {
    const hasCurrent = error.current_group !== undefined && error.current_group !== '';
    const hasRequired = error.required_group !== undefined && error.required_group !== '';
    if (!hasCurrent || !hasRequired) { return null }
    return (
        <div className="bg-slate-50 dark:bg-slate-900 p-4 rounded-lg space-y-2">
            <div className="flex justify-between items-center">
                <span className="text-sm text-slate-600 dark:text-slate-400">{labels.currentGroupLabel}</span>
                <Badge variant="outline">{error.current_group}</Badge>
            </div>
            <div className="flex justify-between items-center">
                <span className="text-sm text-slate-600 dark:text-slate-400">{labels.requiredGroupLabel}</span>
                <Badge className="bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300">
                    {error.required_group}
                </Badge>
            </div>
        </div>
    )
}

function UsageLimitInfo({ error }: { error: BackendPermissionError }) {
    if (error.current_usage === undefined || error.limit === undefined) { return null }
    return (
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
            {error.reset_at !== undefined && error.reset_at !== '' && (
                <p className="text-xs text-slate-500 dark:text-slate-400 mt-2">
                    Resets: {new Date(error.reset_at).toLocaleString()}
                </p>
            )}
        </div>
    )
}

function BenefitsList({ benefits }: { benefits?: string[] }) {
    if (benefits === undefined || benefits.length === 0) { return null }
    return (
        <div>
            <h4 className="font-semibold text-sm mb-2 text-slate-900 dark:text-slate-100">Upgrade Benefits:</h4>
            <ul className="space-y-1">
                {benefits.map((benefit) => (
                    <li key={benefit} className="text-sm text-slate-600 dark:text-slate-400 flex items-start">
                        <span className="text-green-500 mr-2">✓</span>
                        {benefit}
                    </li>
                ))}
            </ul>
        </div>
    )
}

function SuggestedActionsList({ actions }: { actions?: string[] }) {
    if (actions === undefined || actions.length === 0) { return null }
    return (
        <div>
            <h4 className="font-semibold text-sm mb-2 text-slate-900 dark:text-slate-100">Suggested Actions:</h4>
            <ul className="space-y-1">
                {actions.map((action) => (
                    <li key={action} className="text-sm text-slate-600 dark:text-slate-400 flex items-start">
                        <span className="text-blue-500 mr-2">→</span>
                        {action}
                    </li>
                ))}
            </ul>
        </div>
    )
}

function SecurityWarning({ error, labels, getRiskBadge }: {
    error: BackendPermissionError,
    labels: PlatformLabels,
    getRiskBadge: () => React.ReactNode
}) {
    if (error.error_type !== 'SecurityRestriction') { return null }
    return (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-4 rounded-lg">
            <div className="flex items-start gap-2">
                {getRiskBadge()}
            </div>
            <p className="text-sm text-red-800 dark:text-red-200 mt-2">
                {error.reason ?? labels.securityReason}
            </p>
            {error.contact_support === true && (
                <p className="text-xs text-red-600 dark:text-red-400 mt-2">
                    Please contact support if you believe this is an error.
                </p>
            )}
        </div>
    )
}

function ExpirationInfo({ error, labels }: {
    error: BackendPermissionError,
    labels: PlatformLabels
}) {
    if (error.error_type !== 'PermissionExpired' || error.expired_at === undefined || error.expired_at === '') { return null }
    return (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 p-4 rounded-lg">
            <p className="text-sm text-yellow-800 dark:text-yellow-200">
                {labels.expiredLabel} {new Date(error.expired_at).toLocaleString()}
            </p>
        </div>
    )
}

