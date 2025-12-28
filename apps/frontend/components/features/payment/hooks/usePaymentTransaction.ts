import { PAYMENT_ESCROW_ABI } from '@/lib/contracts/PaymentEscrowABI'
import { devLog } from '@/shared/utils'
import { useEffect, useState } from 'react'
import { parseUnits } from 'viem'
import { useWaitForTransactionReceipt, useWriteContract } from 'wagmi'

interface UsePaymentTransactionProps {
    tokenAddress: string | null
    escrowAddress: string | null
    amount: number
    planId: number | string
    onSuccess?: (hash: string) => void
    onError?: (error: string) => void
}

export function usePaymentTransaction({
    tokenAddress,
    escrowAddress,
    amount,
    planId,
    onSuccess,
    onError
}: UsePaymentTransactionProps) {
    const [step, setStep] = useState<'idle' | 'paying' | 'complete'>('idle')

    const {
        writeContract: writePayment,
        data: paymentHash,
        error: paymentError,
        isPending: isPaying
    } = useWriteContract()

    const {
        isLoading: isPaymentConfirming,
        isSuccess: isPaymentConfirmed,
        error: receiptError
    } = useWaitForTransactionReceipt({
        hash: paymentHash,
    })

    // Handle payment confirmation -> success
    useEffect(() => {
        if (isPaymentConfirmed && paymentHash) {
            devLog('✅ Payment confirmed!')
            setStep('complete')
            onSuccess?.(paymentHash)
        }
    }, [isPaymentConfirmed, paymentHash, onSuccess])

    // Handle errors
    useEffect(() => {
        const error = paymentError || receiptError
        if (error) {
            console.error('Payment error:', error)
            const isUserRejection = error.message.toLowerCase().includes('user rejected') ||
                error.message.toLowerCase().includes('user denied')

            if (isUserRejection) {
                devLog('👤 User cancelled payment')
                setStep('idle')
            } else {
                onError?.(`Payment failed: ${error.message}`)
                setStep('idle')
            }
        }
    }, [paymentError, receiptError, onError])

    // Convert UUID plan ID to numeric ID for smart contract
    const getNumericPlanId = (uuidOrId: string | number): bigint => {
        // Plan UUID to numeric ID mapping
        const planIdMap: Record<string, number> = {
            '233e3484-3093-4478-bc75-2c74d8570862': 1, // Free Plan
            'adc7bfb7-16ea-4758-8717-ed5c899f36af': 2, // Starter Plan ($14.99)
            'a5b20d50-6616-41a3-b130-4ed2c28f3063': 3, // Pro Plan ($29.99)
            '3a6ec235-6a7e-4fbd-aa8a-178ad86b5b35': 4, // Enterprise Plan ($99.99)
            '0395b65b-dfaf-4031-8811-bcaf756ac882': 5, // API Developer ($49.99)
        };

        try {
            if (typeof uuidOrId === 'number') {
                return BigInt(uuidOrId);
            } else if (typeof uuidOrId === 'string') {
                if (uuidOrId in planIdMap) {
                    return BigInt(planIdMap[uuidOrId]);
                }
                if (/^\d+$/.test(uuidOrId)) {
                    return BigInt(uuidOrId);
                }
                console.warn('Unknown plan UUID, defaulting to Pro Plan (ID 3):', uuidOrId);
                return 3n;
            }
        } catch (e) {
            console.warn('Failed to convert planId to BigInt, using Pro Plan (ID 3)', e);
        }
        return 3n; // Default to Pro Plan
    };

    const pay = async () => {
        if (!tokenAddress || !escrowAddress) {
            onError?.('Invalid contract addresses')
            return
        }

        try {
            setStep('paying')
            const decimals = 6 // Standard for USDT/USDC
            const transferAmount = parseUnits(amount.toString(), decimals)
            const numericPlanId = getNumericPlanId(planId)

            // Send a small amount of ETH for display purposes (will be refunded)
            const displayAmount = parseUnits(amount.toString(), 18);

            devLog('💸 Executing payment...', {
                planId: numericPlanId,
                tokenAddress,
                amount: transferAmount.toString()
            })

            writePayment({
                address: escrowAddress as `0x${string}`,
                abi: PAYMENT_ESCROW_ABI,
                functionName: 'payWithAmountDisplay',
                args: [numericPlanId, tokenAddress as `0x${string}`, transferAmount],
                value: displayAmount
            })
        } catch (error) {
            console.error('Payment execution failed:', error)
            setStep('idle')
            onError?.(`Payment execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
        }
    }

    return {
        pay,
        paymentHash,
        isPaying,
        isPaymentConfirming,
        isPaymentConfirmed,
        step
    }
}
