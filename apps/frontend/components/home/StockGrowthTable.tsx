"use client";

import React from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { TableStockData } from "@/types/stockFetchData";
import { Button } from "../ui/button";

interface StockGrowthTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableStockData[];
}

const StockGrowthTable: React.FC<StockGrowthTableProps> = ({
  style,
  className,
  data,
}) => {
  return (
    <div className={`w-full ${className || ""}`} style={style}>
      <div className="scroll-shadow-container custom-scrollbar rounded-xl border bg-card">
        <Table>
          <TableHeader className="bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-pink-500/5 backdrop-blur-sm">
            <TableRow className="hover:bg-transparent">
              <TableHead className="font-semibold text-primary/70">
                No.
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Symbol
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Name
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Growth %
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Current EPS
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Next EPS
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Price
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Price Change Percentage">Change %</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Trading Volume">Volume</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Sector
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Country
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Last Earnings
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                Next Earnings
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Open TradingView Chart">Chart</span>
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
                <TableCell className="font-medium">{row.currentQuarterEps}</TableCell>
                <TableCell className="font-medium">{row.nextEps}</TableCell>
                <TableCell className="font-medium">
                  {row.price} {row.currency}
                </TableCell>
                <TableCell>
                  <span
                    className={`font-medium ${
                      parseFloat(row.changePercent) >= 0
                        ? "text-green-500"
                        : "text-rose-500"
                    }`}
                  >
                    {parseFloat(row.changePercent) >= 0 ? "+" : ""}
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
