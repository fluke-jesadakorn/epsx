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

interface StockRankTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableStockData[];
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
              <TableHead className="font-semibold text-primary/70">
                <span title="Current EPS">EPS</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Quarter-over-Quarter EPS Growth">EPS Growth Q (%)</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Next Quarter EPS Forecast">Next Q EPS</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">Sector</TableHead>
              <TableHead className="font-semibold text-primary/70">Exchange</TableHead>
              <TableHead className="font-semibold text-primary/70">
                <span title="Next Earnings Date">Earnings Date</span>
              </TableHead>
              <TableHead className="font-semibold text-primary/70">Tags</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.map((row, index) => (
              <TableRow key={row.symbol} className="hover:bg-primary/5 transition-colors">
                <TableCell>{index + 1}</TableCell>
                <TableCell className="font-medium">{row.symbol}</TableCell>
                <TableCell className="font-medium">{row.eps}</TableCell>
                <TableCell>
                  <span
                    className={`font-medium ${
                      parseFloat(row.epsGrowthQ) >= 0
                        ? "text-green-500"
                        : "text-rose-500"
                    }`}
                  >
                    {parseFloat(row.epsGrowthQ) >= 0 ? "+" : ""}
                    {row.epsGrowthQ}%
                  </span>
                </TableCell>
                <TableCell className="font-medium">{row.epsNextQuarter}</TableCell>
                <TableCell className="text-muted-foreground">{row.sector}</TableCell>
                <TableCell className="text-muted-foreground">{row.exchange}</TableCell>
                <TableCell className="text-muted-foreground">{row.earningsDate}</TableCell>
                <TableCell>
                  <div className="flex flex-wrap gap-1">
                    {row.tags.map((tag) => (
                      <span
                        key={tag}
                        className="px-2 py-1 text-xs rounded-full bg-primary/10 text-primary"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
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
