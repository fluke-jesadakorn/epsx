import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, ExternalLink, Loader2 } from 'lucide-react'

interface PaymentProgressProps {
    step: 'idle' | 'approving' | 'paying' | 'complete'
    approvalHash?: string
    paymentHash?: string
    isApprovalConfirming: boolean
    isPaymentConfirming: boolean
    explorerUrl?: string
}

export function PaymentProgress({
    step,
    approvalHash,
    paymentHash,
    isApprovalConfirming,
    isPaymentConfirming,
    explorerUrl
}: PaymentProgressProps) {
    if (!approvalHash && !paymentHash) {return null}

    const isApproving = step === 'approving'
    const hash = isApproving ? approvalHash : paymentHash
    const isConfirming = isApproving ? isApprovalConfirming : isPaymentConfirming
    const actionName = isApproving ? 'Approval' : 'Payment'

    return (
        <Alert>
            {isConfirming ? (
                <Loader2 className="w-4 h-4 animate-spin text-primary" />
            ) : (
                <AlertCircle className="w-4 h-4" />
            )}
            <AlertDescription className="flex items-center justify-between w-full">
                <span>
                    {actionName} {isConfirming ? 'pending confirmation...' : 'submitted'}
                </span>
                {hash && explorerUrl && (
                    <a
                        href={`${explorerUrl}/tx/${hash}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-1 text-primary hover:underline"
                    >
                        View <ExternalLink className="w-3 h-3" />
                    </a>
                )}
            </AlertDescription>
        </Alert>
    )
}
