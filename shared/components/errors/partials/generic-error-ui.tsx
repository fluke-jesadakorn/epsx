
import type { ApiError } from '../../../utils/response-handler'
import type { PermissionErrorUIProps, UIComponentProps } from './types'

interface GenericErrorUIProps extends Omit<PermissionErrorUIProps, 'error'> {
    error: ApiError
}

export function GenericErrorUI(props: GenericErrorUIProps) {
    const { className, error, platform, components } = props
    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button } = components
    const isAdmin = platform === 'admin'

    if (Card !== undefined && CardHeader !== undefined && CardTitle !== undefined && CardDescription !== undefined && CardContent !== undefined && Button !== undefined) {
        return <GenericErrorCard {...props} />
    }

    // Fallback
    return (
        <div className={`p-4 border border-gray-200 bg-gray-50 rounded ${className}`}>
            <h3 className="font-bold text-gray-800">
                {isAdmin ? 'Admin Operation Failed' : 'Something Went Wrong'}
            </h3>
            <p className="text-gray-600 mt-2">{error.error.user_message}</p>
        </div>
    )
}

function GenericErrorCard(props: GenericErrorUIProps) {
    const {
        error,
        onRetry,
        onSupport,
        showRetry,
        showSupport,
        className,
        platform,
        components,
        icons
    } = props

    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button } = components as Required<UIComponentProps>
    const { AlertTriangle, RefreshCw, HelpCircle } = icons
    const isAdmin = platform === 'admin'

    return (
        <Card className={`border-gray-200 ${className}`}>
            <CardHeader>
                <div className="flex items-center space-x-2">
                    {AlertTriangle !== undefined && <AlertTriangle className="h-5 w-5 text-gray-500" />}
                    <CardTitle className="text-gray-800">
                        {isAdmin ? 'Admin Operation Failed' : 'Something Went Wrong'}
                    </CardTitle>
                </div>
                <CardDescription className="text-gray-600">
                    {error.error.user_message}
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {error.error.suggested_actions.length > 0 && (
                    <div>
                        <p className="text-sm font-medium text-gray-800 mb-2">Suggested Actions:</p>
                        <ul className="space-y-1">
                            {error.error.suggested_actions.map((action) => (
                                <li key={action} className="flex items-start text-sm text-gray-600">
                                    <span className="text-gray-400 mr-2">•</span>
                                    {action}
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                <div className="flex flex-wrap gap-2">
                    {showRetry === true && onRetry !== undefined && (
                        <Button variant="outline" onClick={onRetry}>
                            {RefreshCw !== undefined && <RefreshCw className="h-4 w-4 mr-1" />}
                            Try Again
                        </Button>
                    )}

                    {showSupport === true && onSupport !== undefined && (
                        <Button variant="ghost" onClick={() => onSupport(error)}>
                            {HelpCircle !== undefined && <HelpCircle className="h-4 w-4 mr-1" />}
                            {isAdmin ? 'Technical Support' : 'Get Help'}
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
