'use client';

import { useState } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { BLOCKCHAIN_CONFIG } from '@/app/constants/packages';

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

export function TransactionHistory({ transactions, className = '' }: TransactionHistoryProps) {
  const [page, setPage] = useState(1);
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const itemsPerPage = 5;

  // Filter transactions based on search and status
  const filteredTransactions = transactions.filter(transaction => {
    const matchesSearch = transaction.orderNo.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         transaction.currency.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         transaction.blockchainData.txHash.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesStatus = statusFilter === 'all' || transaction.status.toLowerCase() === statusFilter.toLowerCase();
    
    return matchesSearch && matchesStatus;
  });

  const totalPages = Math.ceil(filteredTransactions.length / itemsPerPage);
  const currentTransactions = filteredTransactions.slice(
    (page - 1) * itemsPerPage,
    page * itemsPerPage
  );

  const exportToCSV = () => {
    const csvContent = [
      ['Order No', 'Amount', 'Currency', 'Status', 'Date', 'Network', 'TX Hash'].join(','),
      ...filteredTransactions.map(tx => [
        tx.orderNo,
        tx.actualAmount,
        tx.currency,
        tx.status,
        new Date(tx.finishTime).toLocaleDateString(),
        tx.blockchainData.network,
        tx.blockchainData.txHash
      ].join(','))
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
        return <Badge className="bg-green-100 text-green-800">✅ {status}</Badge>;
      case 'pending':
        return <Badge className="bg-yellow-100 text-yellow-800">⏳ {status}</Badge>;
      case 'failed':
        return <Badge className="bg-red-100 text-red-800">❌ {status}</Badge>;
      default:
        return <Badge variant="outline">{status}</Badge>;
    }
  };

  return (
    <Card className={`${className} transition-shadow hover:shadow-lg`}>
      <CardHeader>
        <CardTitle className="text-xl font-semibold flex items-center gap-2">
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
        <div className="mb-4 flex flex-col gap-2">
          <Input
            placeholder="Search by order no, currency, or TX hash"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          <div className="flex gap-2">
            <Button
              variant={statusFilter === 'all' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('all')}
              className="flex-1"
            >
              All
            </Button>
            <Button
              variant={statusFilter === 'completed' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('completed')}
              className="flex-1"
            >
              Completed
            </Button>
            <Button
              variant={statusFilter === 'pending' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('pending')}
              className="flex-1"
            >
              Pending
            </Button>
            <Button
              variant={statusFilter === 'failed' ? 'default' : 'outline'}
              onClick={() => setStatusFilter('failed')}
              className="flex-1"
            >
              Failed
            </Button>
          </div>
          <Button
            variant="outline"
            onClick={exportToCSV}
            className="self-end"
          >
            📥 Export to CSV
          </Button>
        </div>

        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Date</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Transaction</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {currentTransactions.length > 0 ? (
                currentTransactions.map((tx) => (
                  <TableRow key={tx.orderNo}>
                    <TableCell>{new Date(tx.finishTime).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })}</TableCell>
                    <TableCell>
                      {formatAmount(tx.actualAmount)} {tx.currency}
                    </TableCell>
                    <TableCell>
                      {getStatusBadge(tx.status)}
                    </TableCell>
                    <TableCell>
                      {tx.blockchainData.txHash && (
                        <Button
                          variant="link"
                          className="p-0 h-auto font-normal underline-offset-4"
                          onClick={() => window.open(tx.blockExplorerUrl, '_blank')}
                        >
                          View on {tx.blockchainData.network || 'BSC'} Explorer
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell colSpan={4} className="text-center py-4">
                    No transactions found
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>

        {totalPages > 1 && (
          <div className="flex items-center justify-end space-x-2 py-4">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page === 1}
            >
              Previous
            </Button>
            <span className="text-sm text-muted-foreground">
              Page {page} of {totalPages}
            </span>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              disabled={page === totalPages}
            >
              Next
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
