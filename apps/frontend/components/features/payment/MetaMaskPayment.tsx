'use client'

import { useState, useEffect } from 'react'
import { useAccount, useWriteContract, useWaitForTransactionReceipt } from 'wagmi'
import { parseUnits, formatUnits } from 'viem'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { 
  Wallet, 
  Send, 
  CheckCircle, 
  AlertCircle, 
  Loader2,
  ExternalLink,
  Fuel,
  DollarSign
} from 'lucide-react'

interface MetaMaskPaymentProps {
  planId: number
  planName: string
  amount: number
  currency: string
  onSuccess: (txHash: string) => void
  onError: (error: string) => void
  className?: string
}

// USDT contract addresses on different networks
const USDT_CONTRACTS = {
  ethereum: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
  polygon: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F',
  arbitrum: '0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9',
  base: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913'
} as const

// USDC contract addresses  
const USDC_CONTRACTS = {
  ethereum: '0xA0b86a33E6441986C3E02B9E5a81B4E89F9b6F60',
  polygon: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174',
  arbitrum: '0xaf88d065e77c8cC2239327C5EDb3A432268e5831',
  base: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913'
} as const

// Payment recipient address (should be from env or config)
const PAYMENT_RECIPIENT = process.env.NEXT_PUBLIC_PAYMENT_RECIPIENT_ADDRESS || '0x742B1e18D6b5e2C6FFB9e0Bf6Bd1E8cC9e2B7F9A'

// ERC20 ABI for transfer function
const ERC20_ABI = [
  {
    type: 'function',
    name: 'transfer',
    inputs: [
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [{ type: 'bool' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'balanceOf',
    inputs: [{ name: 'account', type: 'address' }],
    outputs: [{ type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'decimals',
    inputs: [],
    outputs: [{ type: 'uint8' }],
    stateMutability: 'view'
  }
] as const

export default function MetaMaskPayment({
  planId,
  planName,
  amount,
  currency,
  onSuccess,
  onError,
  className = ''
}: MetaMaskPaymentProps) {
  const { address, isConnected, chain } = useAccount()
  const [selectedToken, setSelectedToken] = useState<'USDT' | 'USDC'>('USDT')
  const [estimatedGas, setEstimatedGas] = useState<string>('0.005')
  const [isEstimatingGas, setIsEstimatingGas] = useState(false)

  const { 
    writeContract, 
    data: hash, 
    error: writeError, 
    isPending: isWriting 
  } = useWriteContract()

  const { 
    isLoading: isConfirming, 
    isSuccess: isConfirmed,
    error: receiptError 
  } = useWaitForTransactionReceipt({
    hash,
  })

  // Get contract address based on selected token and current chain
  const getContractAddress = () => {
    const contracts = selectedToken === 'USDT' ? USDT_CONTRACTS : USDC_CONTRACTS
    
    switch (chain?.id) {
      case 1: return contracts.ethereum
      case 137: return contracts.polygon  
      case 42161: return contracts.arbitrum
      case 8453: return contracts.base
      default: return contracts.ethereum // fallback to ethereum
    }
  }

  // Handle successful transaction
  useEffect(() => {
    if (isConfirmed && hash) {
      onSuccess(hash)
    }
  }, [isConfirmed, hash, onSuccess])

  // Handle transaction errors
  useEffect(() => {
    if (writeError) {
      onError(`Transaction failed: ${writeError.message}`)
    }
    if (receiptError) {
      onError(`Transaction receipt error: ${receiptError.message}`)
    }
  }, [writeError, receiptError, onError])

  // Execute the token transfer
  const handlePayment = async () => {
    if (!isConnected || !address) {
      onError('Please connect your wallet first')
      return
    }

    try {
      const contractAddress = getContractAddress()
      const decimals = 6 // USDT and USDC both use 6 decimals
      const transferAmount = parseUnits(amount.toString(), decimals)

      writeContract({
        address: contractAddress as `0x${string}`,
        abi: ERC20_ABI,
        functionName: 'transfer',
        args: [PAYMENT_RECIPIENT as `0x${string}`, transferAmount],
      })
    } catch (error) {
      onError(`Payment preparation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  // Estimate gas costs (simplified)
  const estimateGas = async () => {
    setIsEstimatingGas(true)
    try {
      // Mock gas estimation - in real app, use eth_estimateGas
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      switch (chain?.id) {
        case 1: setEstimatedGas('0.008') // Ethereum mainnet
          break
        case 137: setEstimatedGas('0.002') // Polygon
          break
        case 42161: setEstimatedGas('0.001') // Arbitrum
          break
        case 8453: setEstimatedGas('0.001') // Base
          break
        default: setEstimatedGas('0.005')
      }
    } catch (error) {
      console.error('Gas estimation failed:', error)
    } finally {
      setIsEstimatingGas(false)
    }
  }

  useEffect(() => {
    if (isConnected && chain) {
      estimateGas()
    }
  }, [isConnected, chain, selectedToken])

  if (!isConnected) {
    return (
      <Card className={className}>
        <CardContent className="p-6 text-center space-y-4">
          <Wallet className="w-12 h-12 mx-auto text-muted-foreground" />
          <div>
            <h3 className="font-semibold">Connect Your Wallet</h3>
            <p className="text-sm text-muted-foreground">
              Connect MetaMask to pay with cryptocurrency
            </p>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Wallet className="w-5 h-5" />
          Pay with MetaMask
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Token Selection */}
        <div>
          <label className="text-sm font-medium mb-3 block">Select Token</label>
          <div className="grid grid-cols-2 gap-3">
            {(['USDT', 'USDC'] as const).map((token) => (
              <button
                key={token}
                onClick={() => setSelectedToken(token)}
                className={`p-4 border rounded-lg text-left transition-all ${
                  selectedToken === token
                    ? 'border-primary bg-primary/5'
                    : 'border-muted hover:border-primary/50'
                }`}
              >
                <div className="font-medium">{token}</div>
                <div className="text-sm text-muted-foreground">
                  {token === 'USDT' ? 'Tether USD' : 'USD Coin'}
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Payment Details */}
        <div className="space-y-3 p-4 bg-muted/50 rounded-lg">
          <div className="flex justify-between items-center">
            <span className="text-sm">Plan</span>
            <span className="font-medium">{planName}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm">Amount</span>
            <span className="font-medium">${amount} {selectedToken}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm">Network</span>
            <Badge variant="outline">
              {chain?.name || 'Unknown'}
            </Badge>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm flex items-center gap-1">
              <Fuel className="w-3 h-3" />
              Est. Gas Fee
            </span>
            <span className="font-medium">
              {isEstimatingGas ? (
                <Loader2 className="w-4 h-4" />
              ) : (
                `~${estimatedGas} ETH`
              )}
            </span>
          </div>
          <div className="border-t pt-3">
            <div className="flex justify-between items-center font-medium">
              <span>Total</span>
              <span>${amount} {selectedToken}</span>
            </div>
          </div>
        </div>

        {/* Transaction Status */}
        {hash && (
          <Alert>
            <AlertCircle className="w-4 h-4" />
            <AlertDescription className="flex items-center justify-between">
              <span>
                Transaction {isConfirming ? 'pending' : isConfirmed ? 'confirmed' : 'submitted'}
              </span>
              <a 
                href={`${chain?.blockExplorers?.default?.url}/tx/${hash}`}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-primary hover:underline"
              >
                View <ExternalLink className="w-3 h-3" />
              </a>
            </AlertDescription>
          </Alert>
        )}

        {/* Payment Button */}
        <Button
          onClick={handlePayment}
          disabled={isWriting || isConfirming || isConfirmed}
          className="w-full"
          size="lg"
        >
          {isWriting ? (
            <>
              <Loader2 className="w-4 h-4 mr-2" />
              Preparing Transaction...
            </>
          ) : isConfirming ? (
            <>
              <Loader2 className="w-4 h-4 mr-2" />
              Confirming Payment...
            </>
          ) : isConfirmed ? (
            <>
              <CheckCircle className="w-4 h-4 mr-2" />
              Payment Confirmed
            </>
          ) : (
            <>
              <Send className="w-4 h-4 mr-2" />
              Pay ${amount} {selectedToken}
            </>
          )}
        </Button>

        {/* Security Notice */}
        <Alert>
          <DollarSign className="w-4 h-4" />
          <AlertDescription className="text-sm">
            Your transaction will be processed on the {chain?.name} network. 
            Make sure you have enough {selectedToken} tokens and ETH for gas fees.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}