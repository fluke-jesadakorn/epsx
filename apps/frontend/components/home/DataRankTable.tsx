"use client";

import { ChevronDown, ChevronUp, LayoutGrid, Table2 } from "lucide-react";
import React, { useState, useEffect } from "react";

import { Card, CardContent } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

import { Button } from "../ui/button";

import type { TableDataMetrics } from "@/types/stockFetchData";

interface ColumnDef {
  key: keyof TableDataMetrics | "number" | "chart";
  header: string;
  tooltip?: string;
  render?: (row: TableDataMetrics, index: number) => React.ReactNode;
}

interface DataRankTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableDataMetrics[];
  columns?: ColumnDef[];
  defaultView?: 'table' | 'card';
}

const defaultColumns: ColumnDef[] = [
  { key: "number", header: "No." },
  { key: "symbol", header: "Symbol" },
  { key: "name", header: "Name" },
  { key: "valueIndex", header: "Value Index", tooltip: "Current Value Score" },
  { key: "growthRate", header: "Growth Rate", tooltip: "Value Change Percentage" },
  { key: "activityScore", header: "Activity Score", tooltip: "Engagement Level" },
  { key: "marketSize", header: "Market Size", tooltip: "Total Market Presence" },
  { key: "growthFactor", header: "Growth Factor", tooltip: "Growth Potential Indicator" },
  { key: "sector", header: "Sector" },
  { key: "country", header: "Country" },
  { key: "exchange", header: "Exchange" },
  { key: "entryPhase", header: "Entry Phase", tooltip: "Optimal Entry Time" },
  { key: "phaseStatus", header: "Phase Status", tooltip: "Current Phase Status" },
  { key: "metricScore", header: "Metric Score", tooltip: "Current Metric Score" },
  { key: "growthIndicator", header: "Growth Indicator", tooltip: "Growth Potential" },
  { key: "currentMetric", header: "Current Metric", tooltip: "Current Metric Value" },
  { key: "predictedMetric", header: "Predicted Metric", tooltip: "Predicted Metric Value" },
  { key: "lastAnalysisDate", header: "Last Analysis", tooltip: "Date of Last Analysis" },
  { key: "nextAnalysisDate", header: "Next Analysis", tooltip: "Date of Next Analysis" },
  { key: "chart", header: "Analytics", tooltip: "Open Analytics View" },
];

interface DataCardProps {
  data: TableDataMetrics;
  index: number;
}

function DataCard({ data, index }: DataCardProps): React.JSX.Element {
  const [expanded, setExpanded] = useState(false);
  const [isPressed, setIsPressed] = useState(false);

  return (
    <Card 
      className={`w-full transition-all duration-200 hover:shadow-lg border-blue-500/10 hover:border-blue-500/30 relative ${
        isPressed ? "scale-[0.98] opacity-90" : ""
      }`}
      onTouchStart={() => setIsPressed(true)}
      onTouchEnd={() => setIsPressed(false)}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseLeave={() => setIsPressed(false)}
    >
      {/* Rank Number Badge */}
      <div className="absolute -top-2 -left-2 w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 flex items-center justify-center text-white text-sm font-bold shadow-lg">
        {index + 1}
      </div>
      <CardContent className="p-4 pt-6">
        {/* Primary Information */}
        <div className="flex justify-between items-start mb-2">
          <div>
            <div className="text-lg font-semibold text-primary">
              {data.symbol}
            </div>
            <div className="text-sm text-muted-foreground line-clamp-1">
              {data.name}
            </div>
          </div>
          <div className="text-right">
            <div className="font-medium">{data.valueIndex} {data.currency}</div>
            <div
              className={`text-sm font-medium ${
                parseFloat(data.growthRate) >= 0
                  ? "text-green-500"
                  : "text-rose-500"
              }`}
            >
              {parseFloat(data.growthRate) >= 0 ? "+" : ""}
              {data.growthRate}%
            </div>
          </div>
        </div>

        {/* Market Size - Always visible */}
        <div className="text-sm text-muted-foreground mb-2 flex items-center gap-2">
          <span>Market Size:</span>
          <span className="font-medium">{data.marketSize}</span>
        </div>

        {/* Action Buttons Row */}
        <div className="flex justify-between items-center gap-2 mt-2">
          <Button
            size="sm"
            variant="ghost"
            className="w-full"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? (
              <ChevronUp className="h-4 w-4 mr-1" />
            ) : (
              <ChevronDown className="h-4 w-4 mr-1" />
            )}
            {expanded ? "Less" : "More"}
          </Button>
          <Button asChild size="sm" variant="secondary" className="w-[100px]">
            <a
              href={`https://www.tradingview.com/chart?symbol=${data.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              Analytics
            </a>
          </Button>
        </div>

        {/* Expandable Content */}
        <div className={`mt-4 pt-4 border-t border-border/50 grid gap-3 text-sm overflow-hidden transition-all duration-300 ${
          expanded ? "opacity-100 max-h-[500px]" : "opacity-0 max-h-0"
        }`}>
          <div className="grid grid-cols-2 gap-2">
              <div>
                <div className="text-muted-foreground">Sector</div>
                <div>{data.sector}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Growth Factor</div>
                <div>{data.growthFactor}</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-muted-foreground mb-1">Entry Phase</div>
                <div className={`${data.entryPhase?.active ? "text-green-500 font-medium" : ""}`}>
                  {data.entryPhase?.date || "N/A"}
                  {data.entryPhase?.active && (
                    <span className="block text-xs">Active</span>
                  )}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground mb-1">Phase Status</div>
                <div className={`${data.phaseStatus?.active ? "text-yellow-500 font-medium" : ""}`}>
                  {data.phaseStatus?.date || "N/A"}
                  {data.phaseStatus?.active && (
                    <span className="block text-xs">
                      {data.phaseStatus.type === "monitor" ? "Monitor" : "Exit"}
                    </span>
                  )}
                </div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-2">
            <div>
              <div className="text-muted-foreground">Country</div>
              <div>{data.country}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Exchange</div>
              <div>{data.exchange}</div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

function DataRankTable({
  style,
  className,
  data,
  columns = defaultColumns,
  defaultView = 'card'
}: DataRankTableProps): React.JSX.Element {
  const [viewMode, setViewMode] = useState<'table' | 'card'>(defaultView);
  const [isMobile, setIsMobile] = useState(false);
  const [isNarrow, setIsNarrow] = useState(false);

  useEffect(() => {
    const checkScreenSize = () => {
      const width = typeof window !== 'undefined' ? window.innerWidth : 0;
      setIsMobile(width < 768);
      setIsNarrow(width < 1200);
      // Only apply responsive switching for table view
      if (defaultView === 'table') {
        setViewMode(width < 768 ? 'card' : 'table');
      }
    }

    checkScreenSize();

    if (typeof window !== 'undefined') {
      window.addEventListener('resize', checkScreenSize);
      return () => window.removeEventListener('resize', checkScreenSize);
    }
  }, [defaultView]);

  const renderTableView = () => (
    <div className="relative w-full">
      <div className="rounded-xl border bg-card max-w-[100vw] overflow-x-auto custom-scrollbar">
        <div className="min-w-[1200px]">
          <Table>
            <TableHeader className="bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-pink-500/5 backdrop-blur-sm sticky top-0 z-10">
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
                  column.key === "symbol" || column.key === "name" || column.key === "valueIndex"
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
      {isNarrow && (
        <div className="mt-2 text-sm text-muted-foreground text-center">
          Scroll horizontally to see more →
        </div>
      )}
    </div>
  );

  const renderCardView = () => (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {data.map((item, index) => (
        <DataCard key={item.symbol} data={item} index={index} />
      ))}
    </div>
  );

  // Render cell content for table view
  const renderCell = (row: TableDataMetrics, column: ColumnDef, index: number) => {
    if (column.render) {
      return column.render(row, index);
    }

    switch (column.key) {
      case "number":
        return index + 1;
      case "valueIndex":
        return `${row.valueIndex || 'N/A'} ${row.currency || ''}`.trim();
      case "growthRate":
        const growthRate = parseFloat(row.growthRate || '0');
        return (
          <span className={`font-medium ${growthRate >= 0 ? "text-green-500" : "text-rose-500"}`}>
            {growthRate >= 0 ? "+" : ""}
            {!isNaN(growthRate) ? `${growthRate.toFixed(2)}%` : 'N/A'}
          </span>
        );
      case "activityScore":
        return row.activityScore || 'N/A';
      case "marketSize":
        return row.marketSize || 'N/A';
      case "growthFactor":
        return row.growthFactor || 'N/A';
      case "sector":
        return row.sector || 'N/A';
      case "country":
        return row.country || 'N/A';
      case "exchange":
        return row.exchange || 'N/A';
      case "entryPhase":
        return (
          <div className="flex flex-col gap-1">
            <span className={`text-xs ${row.entryPhase?.active ? "text-green-500 font-medium" : "text-muted-foreground"}`}>
              {row.entryPhase?.date || "N/A"}
            </span>
            {row.entryPhase?.active && (
              <span className="text-xs text-green-500 font-medium">Active</span>
            )}
          </div>
        );
      case "phaseStatus":
        return (
          <div className="flex flex-col gap-1">
            <span className={`text-xs ${row.phaseStatus?.active ? "text-yellow-500 font-medium" : "text-muted-foreground"}`}>
              {row.phaseStatus?.date || "N/A"}
            </span>
            {row.phaseStatus?.active && (
              <span className="text-xs text-yellow-500 font-medium">
                {row.phaseStatus?.type === "monitor" ? "Monitor" : "Exit"}
              </span>
            )}
          </div>
        );
      case "metricScore":
        return row.metricScore || 'N/A';
      case "growthIndicator":
        return row.growthIndicator || 'N/A';
      case "currentMetric":
        return row.currentMetric || 'N/A';
      case "predictedMetric":
        return row.predictedMetric || 'N/A';
      case "lastAnalysisDate":
        return row.lastAnalysisDate || 'N/A';
      case "nextAnalysisDate":
        return row.nextAnalysisDate || 'N/A';
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
        const value = row[column.key as keyof TableDataMetrics];
        if (typeof value === "string") {
          return value;
        }
        return null;
    }
  };

  return (
    <div className={`w-full space-y-4 p-4 sm:p-6 ${className || ""}`} style={style}>
      {/* Title and View Toggle */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
        <div>
          <h2 className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
            Top Data Rankings
          </h2>
          <div className="w-20 h-1 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full mt-2" />
        </div>
        
        {/* Only show view toggle for table default view */}
        {!isMobile && defaultView === 'table' && (
          <div className="flex items-center gap-2">
            <Button
              variant={viewMode === 'table' ? "default" : "outline"}
              size="sm"
              onClick={() => setViewMode('table')}
              className="w-[100px]"
            >
              <Table2 className="h-4 w-4 mr-2" />
              Table
            </Button>
            <Button
              variant={viewMode === 'card' ? "default" : "outline"}
              size="sm"
              onClick={() => setViewMode('card')}
              className="w-[100px]"
            >
              <LayoutGrid className="h-4 w-4 mr-2" />
              Cards
            </Button>
          </div>
        )}
      </div>

      {/* Content based on view mode */}
      <div className="-mx-4 sm:-mx-6 px-4 sm:px-6">
        {viewMode === 'table' ? renderTableView() : renderCardView()}
      </div>
    </div>
  );
};

export default DataRankTable;
