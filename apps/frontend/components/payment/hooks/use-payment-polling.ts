'use client';

import { getTransactionStatusAction } from '@/app/actions/payments';
import { logger } from '@/shared/utils/logger';
import { useCallback, useEffect, useRef } from 'react';
import type { PaymentStep } from './use-payment-flow';

const MAX_POLL_MS = 5 * 60 * 1000;
const POLL_INTERVALS = [3000, 5000, 8000, 12000, 15000];

interface UsePaymentPollingCtx {
    step: PaymentStep;
    address?: string;
    txHash: string | null;
    refetchPlanAccess: () => void;
    setStep: (s: PaymentStep) => void;
    setError: (e: string | null) => void;
}

export function usePaymentPolling({ step, address, txHash, refetchPlanAccess, setStep, setError }: UsePaymentPollingCtx) {
    const pollRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const pollStartRef = useRef<number>(0);
    const pollAttemptRef = useRef<number>(0);
    const visCleanupRef = useRef<(() => void) | null>(null);
    const hasResumedRef = useRef(false);

    const clearPoll = useCallback(() => {
        if (pollRef.current) { clearTimeout(pollRef.current); pollRef.current = null; }
    }, []);

    useEffect(() => () => { clearPoll(); visCleanupRef.current?.(); }, [clearPoll]);

    // Wallet disconnect detection during verification
    useEffect(() => {
        if (step !== 'verifying') { return; }
        if (address) { return; }
        clearPoll();
        setError('Wallet disconnected during verification. Your payment is safe — check your account later.');
    }, [step, address, clearPoll, setError]);

    const startPolling = useCallback((hash: string) => {
        visCleanupRef.current?.();
        pollStartRef.current = Date.now();
        pollAttemptRef.current = 0;

        const schedulePoll = () => {
            const idx = Math.min(pollAttemptRef.current, POLL_INTERVALS.length - 1);
            const delay = POLL_INTERVALS[idx];
            pollRef.current = setTimeout(() => {
                void (async () => {
                    if (typeof document !== 'undefined' && document.hidden) { schedulePoll(); return; }
                    if (Date.now() - pollStartRef.current > MAX_POLL_MS) {
                        clearPoll();
                        setError('Verification is taking longer than expected. Your payment is safe — check your account later or contact support.');
                        return;
                    }
                    try {
                        const status = await getTransactionStatusAction(hash);
                        if (status.success && status.data) {
                            const s = (status.data as { status: string }).status;
                            if (s === 'confirmed') { clearPoll(); setStep('success'); refetchPlanAccess(); return; }
                            if (s === 'failed' || s === 'expired') { clearPoll(); setError('Payment verification failed. Please contact support.'); return; }
                        }
                    } catch (_err) { logger.error('[Payment] Poll status error:', _err); }
                    pollAttemptRef.current += 1;
                    schedulePoll();
                })();
            }, delay);
        };

        const handleVisibility = () => {
            if (!document.hidden && pollRef.current === null && pollStartRef.current > 0) {
                pollAttemptRef.current = 0;
                schedulePoll();
            }
        };
        if (typeof document !== 'undefined') {
            document.addEventListener('visibilitychange', handleVisibility);
            visCleanupRef.current = () => { document.removeEventListener('visibilitychange', handleVisibility); };
        }
        schedulePoll();
    }, [clearPoll, refetchPlanAccess, setStep, setError]);

    // Resume polling on mount if URL has step=verifying + tx hash
    useEffect(() => {
        if (hasResumedRef.current) { return; }
        if (step !== 'verifying' || txHash === null) { return; }
        if (pollStartRef.current > 0) { return; }
        hasResumedRef.current = true;
        startPolling(txHash);
    }, [step, txHash, startPolling]);

    return { startPolling, clearPoll, hasResumedRef, pollStartRef };
}
