
import { useCallback, useState } from 'react'
import type { InsufficientTierError } from '../../../utils/response-handler'
import type { IconComponentProps, PermissionErrorUIProps, UIComponentProps } from './types'

interface InsufficientTierUIProps extends Omit<PermissionErrorUIProps, 'error'> {
    error: InsufficientTierError
}

export function InsufficientTierUI(props: InsufficientTierUIProps) {
    const { variant = 'card', components } = props
    const { Card, CardHeader, CardTitle, CardContent, Button } = components

    const hasBasicComponents = Card && CardHeader && CardTitle && CardContent && Button

    if (variant === 'full-page' && hasBasicComponents) {
        return <InsufficientTierFullPage {...props} />
    }

    if (variant === 'card' && hasBasicComponents) {
        return <InsufficientTierCard {...props} />
    }

    // Fallback
    return (
        <div className={`p-4 border border-yellow-200 bg-yellow-50 rounded ${props.className}`}>
            <h3 className="font-bold text-yellow-800">
                {props.platform === 'admin' ? 'Admin Permissions Insufficient' : 'Upgrade Required'}
            </h3>
            <p className="text-yellow-700 mt-2">{props.error.error.user_message}</p>
        </div>
    )
}

function InsufficientTierFullPage(props: InsufficientTierUIProps) {
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

    const [isUpgrading, setIsUpgrading] = useState(false)
    const { Card, CardHeader, CardTitle, CardContent, Badge, Button } = components as Required<UIComponentProps>
    const { Star, ArrowUp, CheckCircle, RefreshCw, HelpCircle } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    const handleUpgrade = useCallback(() => {
        setIsUpgrading(true)
        onUpgrade?.(error.error.required_tier)
    }, [onUpgrade, error.error.required_tier])

    return (
        <div className={`min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-purple-50 ${className}`}>
            <div className="max-w-lg w-full space-y-8 p-6">
                <div className="text-center">
                    <Star className="mx-auto h-12 w-12 text-yellow-500" />
                    <h2 className="mt-6 text-3xl font-bold text-gray-900">
                        {isAdmin ? 'Admin Tier Upgrade Required' : 'Upgrade Required'}
                    </h2>
                    <p className="mt-2 text-lg text-gray-600">{error.error.user_message}</p>
                </div>

                <Card className="border-yellow-200 bg-yellow-50">
                    <CardHeader>
                        <div className="flex items-center justify-between">
                            <div>
                                <CardTitle className="text-yellow-800">Current Plan</CardTitle>
                                <Badge variant="outline" className="mt-1 capitalize">
                                    {error.error.current_tier}
                                </Badge>
                            </div>
                            <ArrowUp className="h-6 w-6 text-yellow-600" />
                            <div>
                                <CardTitle className="text-yellow-800">Required Plan</CardTitle>
                                <Badge className="mt-1 bg-yellow-600 text-white capitalize">
                                    {error.error.required_tier}
                                </Badge>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        {error.error.upgrade_info.benefits.length > 0 && (
                            <div>
                                <h4 className="font-medium text-yellow-800 mb-2">Upgrade Benefits:</h4>
                                <ul className="space-y-1">
                                    {error.error.upgrade_info.benefits.map((benefit) => (
                                        <li key={benefit} className="flex items-start text-sm text-yellow-700">
                                            <CheckCircle className="h-4 w-4 text-green-500 mr-2 mt-0.5 flex-shrink-0" />
                                            {benefit}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        <div className="flex flex-col space-y-2">
                            {!isAdmin && onUpgrade && (
                                <Button
                                    onClick={handleUpgrade}
                                    disabled={isUpgrading}
                                    className="w-full bg-yellow-600 hover:bg-yellow-700"
                                >
                                    {isUpgrading ? (
                                        <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                                    ) : (
                                        <Star className="h-4 w-4 mr-2" />
                                    )}
                                    Upgrade to {error.error.required_tier}
                                </Button>
                            )}

                            {showRetry && onRetry && (
                                <Button variant="outline" onClick={onRetry}>
                                    <RefreshCw className="h-4 w-4 mr-2" />
                                    {isAdmin ? 'Check Admin Permissions' : 'Check Current Plan'}
                                </Button>
                            )}

                            {showSupport && onSupport && (
                                <Button variant="ghost" onClick={() => { onSupport(error); }}>
                                    <HelpCircle className="h-4 w-4 mr-2" />
                                    {isAdmin ? 'Contact System Admin' : 'Contact Sales'}
                                </Button>
                            )}
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    )
}

function InsufficientTierCard(props: InsufficientTierUIProps) {
    const {
        error,
        onRetry,
        onUpgrade,
        onSupport: _onSupport,
        showRetry,
        showSupport: _showSupport,
        className,
        platform,
        components,
        icons
    } = props

    const [isUpgrading, setIsUpgrading] = useState(false)
    const { Card, CardHeader, CardTitle, CardContent, Button } = components as Required<UIComponentProps>
    const { Star, RefreshCw } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    const handleUpgrade = useCallback(() => {
        setIsUpgrading(true)
        onUpgrade?.(error.error.required_tier)
    }, [onUpgrade, error.error.required_tier])

    return (
        <Card className={`border-yellow-200 bg-yellow-50 ${className}`}>
            <CardHeader>
                <div className="flex items-center space-x-2">
                    <Star className="h-5 w-5 text-yellow-500" />
                    <CardTitle className="text-yellow-800">
                        {isAdmin ? 'Admin Permissions Insufficient' : 'Upgrade Required'}
                    </CardTitle>
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                <p className="text-yellow-700">{error.error.user_message}</p>

                <div className="flex flex-wrap gap-2">
                    {!isAdmin && onUpgrade && (
                        <Button
                            onClick={handleUpgrade}
                            disabled={isUpgrading}
                            className="bg-yellow-600 hover:bg-yellow-700"
                        >
                            {isUpgrading ? (
                                <RefreshCw className="h-4 w-4 mr-1 animate-spin" />
                            ) : (
                                <Star className="h-4 w-4 mr-1" />
                            )}
                            Upgrade Now
                        </Button>
                    )}

                    {showRetry && onRetry && (
                        <Button variant="outline" onClick={onRetry}>
                            <RefreshCw className="h-4 w-4 mr-1" />
                            Retry
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
