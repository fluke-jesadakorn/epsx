'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Loader2, RefreshCw, ShieldAlert, ShieldCheck } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { expireAccessToken, expireRefreshToken, getServerSessionStatus } from './actions';

interface ServerStatus {
    hasAccessToken: boolean;
    hasRefreshToken: boolean;
    accessTokenValue?: string | null;
    refreshTokenValue?: string | null;
}

interface ApiResult {
    success?: boolean;
    data?: Record<string, unknown>;
    error?: string;
}

// eslint-disable-next-line max-lines-per-function, complexity
export default function AuthDebugPage() {
    const { user, isAuthenticated, logout, refreshSession, makeApiRequest } = useSharedAuth();
    const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);
    const [loading, setLoading] = useState(false);
    const [apiResult, setApiResult] = useState<ApiResult | null>(null);

    const fetchServerStatus = useCallback(async () => {
        try {
            const status = await getServerSessionStatus();
            setServerStatus(status);
        } catch {
            // Silently fail
        }
    }, []);

    useEffect(() => {
        void fetchServerStatus();
    }, [fetchServerStatus]);

    const handleExpireAccess = useCallback(async () => {
        setLoading(true);
        await expireAccessToken();
        await fetchServerStatus();
        setLoading(false);
        // eslint-disable-next-line no-alert
        alert('Access Token Expired. Next API call should trigger refresh.');
    }, [fetchServerStatus]);

    const handleExpireRefresh = useCallback(async () => {
        setLoading(true);
        await expireRefreshToken();
        await fetchServerStatus();
        setLoading(false);
        // eslint-disable-next-line no-alert
        alert('Refresh Token Expired. Next API call/refresh should fail and logout.');
    }, [fetchServerStatus]);

    const handleTestApi = useCallback(async () => {
        setLoading(true);
        setApiResult(null);
        try {
            // Call a simple protected endpoint
            const res = await makeApiRequest('/api/auth/session');
            setApiResult({
                success: res.success as boolean | undefined,
                data: res.data as Record<string, unknown> | undefined,
                error: res.error as string | undefined
            });
            // Refresh server view
            await fetchServerStatus();
        } catch (err) {
            setApiResult({ error: err instanceof Error ? err.message : 'Unknown error' });
        } finally {
            setLoading(false);
        }
    }, [fetchServerStatus, makeApiRequest]);

    return (
        <div className="container py-10 space-y-8">
            <div className="flex justify-between items-center">
                <div>
                    <h1 className="text-3xl font-bold">Authentication Debugger</h1>
                    <p className="text-muted-foreground">Inspect and manipulate auth state for verification</p>
                </div>
                <Button variant="outline" onClick={() => window.location.reload()}>
                    <RefreshCw className="mr-2 h-4 w-4" /> Reload Page
                </Button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* CLIENT STATE */}
                <Card>
                    <CardHeader>
                        <CardTitle>Client State (React)</CardTitle>
                        <CardDescription>What the browser sees (useSharedAuth)</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="flex items-center gap-2">
                            Status:
                            {isAuthenticated ? (
                                <span className="text-green-500 font-bold flex items-center"><ShieldCheck className="h-4 w-4 mr-1" /> Authenticated</span>
                            ) : (
                                <span className="text-red-500 font-bold flex items-center"><ShieldAlert className="h-4 w-4 mr-1" /> Unauthenticated</span>
                            )}
                        </div>
                        <pre className="bg-muted p-4 rounded-md overflow-auto text-xs max-h-[300px]">
                            {JSON.stringify(user, null, 2)}
                        </pre>
                        <div className="flex gap-2">
                            <Button variant="destructive" size="sm" onClick={() => void logout()}>Client Logout</Button>
                            <Button variant="secondary" size="sm" onClick={() => void refreshSession()}>Force Refresh</Button>
                        </div>
                    </CardContent>
                </Card>

                {/* SERVER STATE */}
                <Card>
                    <CardHeader>
                        <CardTitle>Server State (HttpOnly)</CardTitle>
                        <CardDescription>What the server sees (cookies)</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                            <div className="p-3 border rounded-md">
                                <div className="text-sm font-semibold mb-1">Access Token</div>
                                {serverStatus?.hasAccessToken === true ? (
                                    <div className="text-green-500 text-xs break-all">Present ({serverStatus.accessTokenValue ?? ''})</div>
                                ) : (
                                    <div className="text-red-500 text-xs">Missing / Expired</div>
                                )}
                            </div>
                            <div className="p-3 border rounded-md">
                                <div className="text-sm font-semibold mb-1">Refresh Token</div>
                                {serverStatus?.hasRefreshToken === true ? (
                                    <div className="text-green-500 text-xs break-all">Present ({serverStatus.refreshTokenValue ?? ''})</div>
                                ) : (
                                    <div className="text-red-500 text-xs">Missing / Expired</div>
                                )}
                            </div>
                        </div>
                        <Button variant="outline" size="sm" onClick={() => { void fetchServerStatus(); }} className="w-full">
                            Refresh Server Status
                        </Button>
                    </CardContent>
                </Card>
            </div>

            {/* CONTROLS */}
            <Card className="border-orange-200 bg-orange-50/10 dark:border-orange-900">
                <CardHeader>
                    <CardTitle>Simulation Controls</CardTitle>
                    <CardDescription>Force specific states to verify system resilience</CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="flex flex-wrap gap-4">
                        <Button
                            variant="secondary"
                            onClick={() => { void handleExpireAccess(); }}
                            disabled={loading || serverStatus?.hasAccessToken !== true}
                        >
                            Expire Access Token
                        </Button>
                        <Button
                            variant="secondary"
                            onClick={() => { void handleExpireRefresh(); }}
                            disabled={loading || serverStatus?.hasRefreshToken !== true}
                        >
                            Expire Refresh Token
                        </Button>
                        <div className="w-px bg-border h-10 mx-2" />
                        <Button onClick={() => { void handleTestApi(); }} disabled={loading}>
                            {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            Test Protected API Call
                        </Button>
                    </div>

                    {apiResult && (
                        <div className="mt-6">
                            <div className="text-sm font-semibold mb-2">API Result:</div>
                            <div className={`p-4 rounded-md border ${apiResult.success === true ? 'bg-green-50/10 border-green-200 text-green-700' : 'bg-red-50/10 border-red-200 text-red-700'}`}>
                                <div className="font-mono text-xs">
                                    {JSON.stringify(apiResult, null, 2)}
                                </div>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}
