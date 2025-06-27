/**
 * Utility functions for table data processing
 * TODO: Add support for custom column renderers
 * TODO: Add sorting functionality
 * TODO: Add pagination support
 */

/**
 * Applies access control rules to a stock number value.
 * @param value - The stock number value.
 * @param accessLevel - User access level.
 * @returns The display value based on access level.
 */
function applyStockNumberAccessControl(value: string | number, accessLevel: number): string | number {
  const stockNumber = Number(value);
  if (accessLevel === 1 && stockNumber < 21) {
    return "Subscribe for access";
  } else if (accessLevel === 2 && stockNumber < 11) {
    return "Premium content";
  }
  return value || "-";
}

/**
 * Stub for custom column renderers.
 * @param columnMetric - The metric of the column.
 * @param value - The value to render.
 * @returns The rendered value.
 */
function customColumnRenderer(columnMetric: string, value: string | number): string | number {
  // TODO: Implement custom rendering logic per column
  // Suppress unused parameter warning by referencing columnMetric
  if (columnMetric === '') {
    return value;
  }
  return value;
}

/**
 * Stub for sorting table data.
 * @param data - The table data array.
 * @param sortBy - The column to sort by.
 * @param direction - 'asc' or 'desc'.
 * @returns Sorted data array.
 */
// Commented out to suppress unused function error
// function sortTableData<T>(data: T[], sortBy: string, direction: 'asc' | 'desc'): T[] {
//   // TODO: Implement sorting logic
//   return data;
// }

/**
 * Stub for paginating table data.
 * @param data - The table data array.
 * @param page - Current page number.
 * @param pageSize - Number of items per page.
 * @returns Paginated data array.
 */
// Commented out to suppress unused function error
// function paginateTableData<T>(data: T[], page: number, pageSize: number): T[] {
//   // TODO: Implement pagination logic
//   return data;
// }

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
 * @param accessLevel - User access level (1 = basic, 2 = premium)
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
          let value = dataItem.value;
          // Apply access control for stock_number
          if (columnMetric === 'stock_number') {
            value = applyStockNumberAccessControl(value, accessLevel);
          }
          // Apply custom renderer (stub)
          value = customColumnRenderer(columnMetric, value);
          rowData[columnMetric] = value || "-";
        }
      });

      return rowData;
    });
}
