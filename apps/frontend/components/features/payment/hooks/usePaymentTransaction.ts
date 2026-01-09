import { PAYMENT_ESCROW_ABI } from '@/lib/contracts/PaymentEscrowABI'
import type { TransactionStatusData } from '@/shared/api/payments'
import { devLog } from '@/shared/utils'
import { useCallback, useEffect, useRef, useState } from 'react'
import { parseUnits } from 'viem'
import { useWaitForTransactionReceipt, useWriteContract } from 'wagmi'
import { env } from '../../../../../../shared/env/schema'

interface UsePaymentTransactionProps {
    tokenAddress: string | null
    escrowAddress: string | null
    amount: number
    planId: number | string
    currency?: string
    onSuccess?: (hash: string) => void
    onError?: (error: string) => void
}

// Note: Using TransactionStatusData from @/shared/api/payments

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
        data: receipt,
    } = useWaitForTransactionReceipt({
        hash: paymentHash,
    })

    // Debug: Log wagmi state changes
    useEffect(() => {
        console.log('🔄 WAGMI STATE CHANGED:', {
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
     * Submit transaction to backend for monitoring
     */
    const submitToBackend = useCallback(async (txHash: string) => {
        if (hasSubmittedRef.current) {
            console.log('⚠️ [Debug] Already submitted, skipping:', txHash)
            return
        }

        console.log('🚀 [Debug] submitToBackend called with:', txHash)

        try {
            const submitUrl = `${env.BACKEND_URL}/api/payments/submit`;
            console.log('📤 [Debug] Submitting to:', submitUrl, { txHash, planId, amount, currency })
            devLog('📤 Submitting transaction to backend for monitoring...', { txHash })
            hasSubmittedRef.current = true

            const requestBody = {
                transaction_hash: txHash,
                plan_id: planId.toString(),
                expected_amount: amount,
                currency,
            }
            console.log('📦 [Debug] Request body:', JSON.stringify(requestBody))

            const response = await fetch(submitUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                credentials: 'include',
                body: JSON.stringify(requestBody),
            })

            console.log('📥 [Debug] Response status:', response.status)
            const result = await response.json()
            console.log('📥 [Debug] Response body:', result)

            if (result.success) {
                console.log('✅ [Debug] Transaction submitted to backend', result.data)
                devLog('✅ Transaction submitted to backend', result.data)
                setStep('submitted')
                // Start polling for status
                startPolling(txHash)
            } else {
                console.error('❌ [Debug] Failed to submit to backend:', result.message || result.error)
                // Continue anyway - the old confirm flow will handle it
                onError?.(`Failed to submit transaction: ${result.message || JSON.stringify(result.error)}`)
            }
        } catch (error) {
            console.error('❌ [Debug] Error submitting to backend:', error)
            // Don't fail the payment - it was successful on-chain
        }

    }, [planId, amount, currency, onError])

    /**
     * Poll backend for transaction status
     */
    const startPolling = useCallback((txHash: string) => {
        if (pollingRef.current) {
            clearInterval(pollingRef.current)
        }

        const pollStatus = async () => {
            try {
                const statusUrl = `${env.BACKEND_URL}/api/payments/status/${txHash}`;
                console.log('🔍 [Debug] Polling status from:', statusUrl);

                const response = await fetch(statusUrl, {
                    credentials: 'include',
                })

                if (!response.ok) {
                    if (response.status === 404) {
                        devLog('⏳ [Debug] Transaction not yet processed by backend (404)')
                        return
                    }
                    throw new Error(`Status check failed: ${response.status}`)
                }

                const result = await response.json()
                // console.log('📊 [Debug] Poll result:', result)

                if (result.success) {
                    const status = result.data as TransactionStatusData
                    setTxStatus(status)

                    devLog('📊 Transaction status:', status)

                    if (status.status === 'confirmed') {
                        devLog('✅ Payment confirmed by backend!')
                        console.log('✅ [Debug] Payment confirmed! calling onSuccess')
                        setStep('complete')
                        stopPolling()
                        onSuccess?.(txHash)
                    } else if (status.status === 'failed') {
                        devLog('❌ Payment failed:', status.error_message)
                        setStep('idle')
                        stopPolling()
                        onError?.(status.error_message || 'Transaction failed on-chain')
                    } else if (status.status === 'confirming') {
                        setStep('confirming')
                    }
                }
            } catch (error) {
                console.error('Error polling status:', error)
            }
        }

        // Poll immediately, then every 3 seconds
        pollStatus()
        pollingRef.current = setInterval(pollStatus, 3000)
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
        console.log('🔍 Checking on-chain confirmation state:', {
            paymentHash,
            isPaymentOnChainConfirmed,
            isWaitingForReceipt,
            step,
            hasSubmitted: hasSubmittedRef.current
        })

        // Only submit to backend AFTER the transaction is confirmed on-chain
        if (paymentHash && isPaymentOnChainConfirmed && !hasSubmittedRef.current) {
            console.log('🎉 Payment confirmed on-chain! Submitting to backend:', paymentHash)
            devLog('🎉 Payment confirmed on-chain! Submitting to backend:', paymentHash)
            submitToBackend(paymentHash)
        }
    }, [paymentHash, isPaymentOnChainConfirmed, isWaitingForReceipt, submitToBackend])

    // Handle errors from writeContract
    useEffect(() => {
        if (paymentError) {
            console.error('Payment error:', paymentError)
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
     * Convert UUID plan ID to numeric ID for smart contract
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
            console.error('Failed to convert planId to numeric ID:', e)
        }

        console.warn('Using fallback random plan ID')
        return BigInt(Math.floor(Math.random() * 1000000) + 1)
    }

    const pay = async () => {
        if (!tokenAddress || !escrowAddress) {
            onError?.('Invalid contract addresses')
            return
        }

        try {
            setStep('paying')
            hasSubmittedRef.current = false
            const decimals = 6
            const transferAmount = parseUnits(amount.toString(), decimals)
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
            console.error('Payment execution failed:', error)
            setStep('idle')
            onError?.(`Payment execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
        }
    }

    // Derived states for backward compatibility
    // isPaymentConfirming includes: waiting for on-chain receipt, waiting for backend submission, and backend confirming
    const isPaymentConfirming = step === 'submitted' || step === 'confirming' || isWaitingForReceipt || (isPaymentOnChainConfirmed && !hasSubmittedRef.current)
    // isPaymentConfirmed should ONLY be true when backend has confirmed the transaction
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
