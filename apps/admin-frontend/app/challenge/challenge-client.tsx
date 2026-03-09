'use client';

import { TurnstileWidget } from '@/shared/components/turnstile-widget';
import { useCallback, useState } from 'react';
import { verifyTurnstileAction } from './actions';

interface Props {
    from: string;
}

export function ChallengeClient({ from }: Props) {
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const handleSuccess = useCallback(async (token: string) => {
        setLoading(true);
        setError(null);
        try {
            const result = await verifyTurnstileAction(token, from);
            if (result?.error !== undefined) {
                setError(result.error);
                setLoading(false);
            }
        } catch {
            setError('Verification failed. Please try again.');
            setLoading(false);
        }
    }, [from]);

    const handleError = useCallback(() => {
        setError('Verification failed. Please try again.');
        setLoading(false);
    }, []);

    return (
        <div className="min-h-screen flex items-center justify-center bg-background px-4">
            <div className="w-full max-w-sm text-center space-y-6">
                <div className="space-y-2">
                    <h1 className="text-2xl font-bold">Human Verification</h1>
                    <p className="text-muted-foreground text-sm">
                        Please complete the security check to continue.
                    </p>
                </div>

                {loading ? (
                    <p className="text-sm text-muted-foreground">Verifying...</p>
                ) : (
                    <div className="flex justify-center">
                        <TurnstileWidget
                            onSuccess={(token) => { void handleSuccess(token); }}
                            onError={handleError}
                            action="page-gate"
                        />
                    </div>
                )}

                {error !== null && (
                    <p className="text-sm text-red-500">{error}</p>
                )}
            </div>
        </div>
    );
}
