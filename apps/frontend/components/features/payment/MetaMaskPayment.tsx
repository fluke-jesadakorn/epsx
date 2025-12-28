'use client'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { getPaymentEscrowAddress, getTokenAddress, isPaymentEscrowDeployed } from '@/lib/contracts/addresses'
import { devLog } from '@/shared/utils'
import {
  AlertCircle,
  CheckCircle,
  DollarSign,
  Fuel,
  Loader2,
  Send,
  Wallet
} from 'lucide-react'
import { useEffect, useState } from 'react'
import { getAddress, parseUnits } from 'viem'
import { useAccount, useConnect } from 'wagmi'
import { usePaymentTransaction } from './hooks/usePaymentTransaction'
import { useTokenApproval } from './hooks/useTokenApproval'
import { PaymentProgress } from './PaymentProgress'

interface MetaMaskPaymentProps {
  planId: number | string
  planName: string
  amount: number
  currency: string
  onSuccess: (txHash: string) => void
  onError: (error: string) => void
  className?: string
}

export default function MetaMaskPayment({
  planId,
  planName,
  amount,
  currency,
  onSuccess,
  onError,
  className = ''
}: MetaMaskPaymentProps) {
  // Early validation
  if (!amount || amount === 0 || isNaN(amount)) {
    return (
      <Card className={className}>
        <CardContent className="p-6 text-center text-red-500">
          <AlertCircle className="w-12 h-12 mx-auto mb-2" />
          <h3 className="font-semibold">Invalid Payment Amount</h3>
        </CardContent>
      </Card>
    )
  }

  const { address, isConnected, chain } = useAccount()
  const { connect, connectors } = useConnect()
  const [selectedToken, setSelectedToken] = useState<'USDT' | 'USDC'>('USDT')
  const [estimatedGas, setEstimatedGas] = useState<string>('0.005')
  const [autoConnectAttempted, setAutoConnectAttempted] = useState(false)

  // Contract Addresses
  const getTokenContractAddress = () => {
    if (!chain?.id) return null
    try {
      return getAddress(getTokenAddress(selectedToken, chain.id))
    } catch (e) { return null }
  }

  const getEscrowContractAddress = () => {
    if (!chain?.id || !isPaymentEscrowDeployed(chain.id)) return null
    try {
      return getAddress(getPaymentEscrowAddress(chain.id))
    } catch (e) { return null }
  }

  const tokenAddress = getTokenContractAddress()
  const escrowAddress = getEscrowContractAddress()

  // Hooks
  const {
    approve,
    approvalHash,
    isApproving,
    isApprovalConfirming,
    isApprovalConfirmed,
    step: approvalStep
  } = useTokenApproval({
    tokenAddress,
    spenderAddress: escrowAddress,
    amount: parseUnits(amount.toString(), 6), // 6 decimals for USDT/USDC
    onError
  })

  const {
    pay,
    paymentHash,
    isPaying,
    isPaymentConfirming,
    isPaymentConfirmed,
    step: paymentStep
  } = usePaymentTransaction({
    tokenAddress,
    escrowAddress,
    amount,
    planId,
    onSuccess,
    onError
  })

  // Auto-trigger payment after approval
  useEffect(() => {
    if (isApprovalConfirmed && paymentStep === 'idle' && !isPaying && !isPaymentConfirmed) {
      devLog('🔄 Approval confirmed, triggering payment...')
      pay()
    }
  }, [isApprovalConfirmed, paymentStep, isPaying, isPaymentConfirmed])

  // Get gas token symbol
  const getGasTokenSymbol = () => {
    if (chain?.id === 56 || chain?.id === 97) return 'BNB'
    return 'ETH'
  }

  // Auto-connect
  useEffect(() => {
    if (!isConnected && !autoConnectAttempted && connectors.length > 0) {
      setAutoConnectAttempted(true)
      const attemptConnect = async () => {
        const mm = connectors.find(c => c.name === 'MetaMask' || c.id === 'metaMask')
        if (mm) await connect({ connector: mm })
        else if (connectors[0]) await connect({ connector: connectors[0] })
      }
      setTimeout(attemptConnect, 500)
    }
  }, [isConnected, autoConnectAttempted, connectors])


  if (!isConnected) {
    return (
      <Card className={className}>
        <CardContent className="p-6 text-center space-y-4">
          <Loader2 className="w-12 h-12 mx-auto text-primary animate-spin" />
          <div>
            <h3 className="font-semibold">Connecting Wallet...</h3>
            <p className="text-sm text-muted-foreground">Please connect MetaMask</p>
          </div>
        </CardContent>
      </Card>
    )
  }

  const isBusy = isApproving || isPaying || isApprovalConfirming || isPaymentConfirming

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
                disabled={isBusy}
                className={`p-4 border rounded-lg text-left transition-all ${selectedToken === token
                  ? 'border-primary bg-primary/5'
                  : 'border-muted hover:border-primary/50'
                  } ${isBusy ? 'opacity-50 cursor-not-allowed' : ''}`}
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
            <Badge variant="outline">{chain?.name || 'Unknown'}</Badge>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm flex items-center gap-1">
              <Fuel className="w-3 h-3" />
              Est. Gas Fee
            </span>
            <span className="font-medium">~{estimatedGas} {getGasTokenSymbol()}</span>
          </div>
        </div>

        {/* Status Alert */}
        <PaymentProgress
          step={isPaymentConfirmed ? 'complete' : isPaying || isPaymentConfirming ? 'paying' : isApproving || isApprovalConfirming ? 'approving' : 'idle'}
          approvalHash={approvalHash}
          paymentHash={paymentHash}
          isApprovalConfirming={isApprovalConfirming}
          isPaymentConfirming={isPaymentConfirming}
          explorerUrl={chain?.blockExplorers?.default?.url}
        />

        {/* Info Alert */}
        {!isPaymentConfirmed && (
          <Alert className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
            <AlertCircle className="w-4 h-4 text-blue-600 dark:text-blue-400" />
            <AlertDescription className="text-sm text-blue-700 dark:text-blue-300">
              <strong>Two-Step Process:</strong> 1. Approve {selectedToken} usage 2. Confirm payment
            </AlertDescription>
          </Alert>
        )}

        {/* Action Button */}
        <Button
          onClick={approve}
          disabled={isBusy || isApprovalConfirmed}
          className="w-full"
          size="lg"
        >
          {isBusy ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Processing...
            </>
          ) : isPaymentConfirmed ? (
            <>
              <CheckCircle className="w-4 h-4 mr-2" />
              Payment Successful!
            </>
          ) : (
            <>
              <Send className="w-4 h-4 mr-2" />
              Pay ${amount}
            </>
          )}
        </Button>

        {/* Security Notice */}
        <Alert>
          <DollarSign className="w-4 h-4" />
          <AlertDescription className="text-sm">
            Make sure you have enough {selectedToken} and {getGasTokenSymbol()} for gas fees on {chain?.name}.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}