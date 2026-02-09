import {
    getTransactionStatusAction,
    submitTransactionAction
} from '@/app/actions/payments'
import { PAYMENT_ESCROW_ABI } from '@/lib/contracts/payment-escrow-abi'
import type { TransactionStatusData } from '@/shared/api/payments'
import { devLog } from '@/shared/utils'
import { useCallback, useEffect, useRef, useState } from 'react'
import { parseUnits } from 'viem'
import { useWaitForTransactionReceipt, useWriteContract } from 'wagmi'

interface UsePaymentTransactionProps {
    tokenAddress: string | null
    escrowAddress: string | null
    amount: number
    planId: number | string
    currency?: string
    onSuccess?: (hash: string) => void
    onError?: (error: string) => void
}

/** Polling interval for transaction status checks - queries backend every 3 seconds */
const STATUS_POLL_INTERVAL_MS = 3000

/** BSC token decimals for USDT/USDC - 18 decimals for BSC-Pegged stablecoins */
const TOKEN_DECIMALS = 18

export function usePaymentTransaction({
    tokenAddress,
    escrowAddress,
    amount,
    planId,
    currency = 'USDT',
    onSuccess,
    onError
}: UsePaymentTransactionProps) {
    const [step, setStep] = useState<'idle' | 'paying' | 'submitted' | 'confirming' | 'complete'>('idle')
    const [txStatus, setTxStatus] = useState<TransactionStatusData | null>(null)
    const pollingRef = useRef<NodeJS.Timeout | null>(null)
    const hasSubmittedRef = useRef(false)

    const {
        writeContract: writePayment,
        data: paymentHash,
        error: paymentError,
        isPending: isPaying,
        status: writeStatus,
    } = useWriteContract()

    // Wait for payment transaction to be confirmed on-chain
    const {
        isLoading: isWaitingForReceipt,
        isSuccess: isPaymentOnChainConfirmed,
        error: receiptError,
    } = useWaitForTransactionReceipt({
        hash: paymentHash,
    })

    // Log wagmi state changes in development
    useEffect(() => {
        devLog('🔄 WAGMI STATE CHANGED:', {
            paymentHash,
            paymentError: paymentError?.message,
            isPaying,
            writeStatus,
            isWaitingForReceipt,
            isPaymentOnChainConfirmed,
            receiptError: receiptError?.message,
            step,
            hasSubmitted: hasSubmittedRef.current
        })
    }, [paymentHash, paymentError, isPaying, writeStatus, isWaitingForReceipt, isPaymentOnChainConfirmed, receiptError, step])

    /**
     * Submit confirmed transaction hash to backend for payment monitoring.
     * Backend validates the transaction and tracks its status until confirmed.
     * Only submits once per transaction (guarded by hasSubmittedRef).
     */
    const submitToBackend = useCallback(async (txHash: string) => {
        if (hasSubmittedRef.current) {
            devLog('⚠️ Already submitted, skipping:', txHash)
            return
        }

        try {
            devLog('📤 Submitting transaction to backend for monitoring...', { txHash, planId, amount, currency })
            hasSubmittedRef.current = true

            const result = await submitTransactionAction({
                transaction_hash: txHash,
                plan_id: planId.toString(),
                expected_amount: amount,
                currency,
            });

            devLog('📥 Backend response:', result)

            if (result.success) {
                devLog('✅ Transaction submitted to backend', result.data)
                setStep('submitted')
                startPolling(txHash)
            } else {
                devLog('❌ Failed to submit to backend:', result.error?.message ?? (result as any).error)
                onError?.(`Failed to submit transaction: ${result.error?.message ?? JSON.stringify((result as any).error)}`)
            }
        } catch (error) {
            devLog('❌ Error submitting to backend:', error)
            // Don't fail the payment - it was successful on-chain
        }

    }, [planId, amount, currency, onError])

    /**
     * Start polling backend for transaction confirmation status.
     * Polls every STATUS_POLL_INTERVAL_MS (3s) until status is 'confirmed' or 'failed'.
     * Stops any existing polling before starting new poll.
     */
    const startPolling = useCallback((txHash: string) => {
        if (pollingRef.current) {
            clearInterval(pollingRef.current)
        }

        const pollStatus = async () => {
            try {
                const result = await getTransactionStatusAction(txHash);

                if (result.success && result.data) {
                    const status = (result.data as any) as TransactionStatusData
                    setTxStatus(status)

                    devLog('📊 Transaction status:', status)

                    if (status.status === 'confirmed') {
                        devLog('✅ Payment confirmed by backend!')
                        setStep('complete')
                        stopPolling()
                        onSuccess?.(txHash)
                    } else if (status.status === 'failed') {
                        devLog('❌ Payment failed:', status.error_message)
                        setStep('idle')
                        stopPolling()
                        onError?.(status.error_message ?? 'Transaction failed on-chain')
                    } else if (status.status === 'confirming') {
                        setStep('confirming')
                    }
                } else if ((result as any).status === 404) {
                    devLog('⏳ Transaction not yet processed by backend')
                } else {
                    devLog('Error polling status:', result.error?.message ?? 'Unknown error')
                }
            } catch (error) {
                devLog('Error polling status:', error)
            }
        }

        pollStatus()
        pollingRef.current = setInterval(pollStatus, STATUS_POLL_INTERVAL_MS)
    }, [onSuccess, onError])

    const stopPolling = useCallback(() => {
        if (pollingRef.current) {
            clearInterval(pollingRef.current)
            pollingRef.current = null
        }
    }, [])

    // Cleanup on unmount
    useEffect(() => {
        return () => stopPolling()
    }, [stopPolling])

    // Submit to backend when the payment is confirmed ON-CHAIN (not just sent)
    useEffect(() => {
        if (paymentHash && isPaymentOnChainConfirmed && !hasSubmittedRef.current) {
            devLog('🎉 Payment confirmed on-chain! Submitting to backend:', paymentHash)
            submitToBackend(paymentHash)
        }
    }, [paymentHash, isPaymentOnChainConfirmed, isWaitingForReceipt, submitToBackend])

    // Handle errors from writeContract
    useEffect(() => {
        if (paymentError) {
            devLog('Payment error:', paymentError)
            const isUserRejection = paymentError.message.toLowerCase().includes('user rejected') ||
                paymentError.message.toLowerCase().includes('user denied')

            if (isUserRejection) {
                devLog('👤 User cancelled payment')
            } else {
                onError?.(`Payment failed: ${paymentError.message}`)
            }
            setStep('idle')
            hasSubmittedRef.current = false
        }
    }, [paymentError, onError])

    /**
     * Convert UUID or numeric plan ID to BigInt for smart contract compatibility.
     * Smart contracts require numeric IDs, so UUIDs are converted via djb2 hash.
     * Throws error if ID format is invalid.
     */
    const getNumericPlanId = (uuidOrId: string | number): bigint => {
        try {
            if (typeof uuidOrId === 'number') {
                return BigInt(uuidOrId)
            }

            if (typeof uuidOrId === 'string') {
                if (/^\d+$/.test(uuidOrId)) {
                    return BigInt(uuidOrId)
                }

                // Convert UUID to deterministic numeric ID using djb2 hash
                const hashUuidToNumber = (uuid: string): bigint => {
                    let hash = 5381n
                    const cleanUuid = uuid.replace(/-/g, '').toLowerCase()
                    for (let i = 0; i < cleanUuid.length; i++) {
                        const char = BigInt(cleanUuid.charCodeAt(i))
                        hash = ((hash << 5n) + hash) + char
                    }
                    return hash & 0xFFFFFFFFn
                }

                const numericId = hashUuidToNumber(uuidOrId)
                devLog('🔢 Converted plan UUID to numeric ID', { uuid: uuidOrId, numericId: numericId.toString() })
                return numericId
            }
        } catch (e) {
            devLog('Failed to convert planId to numeric ID:', e)
        }

        // Throw error instead of using random ID - random IDs could cause payment tracking issues
        throw new Error(`Invalid plan ID format: ${uuidOrId}`)
    }

    const pay = async () => {
        if (!tokenAddress || !escrowAddress) {
            onError?.('Invalid contract addresses')
            return
        }

        try {
            setStep('paying')
            hasSubmittedRef.current = false
            const transferAmount = parseUnits(amount.toString(), TOKEN_DECIMALS)
            const numericPlanId = getNumericPlanId(planId)

            devLog('💸 Executing payment...', {
                planId: numericPlanId,
                tokenAddress,
                amount: transferAmount.toString()
            })

            writePayment({
                address: escrowAddress as `0x${string}`,
                abi: PAYMENT_ESCROW_ABI,
                functionName: 'payForPlan',
                args: [numericPlanId, tokenAddress as `0x${string}`, transferAmount],
            })
        } catch (error) {
            devLog('Payment execution failed:', error)
            setStep('idle')
            onError?.(`Payment execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
        }
    }

    // Derived states for backward compatibility
    const isPaymentConfirming = step === 'submitted' || step === 'confirming' || isWaitingForReceipt || (isPaymentOnChainConfirmed && !hasSubmittedRef.current)
    const isPaymentConfirmed = step === 'complete'

    return {
        pay,
        paymentHash,
        isPaying,
        isPaymentConfirming,
        isPaymentConfirmed,
        isPaymentOnChainConfirmed,
        isWaitingForReceipt,
        step,
        txStatus,
    }
}
