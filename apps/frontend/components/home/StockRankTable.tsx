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
import { Button } from "../ui/button";

interface TableStockData {
  symbol: string;
  name: string;
  price: string;
  currency: string;
  changePercent: string;
  volume: string;
  marketCap: string;
  peRatio: string;
  sector: string;
  country: string;
  exchange: string;
  startBuy: {
    date: string;
    active: boolean;
  };
  startAction: {
    date: string;
    active: boolean;
    type: "hold" | "sell";
  };
}

interface ColumnDef {
  key: keyof TableStockData | "number" | "chart";
  header: string;
  tooltip?: string;
  render?: (row: TableStockData, index: number) => React.ReactNode;
}

interface StockRankTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableStockData[];
  columns?: ColumnDef[];
}

const defaultColumns: ColumnDef[] = [
  { key: "number", header: "No." },
  { key: "symbol", header: "Symbol" },
  { key: "name", header: "Name" },
  { key: "price", header: "Price" },
  { key: "changePercent", header: "Change %", tooltip: "Price Change Percentage" },
  { key: "volume", header: "Volume", tooltip: "Trading Volume" },
  { key: "marketCap", header: "Market Cap", tooltip: "Market Capitalization" },
  { key: "peRatio", header: "P/E", tooltip: "Price to Earnings Ratio" },
  { key: "sector", header: "Sector" },
  { key: "country", header: "Country" },
  { key: "exchange", header: "Exchange" },
  { key: "startBuy", header: "Start Buy", tooltip: "When to start buying" },
  { key: "startAction", header: "Hold or Sell", tooltip: "When to start holding/selling" },
  { key: "chart", header: "Chart", tooltip: "Open TradingView Chart" },
];

const StockRankTable: React.FC<StockRankTableProps> = ({
  style,
  className,
  data,
  columns = defaultColumns,
}) => {
  const renderCell = (row: TableStockData, column: ColumnDef, index: number) => {
    if (column.render) {
      return column.render(row, index);
    }

    switch (column.key) {
      case "number":
        return index + 1;
      case "price":
        return `${row.price} ${row.currency}`;
      case "changePercent":
        return (
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
        );
      case "startBuy":
        return (
          <div className="flex flex-col gap-1">
            <span
              className={`text-xs ${row.startBuy.active ? "text-green-500 font-medium" : "text-muted-foreground"}`}
            >
              {row.startBuy.date}
            </span>
            {row.startBuy.active && (
              <span className="text-xs text-green-500 font-medium">
                Active
              </span>
            )}
          </div>
        );
      case "startAction":
        return (
          <div className="flex flex-col gap-1">
            <span
              className={`text-xs ${row.startAction.active ? "text-yellow-500 font-medium" : "text-muted-foreground"}`}
            >
              {row.startAction.date}
            </span>
            {row.startAction.active && (
              <span className="text-xs text-yellow-500 font-medium">
                {row.startAction.type === "hold" ? "Hold" : "Sell"}
              </span>
            )}
          </div>
        );
      case "chart":
        return (
          <Button asChild size="sm" variant="secondary">
            <a
              href={`https://www.tradingview.com/chart?symbol=${row.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              View
            </a>
          </Button>
        );
      default:
        const value = row[column.key as keyof TableStockData];
        if (typeof value === "string") {
          return value;
        }
        return null;
    }
  };

  return (
    <div className={`w-full ${className || ""}`} style={style}>
      <div className="scroll-shadow-container custom-scrollbar rounded-xl border bg-card">
        <Table>
          <TableHeader className="bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-pink-500/5 backdrop-blur-sm">
            <TableRow className="hover:bg-transparent">
              {columns.map((column) => (
                <TableHead
                  key={column.key}
                  className="font-semibold text-primary/70"
                >
                  {column.tooltip ? (
                    <span title={column.tooltip}>{column.header}</span>
                  ) : (
                    column.header
                  )}
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.map((row, index) => (
              <TableRow
                key={row.symbol}
                className="hover:bg-primary/5 transition-colors"
              >
                {columns.map((column) => (
                  <TableCell key={column.key} className={
                    column.key === "symbol" || column.key === "name" || column.key === "price"
                      ? "font-medium"
                      : column.key === "sector" || column.key === "country" || column.key === "exchange"
                      ? "text-muted-foreground"
                      : ""
                  }>
                    {renderCell(row, column, index)}
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

export default StockRankTable;
