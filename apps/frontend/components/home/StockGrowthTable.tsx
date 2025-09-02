'use client';

import React from 'react';

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

import { Button } from '../ui/button';

import type { TableDataMetrics } from '@/types/stockFetchData';

interface StockGrowthTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableDataMetrics[];
}

const StockGrowthTable: React.FC<StockGrowthTableProps> = ({
  style,
  className,
  data,
}) => {
  return (
    <div className={`w-full ${className || ''}`} style={style}>
      <div className="scroll-shadow-container custom-scrollbar bg-card rounded-xl border">
        <Table>
          <TableHeader className="bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-pink-500/5 backdrop-blur-sm">
            <TableRow className="hover:bg-transparent">
              <TableHead className="text-primary/70 font-semibold">
                No.
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Symbol
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Name
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Growth %
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Current Index
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Next Index
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Value
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                <span title="Value Change Percentage">Change %</span>
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                <span title="Data Volume">Volume</span>
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Sector
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Country
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Last Action
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                Next Action
              </TableHead>
              <TableHead className="text-primary/70 font-semibold">
                <span title="Open Analytics View">Chart</span>
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.map((row, index) => (
              <TableRow
                key={row.symbol}
                className="hover:bg-primary/5 transition-colors"
              >
                <TableCell>{index + 1}</TableCell>
                <TableCell className="font-medium">{row.symbol}</TableCell>
                <TableCell className="font-medium">{row.name}</TableCell>
                <TableCell>
                  <span className="font-medium text-green-500">
                    {row.epsGrowth}
                  </span>
                </TableCell>
                <TableCell className="font-medium">
                  {row.currentQuarterEps}
                </TableCell>
                <TableCell className="font-medium">{row.nextEps}</TableCell>
                <TableCell className="font-medium">
                  {row.dataValue} {row.currency}
                </TableCell>
                <TableCell>
                  <span
                    className={`font-medium ${
                      parseFloat(row.changePercent || '0') >= 0
                        ? 'text-green-500'
                        : 'text-rose-500'
                    }`}
                  >
                    {parseFloat(row.changePercent || '0') >= 0 ? '+' : ''}
                    {row.changePercent}%
                  </span>
                </TableCell>
                <TableCell className="font-medium">{row.volume}</TableCell>
                <TableCell className="text-muted-foreground">
                  {row.sector}
                </TableCell>
                <TableCell className="text-muted-foreground">
                  {row.country}
                </TableCell>
                <TableCell className="text-muted-foreground">
                  {row.lastEarningsDate}
                </TableCell>
                <TableCell className="text-muted-foreground">
                  {row.nextEarningsDate}
                </TableCell>
                <TableCell>
                  <Button asChild size="sm" variant="secondary">
                    <a
                      href={`https://www.tradingview.com/chart?symbol=${row.symbol}`}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      View
                    </a>
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

export default StockGrowthTable;
