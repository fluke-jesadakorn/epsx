import { useEffect, useState } from 'react'
import type { RateLimitExceededError } from '../../../utils/response-handler'
import type { IconComponentProps, PermissionErrorUIProps, UIComponentProps } from './types'

interface RateLimitExceededUIProps extends Omit<PermissionErrorUIProps, 'error'> {
    error: RateLimitExceededError
}

const LABELS = {
    ADMIN_LIMIT_REACHED: 'Admin Usage Limit Reached',
    LIMIT_REACHED: 'Usage Limit Reached'
}

export function RateLimitExceededUI(props: RateLimitExceededUIProps) {
    const { className, error, platform, components } = props
    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button } = components

    if (Card && CardHeader && CardTitle && CardDescription && CardContent && Button) {
        return <RateLimitExceededCard {...props} />
    }

    // Fallback
    return (
        <div className={`p-4 border border-blue-200 bg-blue-50 rounded ${className}`}>
            <h3 className="font-bold text-blue-800">
                {platform === 'admin' ? LABELS.ADMIN_LIMIT_REACHED : LABELS.LIMIT_REACHED}
            </h3>
            <p className="text-blue-700 mt-2">{error.error.user_message}</p>
        </div>
    )
}

function RateLimitExceededCard(props: RateLimitExceededUIProps) {
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

    const [countdown, setCountdown] = useState(0)
    const { Card, CardHeader, CardTitle, CardDescription, CardContent, Button, Progress } = components as Required<UIComponentProps>
    const { Zap, Timer, TrendingUp, ArrowUp, RefreshCw, HelpCircle } = icons as Required<IconComponentProps>
    const isAdmin = platform === 'admin'

    useEffect(() => {
        const resetTime = new Date(error.error.rate_limit.reset_at).getTime()

        const updateCountdown = () => {
            const now = Date.now()
            const timeLeft = Math.max(0, resetTime - now)
            setCountdown(Math.ceil(timeLeft / 1000))

            if (timeLeft > 0) {
                const timerId = setTimeout(updateCountdown, 1000)
                return () => clearTimeout(timerId)
            }
            return undefined
        }

        return updateCountdown()
    }, [error.error.rate_limit.reset_at])

    const usagePercentage = Math.round(
        ((error.error.rate_limit.limit - error.error.rate_limit.remaining) / error.error.rate_limit.limit) * 100
    )

    return (
        <Card className={`border-blue-200 bg-blue-50 ${className}`}>
            <CardHeader>
                <div className="flex items-center space-x-2">
                    <Zap className="h-5 w-5 text-blue-500" />
                    <CardTitle className="text-blue-800">
                        {isAdmin ? LABELS.ADMIN_LIMIT_REACHED : LABELS.LIMIT_REACHED}
                    </CardTitle>
                </div>
                <CardDescription className="text-blue-700">
                    {error.error.user_message}
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                        <span className="text-blue-700">Usage</span>
                        <span className="text-blue-800 font-mono">
                            {error.error.rate_limit.limit - error.error.rate_limit.remaining} / {error.error.rate_limit.limit}
                        </span>
                    </div>
                    <Progress value={usagePercentage} className="h-2" />
                    <p className="text-xs text-blue-600">
                        {usagePercentage}% of your {error.error.rate_limit.window_size} limit used
                    </p>
                </div>

                {countdown > 0 && (
                    <div className="bg-blue-100 p-3 rounded-lg text-center">
                        <Timer className="h-5 w-5 text-blue-600 mx-auto mb-1" />
                        <p className="text-sm font-medium text-blue-800">
                            Resets in {Math.floor(countdown / 60)}:{(countdown % 60).toString().padStart(2, '0')}
                        </p>
                    </div>
                )}

                {error.error.upgrade_for_higher_limits && !isAdmin && (
                    <div className="bg-gradient-to-r from-blue-100 to-purple-100 p-3 rounded-lg">
                        <div className="flex items-center space-x-2 mb-2">
                            <TrendingUp className="h-4 w-4 text-purple-600" />
                            <span className="text-sm font-medium text-purple-800">Upgrade Available</span>
                        </div>
                        <p className="text-sm text-purple-700">
                            Get {error.error.upgrade_for_higher_limits.new_limit} requests per {error.error.rate_limit.window_size} with {error.error.upgrade_for_higher_limits.tier}
                        </p>
                    </div>
                )}

                <div className="flex flex-wrap gap-2">
                    {!isAdmin && error.error.upgrade_for_higher_limits && onUpgrade && (
                        <Button onClick={() => onUpgrade(error.error.upgrade_for_higher_limits?.tier)}>
                            <ArrowUp className="h-4 w-4 mr-1" />
                            Upgrade for More
                        </Button>
                    )}

                    {showRetry && onRetry && (
                        <Button
                            variant="outline"
                            onClick={onRetry}
                            disabled={countdown > 0}
                        >
                            <RefreshCw className="h-4 w-4 mr-1" />
                            {countdown > 0 ? `Try in ${Math.ceil(countdown / 60)}m` : 'Try Again'}
                        </Button>
                    )}

                    {showSupport && onSupport && (
                        <Button variant="ghost" onClick={() => { onSupport(error); }}>
                            <HelpCircle className="h-4 w-4 mr-1" />
                            {isAdmin ? 'System Admin' : 'Contact Support'}
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
