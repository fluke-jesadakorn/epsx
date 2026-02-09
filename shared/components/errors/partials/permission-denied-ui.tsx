
import type { PermissionDeniedError } from '../../../utils/response-handler';
import type { IconComponentProps, PermissionErrorUIProps, UIComponentProps } from './types';

interface PermissionDeniedUIProps extends Omit<PermissionErrorUIProps, 'error'> {
    error: PermissionDeniedError
}

const LABELS = {
    ADMIN_ACCESS_DENIED: 'Admin Access Denied',
    ACCESS_DENIED: 'Access Denied'
}

const getAlertRenderer = (c: UIComponentProps) => (c.Alert !== undefined && c.AlertDescription !== undefined ? PermissionDeniedAlert : null);
const getFullPageRenderer = (c: UIComponentProps) => (c.Card !== undefined && c.CardContent !== undefined ? PermissionDeniedFullPage : null);
const getCardRenderer = (c: UIComponentProps) => (c.Card !== undefined && c.CardHeader !== undefined && c.CardTitle !== undefined && c.CardContent !== undefined ? PermissionDeniedCard : null);

export function PermissionDeniedUI(props: PermissionDeniedUIProps) {
    const { variant = 'card', components, icons } = props
    const { ShieldAlert, LogIn, RefreshCw } = icons
    const { Button } = components

    if (Button === undefined || ShieldAlert === undefined || LogIn === undefined || RefreshCw === undefined) {
        return <PermissionDeniedFallback {...props} />
    }

    let Renderer: React.ComponentType<PermissionDeniedUIProps> | null = null;
    if (variant === 'alert') {
        Renderer = getAlertRenderer(components);
    } else if (variant === 'full-page') {
        Renderer = getFullPageRenderer(components);
    } else {
        Renderer = getCardRenderer(components);
    }

    const FinalRenderer = Renderer ?? PermissionDeniedFallback;
    return <FinalRenderer {...props} />;
}

function PermissionDeniedFallback({ platform, error, className }: PermissionDeniedUIProps) {
    const title = platform === 'admin' ? LABELS.ADMIN_ACCESS_DENIED : LABELS.ACCESS_DENIED
    return (
        <div className={`p-4 border border-red-200 bg-red-50 rounded ${className}`}>
            <h3 className="font-bold text-red-800">
                {title}
            </h3>
            <p className="text-red-700 mt-2">{error.error.user_message}</p>
        </div>
    )
}

function PermissionDeniedAlert(props: PermissionDeniedUIProps) {
    const {
        error,
        onRetry,
        onLogin,
        showRetry,
        className,
        platform,
        components,
        icons
    } = props

    const { Alert, AlertDescription, Button } = components as Required<UIComponentProps>
    const { ShieldAlert, LogIn, RefreshCw } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    return (
        <Alert className={`border-red-300 bg-red-50 ${className}`}>
            <ShieldAlert className="h-5 w-5 text-red-600" />
            <AlertDescription>
                <div className="flex items-center justify-between">
                    <div>
                        <span className="font-medium text-red-800">
                            {isAdmin ? LABELS.ADMIN_ACCESS_DENIED : LABELS.ACCESS_DENIED}
                        </span>
                        <p className="text-red-700 mt-1">{error.error.user_message}</p>
                    </div>
                    <div className="flex gap-2 ml-4">
                        {onLogin !== undefined && (
                            <Button variant="default" size="sm" onClick={onLogin}>
                                <LogIn className="h-4 w-4 mr-1" />
                                {isAdmin ? 'Admin Sign In' : 'Sign In'}
                            </Button>
                        )}
                        {showRetry === true && onRetry !== undefined && (
                            <Button variant="outline" size="sm" onClick={onRetry}>
                                <RefreshCw className="h-4 w-4" />
                            </Button>
                        )}
                    </div>
                </div>
            </AlertDescription>
        </Alert>
    )
}

function PermissionDeniedFullPage(props: PermissionDeniedUIProps) {
    const {
        error,
        onRetry,
        onLogin,
        onSupport,
        showRetry,
        showSupport,
        className,
        platform,
        components,
        icons
    } = props

    const { Card, CardContent, Button } = components as Required<UIComponentProps>
    const { ShieldAlert, LogIn, RefreshCw, HelpCircle } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    return (
        <div className={`min-h-screen flex items-center justify-center bg-gray-50 ${className}`}>
            <div className="max-w-md w-full space-y-8">
                <div className="text-center">
                    <ShieldAlert className="mx-auto h-12 w-12 text-red-500" />
                    <h2 className="mt-6 text-3xl font-bold text-gray-900">
                        {isAdmin ? LABELS.ADMIN_ACCESS_DENIED : LABELS.ACCESS_DENIED}
                    </h2>
                    <p className="mt-2 text-sm text-gray-600">{error.error.user_message}</p>
                </div>

                <Card>
                    <CardContent className="pt-6">
                        <div className="space-y-4">
                            {error.error.permission !== '' && (
                                <div className="bg-red-50 p-3 rounded-lg">
                                    <p className="text-sm font-medium text-red-800 mb-1">Missing Permission:</p>
                                    <code className="text-sm text-red-700 bg-red-100 px-2 py-1 rounded font-mono">
                                        {error.error.permission}
                                    </code>
                                </div>
                            )}

                            <div className="flex flex-col space-y-2">
                                {onLogin !== undefined && (
                                    <Button onClick={onLogin} className="w-full">
                                        <LogIn className="h-4 w-4 mr-2" />
                                        {isAdmin ? 'Admin Sign In' : 'Sign In to Continue'}
                                    </Button>
                                )}

                                {showRetry === true && onRetry !== undefined && (
                                    <Button variant="outline" onClick={onRetry} className="w-full">
                                        <RefreshCw className="h-4 w-4 mr-2" />
                                        Try Again
                                    </Button>
                                )}

                                {showSupport === true && onSupport !== undefined && (
                                    <Button variant="ghost" onClick={() => { onSupport(error); }} className="w-full">
                                        <HelpCircle className="h-4 w-4 mr-2" />
                                        {isAdmin ? 'Contact System Admin' : 'Contact Support'}
                                    </Button>
                                )}
                            </div>

                            {error.error.suggested_actions.length > 0 && (
                                <div className="border-t pt-4">
                                    <p className="text-sm font-medium text-gray-900 mb-2">Suggested Actions:</p>
                                    <ul className="text-sm text-gray-600 space-y-1">
                                        {error.error.suggested_actions.map((action) => (
                                            <li key={action} className="flex items-start">
                                                <span className="text-gray-400 mr-2">•</span>
                                                {action}
                                            </li>
                                        ))}
                                    </ul>
                                </div>
                            )}
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    )
}

function PermissionDeniedCard(props: PermissionDeniedUIProps) {
    const {
        error,
        onRetry,
        onLogin,
        onSupport,
        showRetry,
        showSupport,
        className,
        platform,
        components,
        icons
    } = props

    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button } = components as Required<UIComponentProps>
    const { ShieldAlert, LogIn, RefreshCw, HelpCircle } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    return (
        <Card className={`border-red-200 ${className}`}>
            <CardHeader>
                <div className="flex items-center space-x-2">
                    <ShieldAlert className="h-5 w-5 text-red-500" />
                    <CardTitle className="text-red-800">
                        {isAdmin ? LABELS.ADMIN_ACCESS_DENIED : LABELS.ACCESS_DENIED}
                    </CardTitle>
                </div>
                <CardDescription className="text-red-600">
                    {error.error.user_message}
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {error.error.permission !== '' && (
                    <div className="bg-red-50 p-3 rounded-lg">
                        <p className="text-sm font-medium text-red-800 mb-1">Missing Permission:</p>
                        <code className="text-sm text-red-700 bg-red-100 px-2 py-1 rounded font-mono">
                            {error.error.permission}
                        </code>
                    </div>
                )}

                <div className="flex flex-wrap gap-2">
                    {onLogin !== undefined && (
                        <Button onClick={onLogin}>
                            <LogIn className="h-4 w-4 mr-1" />
                            {isAdmin ? 'Admin Sign In' : 'Sign In'}
                        </Button>
                    )}

                    {showRetry === true && onRetry !== undefined && (
                        <Button variant="outline" onClick={onRetry}>
                            <RefreshCw className="h-4 w-4 mr-1" />
                            Try Again
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
