import { devLog } from '@/shared/utils'
import { useEffect, useState } from 'react'
import { useWaitForTransactionReceipt, useWriteContract } from 'wagmi'

// Minimal ERC20 ABI for approval
export const ERC20_APPROVAL_ABI = [
    {
        type: 'function',
        name: 'approve',
        inputs: [
            { name: 'spender', type: 'address' },
            { name: 'amount', type: 'uint256' }
        ],
        outputs: [{ type: 'bool' }],
        stateMutability: 'nonpayable'
    }
] as const

interface UseTokenApprovalProps {
    tokenAddress: string | null
    spenderAddress: string | null
    amount: bigint
    onSuccess?: (hash: string) => void
    onError?: (error: string) => void
}

export function useTokenApproval({
    tokenAddress,
    spenderAddress,
    amount,
    onSuccess,
    onError
}: UseTokenApprovalProps) {
    const [step, setStep] = useState<'idle' | 'approving' | 'confirmed'>('idle')

    const {
        writeContract: writeApproval,
        data: approvalHash,
        error: approvalError,
        isPending: isApproving
    } = useWriteContract()

    const {
        isLoading: isApprovalConfirming,
        isSuccess: isApprovalConfirmed,
        error: receiptError
    } = useWaitForTransactionReceipt({
        hash: approvalHash,
    })

    // Handle approval confirmation -> success
    useEffect(() => {
        if (isApprovalConfirmed && approvalHash) {
            devLog('✅ Approval confirmed!')
            setStep('confirmed')
            onSuccess?.(approvalHash)
        }
    }, [isApprovalConfirmed, approvalHash, onSuccess])

    // Handle errors
    useEffect(() => {
        const error = approvalError ?? receiptError
        if (error) {
            console.error('Approval error:', error)
            const isUserRejection = error.message.toLowerCase().includes('user rejected') ||
                error.message.toLowerCase().includes('user denied')

            if (isUserRejection) {
                devLog('👤 User cancelled approval')
                setStep('idle')
            } else {
                onError?.(`Approval failed: ${error.message}`)
                setStep('idle')
            }
        }
    }, [approvalError, receiptError, onError])

    const approve = async () => {
        if (!tokenAddress || !spenderAddress) {
            onError?.('Invalid token or spender address')
            return
        }

        try {
            setStep('approving')
            devLog('⏳ Approving token spending...', { tokenAddress, spenderAddress, amount: amount.toString() })

            writeApproval({
                address: tokenAddress as `0x${string}`,
                abi: ERC20_APPROVAL_ABI,
                functionName: 'approve',
                args: [spenderAddress as `0x${string}`, amount],
            })
        } catch (error) {
            console.error('Approval preparation failed:', error)
            setStep('idle')
            onError?.(`Approval preparation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
        }
    }

    return {
        approve,
        approvalHash,
        isApproving,
        isApprovalConfirming,
        isApprovalConfirmed,
        step
    }
}
