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

interface TableDataType {
  key: number;
  symbol: string;
  companyName: string;
  currentEps: string;
  previousEps: string;
  epsGrowth: string;
  reportDate: string;
  market: string;
  quarter: number;
  year: number;
}

interface StockRankTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableDataType[];
}

const StockRankTable: React.FC<StockRankTableProps> = ({
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
              <TableHead className="font-semibold text-primary/70">No.</TableHead>
              <TableHead className="font-semibold text-primary/70">Symbol</TableHead>
              <TableHead className="font-semibold text-primary/70">Company Name</TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Latest reported Earnings Per Share">Current EPS</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">Market</TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Percentage change in EPS from previous to current period">
                  EPS Growth (%)
                </span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Date of latest earnings report">Period</span>
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.map((row, index) => (
              <TableRow key={row.key} className="hover:bg-primary/5 transition-colors">
                <TableCell>{index + 1}</TableCell>
                <TableCell className="font-medium">{row.symbol}</TableCell>
                <TableCell className="text-muted-foreground">{row.companyName}</TableCell>
                <TableCell className="font-medium" title={`Previous EPS: ${row.previousEps}`}>
                  {row.currentEps}
                </TableCell>
                <TableCell className="text-muted-foreground">{row.market}</TableCell>
                <TableCell>
                  {row.epsGrowth === "N/A" || row.epsGrowth === ">999999" ? (
                    <span>{row.epsGrowth}</span>
                  ) : (
                    <span
                      className={`font-medium ${
                        parseFloat(row.epsGrowth) >= 0
                          ? "text-green-500"
                          : "text-rose-500"
                      }`}
                    >
                      {parseFloat(row.epsGrowth) >= 0 ? "+" : ""}
                      {row.epsGrowth}%
                    </span>
                  )}
                </TableCell>
                <TableCell className="text-muted-foreground">
                  Q{row.quarter} {row.year} ({row.reportDate})
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

export default StockRankTable;
