/**
 * Hook for direct ERC20 token transfer
 * 
 * This replaces the two-step approve+pay flow with a single transfer.
 * MetaMask will correctly show the token symbol and amount.
 */

import { devLog } from '@/shared/utils'
import { useEffect, useState } from 'react'
import { useWaitForTransactionReceipt, useWriteContract } from 'wagmi'

// ERC20 transfer ABI
export const ERC20_TRANSFER_ABI = [
    {
        type: 'function',
        name: 'transfer',
        inputs: [
            { name: 'to', type: 'address' },
            { name: 'amount', type: 'uint256' }
        ],
        outputs: [{ type: 'bool' }],
        stateMutability: 'nonpayable'
    }
] as const

interface UseDirectTokenTransferProps {
    tokenAddress: string | null
    receiverAddress: string | null
    amount: bigint
    onSuccess?: (hash: string) => void
    onError?: (error: string) => void
}

export function useDirectTokenTransfer({
    tokenAddress,
    receiverAddress,
    amount,
    onSuccess,
    onError
}: UseDirectTokenTransferProps) {
    const [isTransferring, setIsTransferring] = useState(false)

    const {
        writeContract,
        data: txHash,
        error: writeError,
        isPending
    } = useWriteContract()

    const {
        isLoading: isConfirming,
        isSuccess: isConfirmed,
        error: receiptError
    } = useWaitForTransactionReceipt({
        hash: txHash,
    })

    // Handle transfer confirmation
    useEffect(() => {
        if (isConfirmed && txHash) {
            devLog('✅ Transfer confirmed!')
            setIsTransferring(false)
            onSuccess?.(txHash)
        }
    }, [isConfirmed, txHash, onSuccess])

    // Handle errors
    useEffect(() => {
        const error = writeError || receiptError
        if (error) {
            console.error('Transfer error:', error)
            const isUserRejection = error.message.toLowerCase().includes('user rejected') ||
                error.message.toLowerCase().includes('user denied')

            if (isUserRejection) {
                devLog('👤 User cancelled transfer')
            } else {
                onError?.(`Transfer failed: ${error.message}`)
            }
            setIsTransferring(false)
        }
    }, [writeError, receiptError, onError])

    const transfer = async () => {
        if (!tokenAddress || !receiverAddress) {
            onError?.('Invalid token or receiver address')
            return
        }

        if (amount <= 0n) {
            onError?.('Invalid amount')
            return
        }

        try {
            setIsTransferring(true)
            devLog('💸 Initiating token transfer...', {
                tokenAddress,
                receiverAddress,
                amount: amount.toString()
            })

            writeContract({
                address: tokenAddress as `0x${string}`,
                abi: ERC20_TRANSFER_ABI,
                functionName: 'transfer',
                args: [receiverAddress as `0x${string}`, amount],
            })
        } catch (error) {
            console.error('Transfer preparation failed:', error)
            setIsTransferring(false)
            onError?.(`Transfer failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
        }
    }

    return {
        transfer,
        txHash,
        isTransferring: isTransferring || isPending,
        isConfirming,
        isConfirmed
    }
}
