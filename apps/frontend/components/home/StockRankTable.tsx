"use client";

import { AnyObject } from "antd/es/_util/type";
import { Table, Alert, Row, Skeleton, Tooltip } from "antd";
import type { TableColumnsType } from "antd";
import React, { useState, useCallback } from "react";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { EpsGrowthRankingResponse } from "@/types/epsGrowthRanking";

interface StockRankTableProps {
  style?: React.CSSProperties;
  accessLevel?: 1 | 2 | 3; // 1 = basic, 2 = premium, 3 = admin
}

const StockRankTable: React.FC<StockRankTableProps> = ({
  style,
  accessLevel = 1,
}) => {
  const [columns, setColumns] = useState<TableColumnsType<AnyObject>>([]);
  const [pageSize, setPageSize] = useState(10);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalRecords, setTotalRecords] = useState(0);
  const [tableData, setTableData] = useState<
    { [key: string]: string | number }[]
  >([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [data, setData] = useState<EpsGrowthRankingResponse | null>(null);

  // Convert API data to table format
  const processTableData = useCallback((apiData: EpsGrowthRankingResponse) => {
    const filteredData = apiData.data.filter((_, index) => {
      const rank = index + 1;
      // Public users can see rank 21+
      if (!accessLevel) return rank >= 21;
      // Basic users (logged in) can see rank 11-20
      if (accessLevel === 1) return rank >= 11;
      // Premium users can see rank 1-10
      if (accessLevel === 2) return rank >= 1;
      // Admin can see all ranks
      return true;
    });

    return filteredData.map((item, index) => ({
      key: index,
      symbol: item.symbol,
      companyName: item.company_name,
      currentEps: item.eps_diluted?.toFixed(2) ?? 'N/A',
      previousEps: item.previous_eps_diluted?.toFixed(6) ?? 'N/A',
      epsGrowth: item.eps_growth ? (item.eps_growth > 999999 ? '>999999' : item.eps_growth.toFixed(2)) : 'N/A',
      reportDate: new Date(item.report_date).toLocaleDateString(),
      market: item.market_code,
      quarter: item.quarter,
      year: item.year
    }));
  }, []);

  // Create table columns
  const createColumns = useCallback(() => {
    const baseColumns = [
      {
        title: <Tooltip title="Stock symbol">Symbol</Tooltip>,
        dataIndex: "symbol",
        key: "symbol",
        width: 100,
      },
      {
        title: (
          <Tooltip title="Full registered name of the company">
            Company Name
          </Tooltip>
        ),
        dataIndex: "companyName",
        key: "companyName",
        width: 200,
      },
      {
        title: (
          <Tooltip title="Latest reported Earnings Per Share">
            Current EPS
          </Tooltip>
        ),
        dataIndex: "currentEps",
        key: "currentEps",
        width: 120,
        render: (value: string, record: any) => (
          <Tooltip title={`Previous EPS: ${record.previousEps}`}>
            <span>{value}</span>
          </Tooltip>
        ),
        sorter: (a: any, b: any) =>
          (a.currentEps === 'N/A' ? -Infinity : parseFloat(a.currentEps)) - 
          (b.currentEps === 'N/A' ? -Infinity : parseFloat(b.currentEps)),
      },
      {
        title: <Tooltip title="Stock market code">Market</Tooltip>,
        dataIndex: "market",
        key: "market",
        width: 100,
      },
      {
        title: (
          <Tooltip title="Percentage change in EPS from previous to current period">
            EPS Growth (%)
          </Tooltip>
        ),
        dataIndex: "epsGrowth",
        key: "epsGrowth",
        width: 150,
        sorter: (a: any, b: any) =>
          (a.epsGrowth === 'N/A' || a.epsGrowth === '>100000' ? -Infinity : parseFloat(a.epsGrowth)) - 
          (b.epsGrowth === 'N/A' || b.epsGrowth === '>100000' ? -Infinity : parseFloat(b.epsGrowth)),
        render: (value: string) => {
          if (value === 'N/A' || value === '>100000') {
            return <span>{value}</span>;
          }
          const numValue = parseFloat(value);
          return (
            <span style={{ color: numValue >= 0 ? "#52c41a" : "#f5222d" }}>
              {numValue >= 0 ? "+" : ""}
              {value}%
            </span>
          );
        },
      },
      {
        title: (
          <Tooltip title="Date of latest earnings report">Period</Tooltip>
        ),
        dataIndex: "reportDate",
        key: "reportDate",
        width: 200,
        render: (_: string, record: any) => (
          <span>
            Q{record.quarter} {record.year} ({new Date(record.reportDate).toLocaleDateString()})
          </span>
        ),
      },
    ];

    const rowNumberColumn = {
      title: "No.",
      dataIndex: "rowNumber",
      key: "rowNumber",
      width: 50,
      fixed: "left" as const,
      render: (_: string, __: any, index: number) => {
        return (currentPage - 1) * pageSize + index + 1;
      },
    };

    return [rowNumberColumn, ...baseColumns];
  }, [currentPage, pageSize]);

  // Function to fetch data
  const fetchData = useCallback(async () => {
    try {
      setIsLoading(true);
      const response = await fetchEpsGrowthRanking({
        skip: (currentPage - 1) * pageSize,
        limit: pageSize,
      });

      setData(response);
      const processedData = processTableData(response);
      setColumns(createColumns());
      setTableData(processedData);
      setTotalRecords(response.metadata.total);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to fetch data"));
    } finally {
      setIsLoading(false);
    }
  }, [currentPage, pageSize, processTableData, createColumns]);

  // Initial data fetch and setup pagination changes
  React.useEffect(() => {
    fetchData();
  }, [fetchData]);

  if (isLoading && !data) {
    return (
      <div
        style={{
          minHeight: 500,
          width: "100%",
          background: "var(--background-color)",
          borderRadius: 8,
          padding: 24,
        }}
      >
        <Skeleton active paragraph={{ rows: 10 }} title={false} />
      </div>
    );
  }

  if (!accessLevel && currentPage === 1) {
    return (
      <Alert
        message="Login Required"
        description="Please login to view ranks 11-20. Premium users can view ranks 1-10."
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
      />
    );
  }

  if (error || !data?.data) {
    return (
      <Alert
        message="Error Loading Data"
        description={error?.message || "Failed to load stock data"}
        type="error"
        showIcon
      />
    );
  }

  return (
    <Row justify="center" align="middle" style={{ width: "100%", ...style }}>
      <div className="scroll-container">
        <Table
          style={{ width: "100%" }}
          columns={columns}
          dataSource={tableData}
          loading={isLoading}
          pagination={{
            pageSize,
            current: currentPage,
            total: totalRecords,
            responsive: true,
            showSizeChanger: true,
            pageSizeOptions: [10, 20, 50, 100],
            onChange: (page, size) => {
              setCurrentPage(page);
              setPageSize(size);
            },
          }}
          scroll={{
            x: "max-content",
            y: 500,
            scrollToFirstRowOnChange: true,
          }}
          size="large"
        />
      </div>
    </Row>
  );
};

export default StockRankTable;

// TODO: Future features:
// - Add more advanced filtering options (by date range, EPS threshold, growth percentage)
// - Add column customization with saveable presets
// - Add export functionality (CSV, Excel with calculated metrics)
// - Add data visualization options (growth trends, comparative charts)
// - Add real-time updates with websocket connection
// - Add sorting indicators with multi-column sort support
// - Add column resizing and reordering
// - Add row selection for multi-stock comparison
// - Add favorites/watchlist feature with email alerts
// - Add historical data comparison with quarterly/yearly views
// - Add conditional formatting for significant EPS changes
// - Implement caching for paginated data
// - Consider adding a debounce for rapid page changes
// - Add validation for pagination parameters
// - Implement persistent user preferences (page size, column order, sorting)
// - Add proper access control for advanced features
// - Add tooltip explanations for metrics
// - Add performance indicators (green/red) for growth metrics
// - Consider adding "average sector EPS" comparison
// - Add YoY and QoQ growth rate calculations
