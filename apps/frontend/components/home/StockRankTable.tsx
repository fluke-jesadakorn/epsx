"use client";

import React, { useState, useCallback } from "react";
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  SortingState,
  flexRender,
  ColumnDef,
} from "@tanstack/react-table";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { EpsGrowthRankingResponse } from "@/types/epsGrowthRanking";
import { 
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { AlertCircle, Info } from "lucide-react";

// ... interfaces remain the same ...
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
  accessLevel?: 1 | 2 | 3;
}

const StockRankTable: React.FC<StockRankTableProps> = ({
  style,
  className,
  accessLevel = 1,
}) => {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pageSize, setPageSize] = useState(10);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalRecords, setTotalRecords] = useState(0);
  const [tableData, setTableData] = useState<TableDataType[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [data, setData] = useState<EpsGrowthRankingResponse | null>(null);

  const processTableData = useCallback((apiData: EpsGrowthRankingResponse) => {
    const filteredData = apiData.data.filter((_, index) => {
      const rank = index + 1;
      if (!accessLevel) return rank >= 21;
      if (accessLevel === 1) return rank >= 11;
      if (accessLevel === 2) return rank >= 1;
      return true;
    });

    return filteredData.map((item, index) => ({
      key: index,
      symbol: item.symbol,
      companyName: item.company_name,
      currentEps: item.eps_diluted?.toFixed(2) ?? "N/A",
      previousEps: item.previous_eps_diluted?.toFixed(6) ?? "N/A",
      epsGrowth: item.eps_growth
        ? item.eps_growth > 999999
          ? ">999999"
          : item.eps_growth.toFixed(2)
        : "N/A",
      reportDate: new Date(item.report_date).toLocaleDateString(),
      market: item.market_code,
      quarter: item.quarter,
      year: item.year,
    }));
  }, [accessLevel]);

  const columns = React.useMemo<ColumnDef<TableDataType>[]>(
    () => [
      {
        accessorFn: (_, index) => (currentPage - 1) * pageSize + index + 1,
        id: "rowNumber",
        header: "No.",
        size: 50,
      },
      {
        accessorKey: "symbol",
        header: "Symbol",
        size: 100,
      },
      {
        accessorKey: "companyName",
        header: "Company Name",
        size: 200,
      },
      {
        accessorKey: "currentEps",
        header: () => <span title="Latest reported Earnings Per Share">Current EPS</span>,
        size: 120,
        cell: ({ row }) => (
          <span title={`Previous EPS: ${row.original.previousEps}`}>
            {row.original.currentEps}
          </span>
        ),
      },
      {
        accessorKey: "market",
        header: "Market",
        size: 100,
      },
      {
        accessorKey: "epsGrowth",
        header: () => <span title="Percentage change in EPS from previous to current period">EPS Growth (%)</span>,
        size: 150,
        cell: ({ row }) => {
          const value = row.original.epsGrowth;
          if (value === "N/A" || value === ">999999") {
            return <span>{value}</span>;
          }
          const numValue = parseFloat(value);
          return (
            <span className={numValue >= 0 ? "text-green-500" : "text-red-500"}>
              {numValue >= 0 ? "+" : ""}
              {value}%
            </span>
          );
        },
      },
      {
        accessorKey: "reportDate",
        header: () => <span title="Date of latest earnings report">Period</span>,
        size: 200,
        cell: ({ row }) => (
          <span>
            Q{row.original.quarter} {row.original.year} ({row.original.reportDate})
          </span>
        ),
      },
    ],
    [currentPage, pageSize]
  );

  const fetchData = useCallback(async () => {
    try {
      setIsLoading(true);
      const response = await fetchEpsGrowthRanking({
        skip: (currentPage - 1) * pageSize,
        limit: pageSize,
      });

      setData(response);
      const processedData = processTableData(response);
      setTableData(processedData);
      setTotalRecords(response.metadata.total);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to fetch data"));
    } finally {
      setIsLoading(false);
    }
  }, [currentPage, pageSize, processTableData]);

  React.useEffect(() => {
    fetchData();
  }, [fetchData]);

  const table = useReactTable({
    data: tableData,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  if (isLoading && !data) {
    return (
      <div className="min-h-[500px] w-full bg-white rounded-lg p-6 animate-pulse">
        <div className="space-y-4">
          {[...Array(10)].map((_, i) => (
            <div key={i} className="h-8 bg-gray-200 rounded"></div>
          ))}
        </div>
      </div>
    );
  }

  if (!accessLevel && currentPage === 1) {
    return (
      <Alert className="bg-blue-50 border-blue-200">
        <Info className="h-4 w-4 text-blue-600" />
        <AlertDescription className="text-blue-700">
          Please login to view ranks 11-20. Premium users can view ranks 1-10.
        </AlertDescription>
      </Alert>
    );
  }

  if (error || !data?.data) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          {error?.message || "Failed to load stock data"}
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className={`w-full ${className || ''}`} style={style}>
      <div className="scroll-shadow-container custom-scrollbar rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map(headerGroup => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map(header => (
                  <TableHead 
                    key={header.id}
                    className="cursor-pointer hover:bg-gray-50"
                    onClick={header.column.getToggleSortingHandler()}
                    style={{ width: header.getSize() }}
                  >
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext()
                    )}
                    <span className="ml-2">
                      {{
                        asc: "↑",
                        desc: "↓",
                      }[header.column.getIsSorted() as string] ?? null}
                    </span>
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows.map(row => (
              <TableRow key={row.id}>
                {row.getVisibleCells().map(cell => (
                  <TableCell key={cell.id}>
                    {flexRender(
                      cell.column.columnDef.cell,
                      cell.getContext()
                    )}
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <div className="flex items-center justify-between px-4 py-3 border-t border-gray-200">
        <div className="flex items-center space-x-2">
          <select
            className="p-2 text-sm border rounded-md"
            value={pageSize}
            onChange={e => {
              setPageSize(Number(e.target.value));
              setCurrentPage(1);
            }}
          >
            {[10, 20, 50, 100].map(size => (
              <option key={size} value={size}>
                Show {size}
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            onClick={() => setCurrentPage(page => Math.max(1, page - 1))}
            disabled={currentPage === 1}
          >
            Previous
          </Button>
          <span className="px-3 py-1">
            Page {currentPage} of {Math.ceil(totalRecords / pageSize)}
          </span>
          <Button
            variant="outline"
            onClick={() => 
              setCurrentPage(page => 
                Math.min(Math.ceil(totalRecords / pageSize), page + 1)
              )
            }
            disabled={currentPage >= Math.ceil(totalRecords / pageSize)}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
};

export default StockRankTable;
