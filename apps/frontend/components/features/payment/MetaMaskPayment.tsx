'use client'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { PAYMENT_ESCROW_ABI } from '@/lib/contracts/PaymentEscrowABI'
import { getPaymentEscrowAddress, getTokenAddress, isPaymentEscrowDeployed } from '@/lib/contracts/addresses'
import { devLog } from '@/shared/utils'
import {
  AlertCircle,
  CheckCircle,
  DollarSign,
  ExternalLink,
  Fuel,
  Loader2,
  Send,
  Wallet
} from 'lucide-react'
import { useEffect, useState } from 'react'
import { formatUnits, getAddress, parseUnits } from 'viem'
import { useAccount, useConnect, useWaitForTransactionReceipt, useWriteContract } from 'wagmi'

interface MetaMaskPaymentProps {
  planId: number | string
  planName: string
  amount: number
  currency: string
  onSuccess: (txHash: string) => void
  onError: (error: string) => void
  className?: string
}

// Complete ERC20 ABI for token operations
const ERC20_ABI = [
  {
    type: 'function',
    name: 'approve',
    inputs: [
      { name: 'spender', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [{ type: 'bool' }],
    stateMutability: 'nonpayable'
  },
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
    name: 'transferFrom',
    inputs: [
      { name: 'from', type: 'address' },
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [{ type: 'bool' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'allowance',
    inputs: [
      { name: 'owner', type: 'address' },
      { name: 'spender', type: 'address' }
    ],
    outputs: [{ type: 'uint256' }],
    stateMutability: 'view'
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
  },
  {
    type: 'function',
    name: 'name',
    inputs: [],
    outputs: [{ type: 'string' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'symbol',
    inputs: [],
    outputs: [{ type: 'string' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'totalSupply',
    inputs: [],
    outputs: [{ type: 'uint256' }],
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
  // Debug logging only in non-production environments
  devLog('MetaMaskPayment loaded', {
    planId,
    planName,
    amount,
    currency,
  })

  // Early validation
  if (!amount || amount === 0 || isNaN(amount)) {
    devLog('MetaMaskPayment Invalid amount:', { amount, type: typeof amount, planId, planName })
  }

  const { address, isConnected, chain } = useAccount()
  const { connect, connectors } = useConnect()
  const [selectedToken, setSelectedToken] = useState<'USDT' | 'USDC'>('USDT')
  const [estimatedGas, setEstimatedGas] = useState<string>('0.005')
  const [isEstimatingGas, setIsEstimatingGas] = useState(false)
  const [autoConnectAttempted, setAutoConnectAttempted] = useState(false)
  const [paymentStep, setPaymentStep] = useState<'idle' | 'approving' | 'paying' | 'complete'>('idle')

  // Approval transaction
  const {
    writeContract: writeApproval,
    data: approvalHash,
    error: approvalError,
    isPending: isApproving
  } = useWriteContract()

  const {
    isLoading: isApprovalConfirming,
    isSuccess: isApprovalConfirmed
  } = useWaitForTransactionReceipt({
    hash: approvalHash,
  })

  // Payment transaction
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

  // Get token contract address based on selected token and current chain
  const getTokenContractAddress = () => {
    if (!chain?.id) return null

    try {
      const address = getTokenAddress(selectedToken, chain.id)
      // Use viem's getAddress to ensure proper checksum formatting
      return getAddress(address)
    } catch (error) {
      console.error('Error getting token address:', error)
      return null
    }
  }

  // Get payment escrow contract address
  const getEscrowContractAddress = () => {
    if (!chain?.id) return null

    try {
      if (!isPaymentEscrowDeployed(chain.id)) {
        console.error('Payment escrow not deployed on chain:', chain.id)
        return null
      }
      const address = getPaymentEscrowAddress(chain.id)
      // Use viem's getAddress to ensure proper checksum formatting
      return getAddress(address)
    } catch (error) {
      console.error('Error getting escrow address:', error)
      return null
    }
  }

  // Handle approval confirmation -> trigger payment
  useEffect(() => {
    if (isApprovalConfirmed && approvalHash) {
      devLog('✅ Approval confirmed, proceeding to payment...')
      setPaymentStep('paying')
      executePayment()
    }
  }, [isApprovalConfirmed, approvalHash])

  // Handle payment confirmation -> success
  useEffect(() => {
    if (isPaymentConfirmed && paymentHash) {
      devLog('✅ Payment confirmed!')
      setPaymentStep('complete')
      onSuccess(paymentHash)
    }
  }, [isPaymentConfirmed, paymentHash, onSuccess])

  // Handle errors with user rejection detection
  useEffect(() => {
    if (approvalError) {
      if (!handleTransactionError(approvalError, 'approval')) {
        // User cancelled - don't show error
        return
      }
      onError(`Approval failed: ${approvalError.message}`)
    }
    if (paymentError) {
      if (!handleTransactionError(paymentError, 'payment')) {
        // User cancelled - don't show error
        return
      }
      onError(`Payment failed: ${paymentError.message}`)
    }
    if (receiptError) {
      if (!handleTransactionError(receiptError, 'receipt')) {
        // User cancelled - don't show error
        return
      }
      onError(`Transaction error: ${receiptError.message}`)
    }
  }, [approvalError, paymentError, receiptError, onError])

  // Detect if user rejected the transaction
  const isUserRejection = (error: any): boolean => {
    if (!error?.message) return false

    const userRejectionPatterns = [
      'user rejected the request',
      'user denied transaction signature',
      'User rejected the request',
      'User denied transaction signature',
      'user denied',
      'rejected by user',
      'denied by user'
    ]

    return userRejectionPatterns.some(pattern =>
      error.message.toLowerCase().includes(pattern.toLowerCase())
    )
  }

  // Handle transaction errors appropriately
  const handleTransactionError = (error: any, transactionType: 'approval' | 'payment' | 'receipt') => {
    if (isUserRejection(error)) {
      // User cancelled - don't show as error, just reset state
      devLog(`👤 User cancelled ${transactionType} transaction`)
      setPaymentStep('idle')
      return false // Don't trigger error callback
    }

    // Real error - show error message
    console.error(`❌ ${transactionType} error:`, error)
    setPaymentStep('idle')
    return true // Trigger error callback
  }

  // Step 1: Approve escrow contract to spend tokens
  const handlePayment = async () => {
    devLog('🚀 Starting two-step payment process')

    if (!isConnected || !address) {
      console.error('❌ Wallet not connected')
      onError('Please connect your wallet first')
      return
    }

    const tokenAddress = getTokenContractAddress()
    const escrowAddress = getEscrowContractAddress()

    if (!tokenAddress || !escrowAddress) {
      if (chain?.id === 97) {
        onError('Payment escrow contract development mode. Contract functions will need real deployment for production payments.')
      } else {
        onError('Payment contract not available on this network')
      }
      return
    }

    try {
      const decimals = 6 // USDT and USDC both use 6 decimals

      devLog('🚨 CRITICAL DEBUG - Before parseUnits:', {
        amount,
        amountType: typeof amount,
        amountIsZero: amount === 0,
        amountIsUndefined: amount === undefined,
        amountIsNull: amount === null,
        amountIsNaN: isNaN(amount),
        planId,
        planName
      });

      const transferAmount = parseUnits(amount.toString(), decimals)

      devLog('📝 Payment details:', {
        planId,
        tokenAddress,
        escrowAddress,
        amount,
        decimals,
        transferAmount: transferAmount.toString(),
        transferAmountFormatted: transferAmount.toString(),
        humanReadableAmount: formatUnits(transferAmount, decimals),
        chainId: chain?.id
      })

      devLog('🚨 CRITICAL DEBUG - MetaMask will show:', {
        rawAmount: amount.toString(),
        parsedUnits: transferAmount.toString(),
        inTokens: formatUnits(transferAmount, decimals),
        willShowZero: transferAmount === 0n
      });

      setPaymentStep('approving')

      // Step 1: Approve escrow contract to spend tokens
      devLog('⏳ Step 1: Approving token spending...')

      devLog('🔥 APPROVE CALL ARGUMENTS:', {
        tokenAddress,
        escrowAddress,
        transferAmount: transferAmount.toString(),
        transferAmountHex: '0x' + transferAmount.toString(16),
        transferAmountBigInt: transferAmount.toString(),
        approveArgs: [escrowAddress as `0x${string}`, transferAmount],
        fullCall: {
          address: tokenAddress as `0x${string}`,
          functionName: 'approve',
          args: [escrowAddress as `0x${string}`, transferAmount]
        }
      });

      writeApproval({
        address: tokenAddress as `0x${string}`,
        abi: ERC20_ABI,
        functionName: 'approve',
        args: [escrowAddress as `0x${string}`, transferAmount],
      })

      devLog('✅ Approval transaction submitted')
    } catch (error) {
      if (!handleTransactionError(error, 'approval')) {
        // User cancelled - don't show error
        return
      }
      onError(`Payment preparation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  // Step 2: Execute payment through escrow contract
  const executePayment = async () => {
    const tokenAddress = getTokenContractAddress()
    const escrowAddress = getEscrowContractAddress()

    if (!tokenAddress || !escrowAddress) {
      onError('Payment contract not available')
      return
    }

    try {
      const decimals = 6

      devLog('🚨 CRITICAL DEBUG - executePayment parseUnits:', {
        amount,
        amountType: typeof amount,
        amountIsZero: amount === 0,
        planId,
        planName
      });

      const transferAmount = parseUnits(amount.toString(), decimals)

      // Convert UUID plan ID to numeric ID for smart contract
      // Map backend UUIDs to numeric IDs that the contract expects
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
            // Check if it's a UUID that we have a mapping for
            if (uuidOrId in planIdMap) {
              return BigInt(planIdMap[uuidOrId]);
            }
            // Check if it's already a numeric string
            if (/^\d+$/.test(uuidOrId)) {
              return BigInt(uuidOrId);
            }
            // Default to Pro Plan (ID 3) for unknown UUIDs
            console.warn('Unknown plan UUID, defaulting to Pro Plan (ID 3):', uuidOrId);
            return 3n;
          }
        } catch (e) {
          console.warn('Failed to convert planId to BigInt, using Pro Plan (ID 3)', e);
        }
        return 3n; // Default to Pro Plan
      };
      const numericPlanId = getNumericPlanId(planId);

      devLog('📋 Plan ID Conversion:');
      devLog(`  - Original planId: ${planId} (${typeof planId})`);
      devLog(`  - Numeric planId: ${numericPlanId}`);

      // Step 2: Execute payment through escrow contract
      devLog('💸 Step 2: Executing payment transfer...')

      devLog('🔥 PAYMENT CALL ARGUMENTS:', {
        functionName: 'payWithTransfer',
        planId: numericPlanId,
        tokenAddress,
        amount: transferAmount.toString(),
        amountFormatted: amount
      });

      // Use payWithAmountDisplay to show the amount in MetaMask
      // Send a small amount of ETH for display purposes (will be refunded)
      const displayAmount = parseUnits(amount.toString(), 18); // Convert to wei for display

      writePayment({
        address: escrowAddress as `0x${string}`,
        abi: PAYMENT_ESCROW_ABI,
        functionName: 'payWithAmountDisplay',
        args: [numericPlanId, tokenAddress as `0x${string}`, transferAmount],
        value: displayAmount // This will show the amount in MetaMask but get refunded
      })

      devLog('✅ Payment transaction submitted')
    } catch (error) {
      if (!handleTransactionError(error, 'payment')) {
        // User cancelled - don't show error
        return
      }
      onError(`Payment execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }



  // Get gas token symbol based on chain
  const getGasTokenSymbol = () => {
    switch (chain?.id) {
      case 56:
      case 97:
        return 'BNB'
      case 31337:
        return 'ETH'
      default:
        return 'ETH'
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
        case 56: setEstimatedGas('0.0003') // BSC Mainnet
          break
        case 97: setEstimatedGas('0.0003') // BSC Testnet
          break
        case 31337: setEstimatedGas('0.0001') // Hardhat Localhost
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

  // Auto-connect to MetaMask when component mounts
  useEffect(() => {
    if (!isConnected && !autoConnectAttempted && connectors.length > 0) {
      setAutoConnectAttempted(true)

      const attemptAutoConnect = async () => {
        try {
          // Look for MetaMask connector first
          const metaMaskConnector = connectors.find(
            connector => connector.name === 'MetaMask' ||
              connector.name === 'Injected' ||
              connector.id === 'metaMask'
          )

          if (metaMaskConnector) {
            devLog('Auto-connecting to MetaMask...')
            await connect({ connector: metaMaskConnector })
            devLog('MetaMask auto-connect successful')
          } else {
            devLog('MetaMask connector not found, trying first available connector')
            // Fallback to first connector if MetaMask not found
            if (connectors[0]) {
              await connect({ connector: connectors[0] })
              devLog(`Auto-connect to ${connectors[0].name} successful`)
            }
          }
        } catch (error) {
          devLog('Auto-connect failed (user likely hasn\'t connected before)', { error })
          // This is expected behavior if user hasn't connected before
        }
      }

      // Small delay to ensure connectors are ready
      const timeoutId = setTimeout(attemptAutoConnect, 500)
      return () => clearTimeout(timeoutId)
    }
  }, [isConnected, autoConnectAttempted, connectors, connect])

  // Early validation for invalid amount
  if (!amount || amount === 0 || isNaN(amount)) {
    return (
      <Card className={className}>
        <CardContent className="p-6 text-center space-y-4">
          <AlertCircle className="w-12 h-12 mx-auto text-red-500" />
          <div>
            <h3 className="font-semibold text-red-600">Invalid Payment Amount</h3>
            <p className="text-sm text-muted-foreground">
              The selected plan has an invalid price. Please contact support or try a different plan.
            </p>
            <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg text-left">
              <p className="text-xs font-mono">
                Debug: amount={JSON.stringify(amount)} (type: {typeof amount})
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (!isConnected) {
    return (
      <Card className={className}>
        <CardContent className="p-6 text-center space-y-4">
          {!autoConnectAttempted ? (
            <>
              <Loader2 className="w-12 h-12 mx-auto text-primary animate-spin" />
              <div>
                <h3 className="font-semibold">Auto-connecting...</h3>
                <p className="text-sm text-muted-foreground">
                  Attempting to connect to your wallet automatically
                </p>
              </div>
            </>
          ) : (
            <>
              <Wallet className="w-12 h-12 mx-auto text-muted-foreground" />
              <div>
                <h3 className="font-semibold">Connect Your Wallet</h3>
                <p className="text-sm text-muted-foreground">
                  Connect MetaMask to pay with cryptocurrency
                </p>
              </div>
            </>
          )}
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
                className={`p-4 border rounded-lg text-left transition-all ${selectedToken === token
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
            <span className="font-medium">
              ${amount} {selectedToken}
            </span>
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
                `~${estimatedGas} ${getGasTokenSymbol()}`
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
        {(approvalHash || paymentHash) && (
          <Alert>
            <AlertCircle className="w-4 h-4" />
            <AlertDescription className="flex items-center justify-between">
              <span>
                {paymentStep === 'approving'
                  ? `Approval ${isApprovalConfirming ? 'pending' : isApprovalConfirmed ? 'confirmed' : 'submitted'}`
                  : `Payment ${isPaymentConfirming ? 'pending' : isPaymentConfirmed ? 'confirmed' : 'submitted'}`
                }
              </span>
              <a
                href={`${chain?.blockExplorers?.default?.url}/tx/${paymentStep === 'approving' ? approvalHash : paymentHash}`}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-primary hover:underline"
              >
                View <ExternalLink className="w-3 h-3" />
              </a>
            </AlertDescription>
          </Alert>
        )}

        {/* Payment Process Explanation */}
        <div className="space-y-3">
          <Alert className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
            <AlertCircle className="w-4 h-4 text-blue-600 dark:text-blue-400" />
            <AlertDescription className="text-sm text-blue-700 dark:text-blue-300">
              <strong>Two-Step Payment Process:</strong>
              <ol className="list-decimal ml-4 mt-1 space-y-1">
                <li><strong>Approve:</strong> Allow the contract to spend your {selectedToken}.</li>
                <li><strong>Pay:</strong> Confirm the payment transaction.</li>
              </ol>
            </AlertDescription>
          </Alert>

          {paymentStep === 'paying' && (
            <Alert className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
              <AlertCircle className="w-4 h-4 text-blue-600 dark:text-blue-400" />
              <AlertDescription className="text-sm text-blue-700 dark:text-blue-300">
                <strong>MetaMask Display:</strong><br />
                MetaMask will now show the correct <strong>{amount} {getGasTokenSymbol()}</strong> amount for confirmation!
                <br />
                The native currency will be automatically refunded - only the <strong>{amount} {selectedToken}</strong> tokens will be charged.
              </AlertDescription>
            </Alert>
          )}
        </div>

        {/* Payment Button */}
        <Button
          onClick={handlePayment}
          disabled={isApproving || isPaying || isApprovalConfirming || isPaymentConfirming || isApprovalConfirmed || isPaymentConfirmed}
          className="w-full"
          size="lg"
        >
          {isApproving ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Step 1: Approving {selectedToken}...
            </>
          ) : isPaying ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Step 2: Processing Payment...
            </>
          ) : isApprovalConfirming ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Confirming Approval...
            </>
          ) : isPaymentConfirming ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Confirming Payment...
            </>
          ) : isApprovalConfirmed ? (
            <>
              <CheckCircle className="w-4 h-4 mr-2" />
              Approval Confirmed - Click to Pay
            </>
          ) : isPaymentConfirmed ? (
            <>
              <CheckCircle className="w-4 h-4 mr-2" />
              Payment Successful!
            </>
          ) : (
            <>
              <Send className="w-4 h-4 mr-2" />
              {paymentStep === 'approving' ? `Approve & Pay $${amount}` : `Complete Payment ($${amount})`}
            </>
          )}
        </Button>

        {/* Security Notice */}
        <Alert>
          <DollarSign className="w-4 h-4" />
          <AlertDescription className="text-sm">
            Your transaction will be processed on the {chain?.name} network.
            Make sure you have enough {selectedToken} tokens and {getGasTokenSymbol()} for gas fees.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  )
}