'use client';

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { useEffect, useState } from 'react';
import { Badge, Button, Input } from '@/components/ui';

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

interface TransactionHistoryProps {
  transactions: Transaction[];
  className?: string;
}

export function TransactionHistory({
  transactions,
  className = '',
}: TransactionHistoryProps) {
  const [page, setPage] = useState(1);
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [isMobile, setIsMobile] = useState(() => {
    // Check if we're in browser environment and get initial mobile state
    if (typeof window !== 'undefined') {
      return window.innerWidth < 768;
    }
    return false; // Default for SSR
  });
  const itemsPerPage = 5;

  // Check screen size for mobile responsiveness
  useEffect(() => {
    const checkScreenSize = () => {
      const isMobileCheck = window.innerWidth < 768;
      // Screen size check complete
      setIsMobile(isMobileCheck);
    };

    checkScreenSize();
    window.addEventListener('resize', checkScreenSize);
    return () => window.removeEventListener('resize', checkScreenSize);
  }, []);

  // Filter transactions based on search and status
  const filteredTransactions = transactions.filter((transaction) => {
    const matchesSearch =
      transaction.orderNo.toLowerCase().includes(searchTerm.toLowerCase()) ||
      transaction.currency.toLowerCase().includes(searchTerm.toLowerCase()) ||
      transaction.blockchainData.txHash
        .toLowerCase()
        .includes(searchTerm.toLowerCase());

    const matchesStatus =
      statusFilter === 'all' ||
      transaction.status.toLowerCase() === statusFilter.toLowerCase();

    return matchesSearch && matchesStatus;
  });

  const totalPages = Math.ceil(filteredTransactions.length / itemsPerPage);
  const currentTransactions = filteredTransactions.slice(
    (page - 1) * itemsPerPage,
    page * itemsPerPage,
  );

  const exportToCSV = () => {
    const csvContent = [
      [
        'Order No',
        'Amount',
        'Currency',
        'Status',
        'Date',
        'Network',
        'TX Hash',
      ].join(','),
      ...filteredTransactions.map((tx) =>
        [
          tx.orderNo,
          tx.actualAmount,
          tx.currency,
          tx.status,
          new Date(tx.finishTime).toLocaleDateString(),
          tx.blockchainData.network,
          tx.blockchainData.txHash,
        ].join(','),
      ),
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `transactions_${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const formatAmount = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 6,
    }).format(amount);
  };

  const getStatusBadge = (status: string) => {
    const statusLower = status.toLowerCase();
    switch (statusLower) {
      case 'succeeded':
      case 'completed':
        return (
          <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            ✅ {status}
          </Badge>
        );
      case 'pending':
        return (
          <Badge className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
            ⏳ {status}
          </Badge>
        );
      case 'failed':
        return (
          <Badge className="bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
            ❌ {status}
          </Badge>
        );
      default:
        return (
          <Badge
            variant="outline"
            className="border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300"
          >
            {status}
          </Badge>
        );
    }
  };

  // Mobile Transaction Card Component
  const TransactionCard = ({ transaction }: { transaction: Transaction }) => (
    <Card className="mb-4 shadow-sm hover:shadow-md transition-shadow">
      <CardContent className="p-4">
        <div className="flex justify-between items-start mb-3">
          <div>
            <p className="text-sm font-medium text-gray-900 dark:text-white">
              {formatAmount(transaction.actualAmount)} {transaction.currency}
            </p>
            <p className="text-xs text-muted-foreground">
              Order: {transaction.orderNo}
            </p>
          </div>
          {getStatusBadge(transaction.status)}
        </div>

        <div className="space-y-2">
          <div className="flex justify-between items-center text-sm">
            <span className="text-muted-foreground">Date:</span>
            <div className="text-right">
              <div className="text-gray-900 dark:text-white">
                {new Date(transaction.finishTime).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'short',
                  day: 'numeric',
                })}
              </div>
              <div className="text-xs text-muted-foreground">
                {new Date(transaction.finishTime).toLocaleTimeString('en-US', {
                  hour: '2-digit',
                  minute: '2-digit',
                })}
              </div>
            </div>
          </div>

          {transaction.blockchainData.txHash && (
            <div className="pt-2 border-t border-gray-100 dark:border-gray-700">
              <Button
                variant="link"
                className="p-0 h-auto font-normal underline-offset-4 text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 text-sm w-full"
                onClick={() =>
                  window.open(transaction.blockExplorerUrl, '_blank')
                }
              >
                View on {transaction.blockchainData.network || 'BSC'} Explorer →
              </Button>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );

  return (
    <Card
      className={`${className} transition-shadow hover:shadow-lg border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800`}
    >
      <CardHeader>
        <CardTitle className="text-xl font-semibold flex items-center gap-2 text-gray-900 dark:text-white">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M4 4a2 2 0 012-2h8a2 2 0 012 2v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 0v12h8V4H6z"
              clipRule="evenodd"
            />
          </svg>
          Transaction History
        </CardTitle>
        <CardDescription>
          View your payment history and transaction details
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="mb-4 flex flex-col gap-4">
          <Input
            placeholder="Search by order no, currency, or TX hash"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600"
          />
          <div className="flex flex-wrap gap-2">
            <Button
              variant={statusFilter === 'all' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('all')}
              className="flex-1 sm:flex-none min-w-0 text-sm"
            >
              All
            </Button>
            <Button
              variant={statusFilter === 'completed' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('completed')}
              className="flex-1 sm:flex-none min-w-0 text-sm"
            >
              Completed
            </Button>
            <Button
              variant={statusFilter === 'pending' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('pending')}
              className="flex-1 sm:flex-none min-w-0 text-sm"
            >
              Pending
            </Button>
            <Button
              variant={statusFilter === 'failed' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('failed')}
              className="flex-1 sm:flex-none min-w-0 text-sm"
            >
              Failed
            </Button>
          </div>
          <Button
            variant="outline"
            onClick={exportToCSV}
            className="self-end border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            📥 Export to CSV
          </Button>
        </div>

        {/* Responsive transaction display */}
        {/* Debug: Current screen state - Window width: {typeof window !== 'undefined' ? window.innerWidth : 'SSR'}, isMobile: {isMobile.toString()} */}
        {(() => {
          // Rendering decision made
          return null;
        })()}
        {isMobile ? (
          // Mobile Card View
          <div className="mobile-view">
            <div
              style={{
                backgroundColor: 'rgba(0,255,0,0.1)',
                padding: '4px',
                fontSize: '12px',
              }}
            >
              DEBUG: Mobile Cards View (isMobile={isMobile.toString()})
            </div>
            {currentTransactions.length > 0 ? (
              currentTransactions.map((tx) => (
                <TransactionCard key={tx.orderNo} transaction={tx} />
              ))
            ) : (
              <Card className="text-center py-8">
                <CardContent>
                  <p className="text-muted-foreground">No transactions found</p>
                </CardContent>
              </Card>
            )}
          </div>
        ) : (
          // Desktop Table View
          <div className="desktop-view rounded-md border border-gray-200 dark:border-gray-700 overflow-hidden">
            <div
              style={{
                backgroundColor: 'rgba(255,0,0,0.1)',
                padding: '4px',
                fontSize: '12px',
              }}
            >
              DEBUG: Desktop Table View (isMobile={isMobile.toString()})
            </div>
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow className="bg-gray-50 dark:bg-gray-700">
                    <TableHead className="text-gray-900 dark:text-white">
                      Date
                    </TableHead>
                    <TableHead className="text-gray-900 dark:text-white">
                      Amount
                    </TableHead>
                    <TableHead className="text-gray-900 dark:text-white">
                      Status
                    </TableHead>
                    <TableHead className="text-gray-900 dark:text-white">
                      Transaction
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {currentTransactions.length > 0 ? (
                    currentTransactions.map((tx) => (
                      <TableRow
                        key={tx.orderNo}
                        className="hover:bg-gray-50 dark:hover:bg-gray-700"
                      >
                        <TableCell className="text-gray-900 dark:text-white">
                          <div className="text-sm">
                            {new Date(tx.finishTime).toLocaleDateString(
                              'en-US',
                              {
                                year: 'numeric',
                                month: 'short',
                                day: 'numeric',
                              },
                            )}
                          </div>
                          <div className="text-xs text-muted-foreground">
                            {new Date(tx.finishTime).toLocaleTimeString(
                              'en-US',
                              {
                                hour: '2-digit',
                                minute: '2-digit',
                              },
                            )}
                          </div>
                        </TableCell>
                        <TableCell className="text-gray-900 dark:text-white">
                          <div className="font-medium">
                            {formatAmount(tx.actualAmount)} {tx.currency}
                          </div>
                        </TableCell>
                        <TableCell>{getStatusBadge(tx.status)}</TableCell>
                        <TableCell>
                          {tx.blockchainData.txHash && (
                            <Button
                              variant="link"
                              className="p-0 h-auto font-normal underline-offset-4 text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300"
                              onClick={() =>
                                window.open(tx.blockExplorerUrl, '_blank')
                              }
                            >
                              <span className="hidden sm:inline">
                                View on {tx.blockchainData.network || 'BSC'}{' '}
                                Explorer
                              </span>
                              <span className="sm:hidden">View TX</span>
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell
                        colSpan={4}
                        className="text-center py-8 text-muted-foreground"
                      >
                        No transactions found
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          </div>
        )}

        {totalPages > 1 && (
          <div className="flex flex-col sm:flex-row items-center justify-between space-y-2 sm:space-y-0 sm:space-x-2 py-4">
            <span className="text-sm text-muted-foreground">
              Page {page} of {totalPages}
            </span>
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
                className="border-gray-200 dark:border-gray-600"
              >
                Previous
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
                className="border-gray-200 dark:border-gray-600"
              >
                Next
              </Button>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
