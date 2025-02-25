/**
 * Utility functions for table data processing
 * TODO: Add support for custom column renderers
 * TODO: Add sorting functionality
 * TODO: Add pagination support
 */

interface Column {
  label: string;
  metric: string;
}

interface Row {
  asset: {
    ticker: string;
  };
  data: Array<{
    value: string | number;
  }>;
}

interface ApiResponse {
  columns: Column[];
  rows: Row[];
}

/**
 * Creates table columns from API response
 * @param result - API response containing columns data
 * @returns Array of table column configurations
 */
export function createTableColumns(result: ApiResponse) {
  return result.columns.map((col: Column, index: number) => ({
    title: col.label,
    dataIndex: col.metric,
    key: col.metric,
    ...(index < 2 && {
      fixed: "left" as "left" | "right" | undefined,
      width: 100,
    }),
  }));
}

/**
 * Creates table data with access level visibility
 * @param result - API response containing stock data
 * @param accessLevel - User access level (1 = basic, 2 = premium, 3 = admin)
 * @returns Filtered table data based on user access level
 * TODO: Implement proper access control system
 */
export function createTableData(result: ApiResponse, accessLevel: number = 1) {
  return result.rows
    .map((row: Row, index: number) => {
      const rowData: { [key: string]: string | number } = {
        key: index,
        ticker: row.asset.ticker,
      };

      row.data.forEach((dataItem: { value: string | number }, idx: number) => {
        const columnMetric = result.columns[idx]?.metric;
        if (columnMetric) {
          // Apply access level visibility rules
          if (columnMetric === 'stock_number') {
            const stockNumber = Number(dataItem.value);
            
            // TODO: Replace with proper access control system
            // Temporary visibility rules:
            // Basic users (1): Limited access
            // Premium users (2): More access
            // Admin users (3): Full access
            if (accessLevel === 1 && stockNumber < 21) {
              rowData[columnMetric] = "Subscribe for access";
            } else if (accessLevel === 2 && stockNumber < 11) {
              rowData[columnMetric] = "Premium content";
            } else {
              rowData[columnMetric] = dataItem.value || "-";
            }
          } else {
            rowData[columnMetric] = dataItem.value || "-";
          }
        }
      });

      return rowData;
    });
}
