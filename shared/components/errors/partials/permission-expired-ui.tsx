
import type { PermissionExpiredError } from '../../../utils/response-handler'
import type { IconComponentProps, PermissionErrorUIProps, UIComponentProps } from './types'

interface PermissionExpiredUIProps extends Omit<PermissionErrorUIProps, 'error'> {
    error: PermissionExpiredError
}

const LABELS = {
    ADMIN_ACCESS_EXPIRED: 'Admin Access Expired',
    ACCESS_EXPIRED: 'Access Expired'
}

export function PermissionExpiredUI(props: PermissionExpiredUIProps) {
    const { className, error, platform, components } = props
    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button } = components

    if (Card !== undefined && CardHeader !== undefined && CardTitle !== undefined && CardDescription !== undefined && CardContent !== undefined && Button !== undefined) {
        return <PermissionExpiredCard {...props} />
    }

    // Fallback
    return (
        <div className={`p-4 border border-orange-200 bg-orange-50 rounded ${className}`}>
            <h3 className="font-bold text-orange-800">
                {platform === 'admin' ? LABELS.ADMIN_ACCESS_EXPIRED : LABELS.ACCESS_EXPIRED}
            </h3>
            <p className="text-orange-700 mt-2">{error.error.user_message}</p>
        </div>
    )
}

function PermissionExpiredCard(props: PermissionExpiredUIProps) {
    const {
        error,
        onRetry,
        onUpgrade,
        onSupport,
        showRetry,
        showSupport,
        className,
        platform,
        components,
        icons
    } = props

    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button } = components as Required<UIComponentProps>
    const { Clock, CreditCard, RefreshCw, HelpCircle } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    return (
        <Card className={`border-orange-200 bg-orange-50 ${className}`}>
            <CardHeader>
                <div className="flex items-center space-x-2">
                    <Clock className="h-5 w-5 text-orange-500" />
                    <CardTitle className="text-orange-800">
                        {isAdmin ? LABELS.ADMIN_ACCESS_EXPIRED : LABELS.ACCESS_EXPIRED}
                    </CardTitle>
                </div>
                <CardDescription className="text-orange-700">
                    {error.error.user_message}
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {error.error.expired_permissions.length > 0 && (
                    <div className="bg-orange-100 p-3 rounded-lg">
                        <p className="text-sm font-medium text-orange-800 mb-2">Expired Permissions:</p>
                        <div className="space-y-1">
                            {error.error.expired_permissions.map((perm) => (
                                <div key={perm.permission} className="text-sm text-orange-700">
                                    <code className="bg-orange-200 px-2 py-1 rounded text-xs font-mono">
                                        {perm.permission}
                                    </code>
                                    <span className="ml-2 text-xs">
                                        Expired {new Date(perm.expired_at).toLocaleString()}
                                    </span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                <div className="flex flex-wrap gap-2">
                    {!isAdmin && onUpgrade !== undefined && (
                        <Button onClick={() => onUpgrade()} className="bg-orange-600 hover:bg-orange-700">
                            <CreditCard className="h-4 w-4 mr-1" />
                            Renew Access
                        </Button>
                    )}

                    {showRetry === true && onRetry !== undefined && (
                        <Button variant="outline" onClick={onRetry}>
                            <RefreshCw className="h-4 w-4 mr-1" />
                            Check Status
                        </Button>
                    )}

                    {showSupport === true && onSupport !== undefined && (
                        <Button variant="ghost" onClick={() => { onSupport(error); }}>
                            <HelpCircle className="h-4 w-4 mr-1" />
                            {isAdmin ? 'System Admin' : 'Get Help'}
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
