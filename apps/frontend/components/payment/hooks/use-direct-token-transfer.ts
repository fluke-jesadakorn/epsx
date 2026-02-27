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
        error: receiptError,
        data: receiptData
    } = useWaitForTransactionReceipt({
        hash: txHash,
    })

    useEffect(() => {
        if (isConfirmed && txHash) {
            // M8: Verify receipt status is 'success' (not just included in block)
            if (receiptData && receiptData.status !== 'success') {
                onError?.('Transaction was reverted on-chain. Please try again.')
                setIsTransferring(false)
                return
            }
            devLog('✅ Transfer confirmed!')
            setIsTransferring(false)
            onSuccess?.(txHash)
        }
    }, [isConfirmed, txHash, onSuccess, receiptData, onError])

    // Handle errors
    useEffect(() => {
        const error = writeError ?? receiptError
        if (error) {
      // Error logged silently
            const errorMessage = error.message.toLowerCase()

            const isUserRejection = errorMessage.includes('user rejected') ||
                errorMessage.includes('user denied')

            // Detect RPC connection issues (common with local Anvil or rate-limited RPCs)
            const isRpcUnavailable = errorMessage.includes('resource not available') ||
                errorMessage.includes('rpc endpoint returned too many errors') ||
                errorMessage.includes('resourceunavailable') ||
                error.name === 'ResourceUnavailableRpcError'

            const isInsufficientBalance = errorMessage.includes('insufficient') ||
                errorMessage.includes('exceeds balance') ||
                errorMessage.includes('e450d38c')

            if (isUserRejection) {
                devLog('👤 User cancelled transfer')
                onError?.('Transaction cancelled. You can try again when ready.')
            } else if (isInsufficientBalance) {
                onError?.('Insufficient token balance. Please add funds and try again.')
            } else if (isRpcUnavailable) {
                devLog('🔌 RPC connection issue detected')
                onError?.('RPC connection failed. Please check your MetaMask is connected to BNB Smart Chain with a working RPC URL (e.g., https://bsc-dataseed.binance.org).')
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
      // Error logged silently
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
