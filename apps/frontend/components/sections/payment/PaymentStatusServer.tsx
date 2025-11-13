import { 
  getPaymentStatus as _getPaymentStatusAction,
  getPaymentHistory
} from '@/lib/server-actions';
import { PaymentStatusSection } from './PaymentStatusSection';

// Transaction interface that matches what TransactionHistory expects
interface Transaction {
  orderNo: string;
  actualAmount: number;
  currency: string;
  status: string;
  finishTime: string;
  blockchainData: {
    txHash: string;
    network: string;
  };
  blockExplorerUrl: string;
}

interface PaymentStatusServerProps {
  className?: string;
  showTitle?: boolean;
}

export async function PaymentStatusServer({
  className = '',
  showTitle = true
}: PaymentStatusServerProps) {
  let transactions: Transaction[] = [];
  let error: string | null = null;

  try {
    // Fetch payment history server-side
    const userTransactions = await getPaymentHistory();

    // Map payment history to Transaction format expected by TransactionHistory
    transactions = userTransactions.map((tx: any) => ({
      orderNo: tx.id || tx.orderNo || '',
      actualAmount: tx.amount || 0,
      currency: tx.currency || 'USD',
      status: tx.status || 'pending',
      finishTime: tx.finishTime || tx.createdAt || new Date().toISOString(),
      blockchainData: tx.blockchainData || {
        txHash: '',
        network: 'BSC'
      },
      blockExplorerUrl: tx.blockExplorerUrl || ''
    }));
  } catch (err) {
    // Check if this is a redirect error (user not authenticated)
    if (
      err &&
      typeof err === 'object' &&
      'digest' in err &&
      typeof err.digest === 'string' &&
      err.digest.startsWith('NEXT_REDIRECT')
    ) {
      // User not authenticated - show empty state instead of redirecting
      // This allows viewing pricing plans without login
      transactions = [];
      error = null; // No error - just empty state for unauthenticated users
    } else {
      console.error('Failed to fetch transactions server-side:', err);
      error = 'Failed to load transaction history';
    }
  }

  return (
    <PaymentStatusSection
      className={className}
      showTitle={showTitle}
      transactions={transactions}
      error={error}
    />
  );
}